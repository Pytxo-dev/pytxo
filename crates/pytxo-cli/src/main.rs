#[cfg(feature = "full")]
mod account;
#[cfg(feature = "full")]
mod agent_accounts;
mod check;
#[cfg(feature = "full")]
mod commands;
#[cfg(feature = "full")]
mod legacy;
#[cfg(feature = "full")]
mod mission;
#[cfg(feature = "full")]
mod models;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "pytxo",
    bin_name = "pytxo",
    version,
    about = "Pytxo: each agent's branch passes. Do they pass together?",
    long_about = "Each agent's branch passes. Do they pass together?

Pytxo merges your coding agents' branches (Claude Code, Codex, Copilot, Cursor...) \
in a throwaway worktree, runs your tests on the combined result, and blocks the merge if it breaks.

Run it with no arguments inside a repository. Nothing in the repository changes.",
    args_conflicts_with_subcommands = true,
    after_help = "Exit codes: 0 ready (or nothing to check), 2 blocked, 3 stale (the target moved), 1 error.
See it work: pytxo demo · Discord: https://discord.gg/AUFRPFjSYv"
)]
struct Cli {
    #[command(flatten)]
    check: check::CheckArgs,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the agent branches together (what `pytxo` runs with no command)
    #[command(
        alias = "land",
        after_help = "Exit codes: 0 ready (or nothing to check), 2 blocked, 3 stale (the target moved), 1 error.
Branches: claude/*, codex/*, copilot/*, cursor/*, devin/*, jules/*, agent/* and every branch in a
linked worktree, with a commit in the last 14 days. Change them with git config pytxo.branches
\"...\" (or PYTXO_BRANCHES), the test command with git config pytxo.check \"...\" (or PYTXO_CHECK)."
    )]
    Check(check::CheckArgs),
    /// See it work: a throwaway repo where two agent branches pass alone and fail together
    Demo {
        /// Keep the demo repository and print where it is
        #[arg(long)]
        keep: bool,
    },
    /// Receipts: every check records its decision under .git/pytxo/receipts
    Receipts {
        #[command(subcommand)]
        action: Option<ReceiptsAction>,
        #[arg(long, global = true)]
        repo: Option<std::path::PathBuf>,
    },
    /// Pybench: offline benchmarks of the check (no model calls)
    #[command(
        after_help = "Suites: land (what per-branch CI vs the combined check would put on main), overhead (cost), signal (scaffold bytes).
Results are synthetic mechanism tests, not real-world failure rates."
    )]
    Bench {
        /// land, overhead, signal or all
        #[arg(long, default_value = "all")]
        suite: String,
        /// Write the JSON report here
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        /// Print the JSON report instead of the table
        #[arg(long)]
        json: bool,
        /// Repository for the signal suite (default: current)
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        /// Publish the summary to the public Pybench board (account perk)
        #[cfg(feature = "full")]
        #[arg(long)]
        submit: bool,
        /// With --submit: keep the result private to your account (Pytxo Pro)
        #[cfg(feature = "full")]
        #[arg(long, requires = "submit")]
        private: bool,
    },
    #[cfg(feature = "full")]
    #[command(flatten)]
    Legacy(legacy::Legacy),
}

#[derive(Subcommand)]
enum ReceiptsAction {
    /// List receipts, newest first
    List {
        #[arg(long)]
        json: bool,
    },
    /// Print one receipt (full id or unique prefix) and check its digest
    Show { id: String },
    /// Publish a receipt as a public page (account perk). Counts and outcomes only.
    #[cfg(feature = "full")]
    Share {
        id: String,
        /// Also publish the changed file paths
        #[arg(long)]
        include_paths: bool,
        /// Only you can open the page, signed in (Pytxo Pro)
        #[arg(long)]
        private: bool,
    },
}

fn main() {
    #[cfg(all(windows, feature = "full"))]
    if legacy::is_attempt_host_request(std::env::args_os().nth(1).as_deref()) {
        // The owned host must enter its barrier before CLI parsing, logging,
        // Tokio initialization, or any normal Pytxo startup code runs.
        std::process::exit(pytxo_runner::owned_launch::attempt_host_main());
    }
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Usage errors exit 1, not clap's 2: 2 means "blocked".
            let code = if e.use_stderr() { 1 } else { 0 };
            let _ = e.print();
            std::process::exit(code);
        }
    };
    let code = run(cli).unwrap_or_else(|e| {
        eprintln!("error: {e:#}");
        1
    });
    std::process::exit(code);
}

fn run(cli: Cli) -> anyhow::Result<i32> {
    match cli.command {
        None => check::run(cli.check),
        Some(Commands::Check(args)) => check::run(args),
        Some(Commands::Demo { keep }) => check::demo(keep),
        Some(Commands::Receipts { action, repo }) => {
            match action {
                None => check::receipts_list(repo, false)?,
                Some(ReceiptsAction::List { json }) => check::receipts_list(repo, json)?,
                Some(ReceiptsAction::Show { id }) => check::receipts_show(repo, &id)?,
                #[cfg(feature = "full")]
                Some(ReceiptsAction::Share {
                    id,
                    include_paths,
                    private,
                }) => account::receipts_share(repo, &id, include_paths, private)?,
            }
            Ok(0)
        }
        Some(Commands::Bench {
            suite,
            out,
            json,
            repo,
            #[cfg(feature = "full")]
            submit,
            #[cfg(feature = "full")]
            private,
        }) => {
            use pytxo_land::bench::{self, Suite};
            let suites = match suite.as_str() {
                "all" => vec![Suite::Land, Suite::Overhead, Suite::Signal],
                "land" => vec![Suite::Land],
                "overhead" => vec![Suite::Overhead],
                "signal" => vec![Suite::Signal],
                other => anyhow::bail!("unknown suite {other}: use land, overhead, signal or all"),
            };
            let report = bench::run(&suites, repo.as_deref(), |step| {
                if !json {
                    eprintln!("  running {step}");
                }
            })?;
            let body = serde_json::to_string_pretty(&report)?;
            if let Some(path) = &out {
                std::fs::write(path, &body)?;
                eprintln!("wrote {}", path.display());
            }
            if json {
                println!("{body}");
            } else {
                print!("{}", bench::render_text(&report));
            }
            #[cfg(feature = "full")]
            if submit {
                account::submit_bench(&report, private)?;
            }
            Ok(0)
        }
        #[cfg(feature = "full")]
        Some(Commands::Legacy(command)) => {
            legacy::run(command)?;
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_leads_with_the_check() {
        let mut help = Vec::new();
        Cli::command().write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();
        assert!(help.contains("Do they pass together?"), "{help}");
        assert!(help.contains("demo"), "{help}");
        for word in ["control plane", "hypervisor", "telemetry"] {
            assert!(!help.contains(word), "{word} in {help}");
        }
    }

    #[test]
    fn no_command_is_the_check_and_land_is_its_alias() {
        let cli = Cli::try_parse_from(["pytxo"]).unwrap();
        assert!(cli.command.is_none());
        let cli = Cli::try_parse_from(["pytxo", "--into", "dev", "-c", "make test", "codex/a"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.check.into.as_deref(), Some("dev"));
        assert_eq!(cli.check.branches, ["codex/a"]);
        let cli = Cli::try_parse_from(["pytxo", "land", "agent/*", "--per-input"]).unwrap();
        match cli.command {
            Some(Commands::Check(args)) => {
                assert_eq!(args.branches, ["agent/*"]);
                assert!(args.per_input);
            }
            _ => panic!("land is check"),
        }
        assert!(matches!(
            Cli::try_parse_from(["pytxo", "demo"]).unwrap().command,
            Some(Commands::Demo { keep: false })
        ));
        assert!(Cli::try_parse_from(["pytxo", "--json", "--markdown"]).is_err());
    }
}
