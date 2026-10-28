//! Frozen 1.x commands: the shell, runs, missions, fleets, projects, tiers,
//! accounts, and Review/Apply of finished runs. Built only with the default
//! `full` feature; the npm binary leaves them out. See DECISIONS.md (8, 9).

use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use tracing_subscriber::EnvFilter;

use crate::{account, agent_accounts, commands, mission, models};

/// Hidden from `pytxo --help`: they still run, but they are not the product.
#[derive(Subcommand)]
pub(crate) enum Legacy {
    #[command(hide = true)]
    /// Review a finished run's prepared change (or refresh / discard it)
    Review {
        run_id: String,
        /// Re-prepare a stale review against the current project
        #[arg(long)]
        refresh: bool,
        /// Drop the prepared change without writing anything
        #[arg(long)]
        discard: bool,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    /// Apply a reviewed run: writes exactly the reviewed files, refused if anything moved
    Apply {
        run_id: String,
        /// Package digest from `pytxo review` (skips the confirmation prompt)
        #[arg(long)]
        digest: Option<String>,
        /// Apply the package shown without asking
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    /// Sign in to Pytxo for account perks (sharing receipts, Pybench board). Optional.
    Login,
    #[command(hide = true)]
    /// Sign out on this machine
    Logout,
    #[command(hide = true)]
    /// Show the signed-in Pytxo account
    Whoami,
    /// Agent CLI sign-ins: several accounts each for Claude Code and Codex
    #[command(
        after_help = "Each account is the vendor CLI's own config folder; sign-in runs the vendor's command and Pytxo never reads its credentials.
A configured agent can pin one with `account = \"work\"` in pytxo.toml."
    )]
    #[command(hide = true)]
    Accounts {
        #[command(subcommand)]
        action: Option<AccountsAction>,
    },
    #[command(hide = true)]
    /// Initialize local Pytxo directories
    Init {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    /// Verify git repo and Pytxo prerequisites
    Doctor {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
        /// Git + dirs only (skip PTY smoke and network probes)
        #[arg(long)]
        quick: bool,
    },
    #[command(hide = true)]
    Run {
        #[arg(long, default_value = "3")]
        agents: usize,
        #[arg(long, default_value = "echo pytxo")]
        cmd: String,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        keep_worktrees: bool,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        /// Registry ADE id (e.g. cursor, claude) — sets default spawn command
        #[arg(long)]
        ade: Option<String>,
        /// `pty` (default) or `subprocess`
        #[arg(long)]
        execution: Option<String>,
    },
    #[command(hide = true)]
    /// Natural-language mission: plan → approve → isolated run → report
    Mission {
        /// Mission description
        text: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        /// Skip interactive approve prompt
        #[arg(long)]
        yes: bool,
        /// Print plan JSON and exit (no dispatch)
        #[arg(long)]
        json: bool,
        /// Preferred ADE id (claude, codex, opencode, …)
        #[arg(long)]
        ade: Option<String>,
        /// Write plan JSON to this path before approve
        #[arg(long)]
        plan_file: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    /// List ADE CLIs from the registry (PATH detection)
    Agents {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
    },
    #[command(hide = true)]
    /// The interactive shell, or one shell command with `--eval "/dry-run"`
    Shell {
        #[arg(long)]
        eval: Option<String>,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    Status {
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    #[command(hide = true)]
    Logs {
        #[arg(long)]
        agent: String,
        #[arg(long, default_value = "50")]
        tail: usize,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    Stop {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        cleanup_worktrees: bool,
    },
    #[command(hide = true)]
    /// Manage modular projects (multi-path workspaces)
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    #[command(hide = true)]
    /// Galaxy human-in-the-loop approval queue
    Hitl {
        #[command(subcommand)]
        action: HitlAction,
    },
    #[command(hide = true)]
    /// List every execution domain the hypervisor has registered
    Domains {
        #[arg(long)]
        json: bool,
        /// Legacy lightweight output (domain paths only, no run health)
        #[arg(long)]
        paths_only: bool,
    },
    #[command(hide = true)]
    /// Cross-repo fleet DAG orchestration
    Fleet {
        #[command(subcommand)]
        action: FleetAction,
    },
    #[command(hide = true)]
    /// BYOK provider registry and key status
    Providers {
        #[arg(long)]
        json: bool,
    },
    #[command(hide = true)]
    /// Trust the current folder at a permission tier
    Trust {
        #[arg(default_value = "orbit")]
        tier: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    #[command(hide = true)]
    /// Model catalog (list / search / refresh)
    Models {
        #[command(subcommand)]
        action: ModelsAction,
    },
}

#[derive(Subcommand)]
pub(crate) enum AccountsAction {
    /// Every account with its sign-in state
    List {
        #[arg(long)]
        json: bool,
    },
    /// Add an account and open the vendor sign-in (e.g. `add claude work`)
    Add { cli: String, name: String },
    /// Sign an account in again (default: the CLI's own sign-in)
    Login { cli: String, name: Option<String> },
    /// Use this account for new work on that CLI (`default` for the CLI's own)
    Use { cli: String, name: String },
    /// Sign out and delete an account folder
    Remove { cli: String, name: String },
}

#[derive(Subcommand)]
pub(crate) enum ModelsAction {
    /// List models for a provider (cached 24h)
    List {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        refresh: bool,
        #[arg(long)]
        json: bool,
    },
    /// Fuzzy search models
    Search {
        query: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Force refresh provider cache
    Refresh { provider: String },
}

#[derive(Subcommand)]
pub(crate) enum HitlAction {
    /// List pending approval requests for a domain
    List {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Approve a pending request
    Approve {
        id: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Deny a pending request
    Deny {
        id: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum FleetAction {
    /// Create a fleet manifest at ~/.pytxo/fleets/<id>.toml
    Init {
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long = "add")]
        add: Vec<std::path::PathBuf>,
        #[arg(long = "cmd")]
        cmd: Vec<String>,
        #[arg(long, default_value = "1")]
        agents: usize,
    },
    /// Print fleet execution plan JSON
    DryRun {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
    },
    /// Run a fleet DAG (barrier sync across domains)
    Run {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        continue_on_error: bool,
    },
    /// List fleet runs from hypervisor catalog
    Status {
        #[arg(long)]
        id: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum ProjectAction {
    /// Create a project manifest at ~/.pytxo/projects/<id>.toml
    Init {
        id: String,
        #[arg(long)]
        name: Option<String>,
        /// Add a path root (repeatable); the first is primary
        #[arg(long = "add")]
        add: Vec<std::path::PathBuf>,
    },
    /// List a project's path roots
    List {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
    },
    /// Add a path root to an existing project
    Paths {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long = "add")]
        add: std::path::PathBuf,
        #[arg(long)]
        read_only: bool,
    },
    /// Remove a path root from a project (by label or path)
    Remove {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        label: String,
    },
    /// Show recent runs for a project (primary domain WAL)
    Status {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long, default_value = "10")]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    /// Run the task plan once across all writable roots (single run_id)
    Run {
        #[arg(long)]
        manifest: Option<std::path::PathBuf>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long, default_value = "echo pytxo")]
        cmd: String,
        #[arg(long, default_value = "3")]
        agents: usize,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[cfg(windows)]
pub fn is_attempt_host_request(first_arg: Option<&std::ffi::OsStr>) -> bool {
    first_arg == Some(std::ffi::OsStr::new("pytxo-attempt-host/1"))
}

pub fn run(command: Legacy) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    tokio::runtime::Runtime::new()?.block_on(dispatch(command))
}

async fn dispatch(command: Legacy) -> Result<()> {
    match command {
        Legacy::Mission {
            text,
            repo,
            yes,
            json,
            ade,
            plan_file,
        } => {
            mission::run_mission(mission::MissionOptions {
                text,
                repo,
                yes,
                json,
                ade,
                plan_file,
            })
            .await?;
        }
        Legacy::Review {
            run_id,
            refresh,
            discard,
            json,
            config,
            repo,
        } => run_review(&run_id, refresh, discard, json, config, repo)?,
        Legacy::Apply {
            run_id,
            digest,
            yes,
            config,
            repo,
        } => run_apply(&run_id, digest, yes, config, repo)?,
        Legacy::Login => account::login()?,
        Legacy::Logout => account::logout()?,
        Legacy::Whoami => account::whoami()?,
        Legacy::Accounts { action } => match action {
            None => agent_accounts::list(false)?,
            Some(AccountsAction::List { json }) => agent_accounts::list(json)?,
            Some(AccountsAction::Add { cli, name }) => agent_accounts::add(&cli, &name)?,
            Some(AccountsAction::Login { cli, name }) => {
                agent_accounts::login(&cli, name.as_deref())?
            }
            Some(AccountsAction::Use { cli, name }) => agent_accounts::use_account(&cli, &name)?,
            Some(AccountsAction::Remove { cli, name }) => agent_accounts::remove(&cli, &name)?,
        },
        Legacy::Init { repo } => {
            commands::init(repo)?;
            println!("Initialized Pytxo.");
        }
        Legacy::Doctor { repo, json, quick } => commands::doctor(repo, json, quick)?,
        Legacy::Run {
            agents,
            mut cmd,
            config,
            dry_run,
            keep_worktrees,
            repo,
            ade,
            execution,
        } => {
            if let Some(ref ade_id) = ade {
                if cmd == "echo pytxo" {
                    let spec = pytxo_core::resolve_ade(ade_id).ok_or_else(|| {
                        anyhow::anyhow!("unknown ADE {ade_id} — try `pytxo agents`")
                    })?;
                    if !pytxo_core::ade_can_dispatch(spec) {
                        return Err(anyhow::anyhow!(
                            "ADE {ade_id} is detection-only until its permission model is mapped"
                        ));
                    }
                    cmd = spec.default_cmd.to_string();
                }
            }
            let execution = match execution.as_deref() {
                None => None,
                Some(s) => Some(pytxo_orchestrate::ExecutionBackend::parse(s).ok_or_else(
                    || anyhow::anyhow!("--execution must be pty, subprocess, or cloud"),
                )?),
            };
            let run_id = commands::run(commands::RunOptions {
                agents,
                cmd,
                config,
                dry_run,
                keep_worktrees,
                repo,
                execution,
                project: None,
                tasks: None,
                task_cmd_template: None,
                task_prompts: None,
            })
            .await?;
            if !dry_run {
                println!("Run {} finished", run_id);
            }
        }
        Legacy::Status {
            config,
            repo,
            limit,
            json,
        } => commands::status(config, repo, limit, json)?,
        Legacy::Logs {
            agent,
            tail,
            config,
            repo,
        } => {
            for line in commands::logs(config, repo, &agent, tail)? {
                println!("{line}");
            }
        }
        Legacy::Stop {
            all,
            config,
            repo,
            cleanup_worktrees,
        } => commands::stop(config, repo, all, cleanup_worktrees).await?,
        Legacy::Project { action } => run_project(action).await?,
        Legacy::Hitl { action } => run_hitl(action)?,
        Legacy::Domains { json, paths_only } => {
            if paths_only && !json {
                let domains = commands::list_catalog_domains()?;
                if domains.is_empty() {
                    println!("No domains registered yet.");
                } else {
                    for d in domains {
                        let proj = d
                            .project_id
                            .map(|p| format!(" project={p}"))
                            .unwrap_or_default();
                        println!("{} [{}]{proj} — {}", d.repo_root, d.status, d.db_path);
                    }
                }
            } else {
                let domains = commands::list_catalog_domains_enriched()?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&domains)?);
                } else if domains.is_empty() {
                    println!("No domains registered yet.");
                } else {
                    for d in domains {
                        let proj = d
                            .project_id
                            .map(|p| format!(" project={p}"))
                            .unwrap_or_default();
                        let latest = d.latest_run_status.as_deref().unwrap_or("—");
                        println!(
                            "{} [{}] active={} latest={}{proj}",
                            d.repo_root, d.status, d.active_runs, latest
                        );
                    }
                }
            }
        }
        Legacy::Fleet { action } => run_fleet(action).await?,
        Legacy::Providers { json } => models::providers_list(json)?,
        Legacy::Agents { repo, json } => {
            if json {
                #[derive(serde::Serialize)]
                struct AgentRow {
                    id: &'static str,
                    display_name: &'static str,
                    default_cmd: &'static str,
                    on_path: bool,
                }
                let rows: Vec<AgentRow> = pytxo_core::all_ade_clis()
                    .iter()
                    .map(|spec| AgentRow {
                        id: spec.id,
                        display_name: spec.display_name,
                        default_cmd: spec.default_cmd,
                        on_path: pytxo_core::ade_on_path(spec),
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                let events = pytxo_shell::eval_line(None, repo, "/agents").await?;
                pytxo_shell::emit_shell_events(&events);
            }
        }
        Legacy::Shell {
            eval: None,
            config: _,
            repo: _,
        } => pytxo_tui::run()?,
        Legacy::Shell {
            eval: Some(eval),
            config,
            repo,
        } => {
            let events = pytxo_shell::eval_line(config, repo, &eval).await?;
            pytxo_shell::emit_shell_events(&events);
        }
        Legacy::Trust { tier, repo } => {
            let line = if tier.eq_ignore_ascii_case("orbit") {
                "/trust".into()
            } else {
                format!("/trust {tier}")
            };
            let events = pytxo_shell::eval_line(None, repo, &line).await?;
            pytxo_shell::emit_shell_events(&events);
        }
        Legacy::Models { action } => match action {
            ModelsAction::List {
                provider,
                refresh,
                json,
            } => models::models_list(provider.as_deref(), refresh, json)?,
            ModelsAction::Search {
                query,
                provider,
                json,
            } => models::models_search(&query, provider.as_deref(), json)?,
            ModelsAction::Refresh { provider } => models::models_refresh(&provider)?,
        },
    }
    Ok(())
}

fn run_hitl(action: HitlAction) -> anyhow::Result<()> {
    match action {
        HitlAction::List { repo } => {
            let pending = commands::list_hitl_pending(repo)?;
            if pending.is_empty() {
                println!("No pending approvals.");
            }
            for r in pending {
                println!("{} [{}] {} — {}", r.id, r.agent_key, r.action, r.reason);
            }
        }
        HitlAction::Approve { id, repo } => {
            let found = commands::hitl_respond(repo, &id, true)?;
            println!("{}", if found { "approved" } else { "not found" });
        }
        HitlAction::Deny { id, repo } => {
            let found = commands::hitl_respond(repo, &id, false)?;
            println!("{}", if found { "denied" } else { "not found" });
        }
    }
    Ok(())
}

async fn run_fleet(action: FleetAction) -> anyhow::Result<()> {
    use pytxo_orchestrate::{
        fleet_dry_run_json, fleet_init, fleet_run, fleet_status, FleetRunOptions,
    };

    match action {
        FleetAction::Init {
            id,
            name,
            add,
            cmd,
            agents,
        } => {
            if add.is_empty() {
                anyhow::bail!("fleet init requires at least one --add <repo>");
            }
            let cmds: Vec<String> = if cmd.is_empty() {
                vec!["echo pytxo".into(); add.len()]
            } else if cmd.len() == 1 {
                vec![cmd[0].clone(); add.len()]
            } else if cmd.len() != add.len() {
                anyhow::bail!("--cmd count must be 1 or match --add count");
            } else {
                cmd
            };
            let nodes: Vec<_> = add
                .into_iter()
                .zip(cmds)
                .map(|(repo, c)| (repo, c, agents, Vec::new()))
                .collect();
            let path = fleet_init(&id, name, nodes)?;
            println!("Created fleet '{id}' at {}", path.display());
        }
        FleetAction::DryRun { manifest, id } => {
            println!("{}", fleet_dry_run_json(manifest, id)?);
        }
        FleetAction::Run {
            manifest,
            id,
            dry_run,
            continue_on_error,
        } => {
            let result = fleet_run(FleetRunOptions {
                manifest,
                fleet_id: id,
                dry_run,
                continue_on_error,
                ..FleetRunOptions::default()
            })
            .await?;
            if dry_run {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "Fleet {} finished [{}] ({} nodes)",
                    result.fleet_run_id,
                    result.status,
                    result.nodes.len()
                );
            }
        }
        FleetAction::Status { id, limit, json } => {
            let rows = fleet_status(id.as_deref(), limit)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if rows.is_empty() {
                println!("No fleet runs recorded.");
            } else {
                for r in rows {
                    let fin = r.finished_at.as_deref().unwrap_or("—");
                    println!(
                        "{} [{}] fleet={} started={} finished={}",
                        r.id, r.status, r.fleet_id, r.started_at, fin
                    );
                }
            }
        }
    }
    Ok(())
}

async fn run_project(action: ProjectAction) -> anyhow::Result<()> {
    match action {
        ProjectAction::Init { id, name, add } => {
            let path = commands::project_init(&id, name, add)?;
            println!("Created project '{id}' at {}", path.display());
        }
        ProjectAction::List { manifest, id } => {
            let roots = commands::project_roots(manifest, id)?;
            for (label, path, read_only, primary, permission_profile) in roots {
                let mut tags: Vec<String> = Vec::new();
                if primary {
                    tags.push("primary".into());
                }
                if read_only {
                    tags.push("read-only".into());
                }
                if let Some(p) = permission_profile {
                    tags.push(p);
                }
                let suffix = if tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", tags.join(", "))
                };
                println!("{label}: {path}{suffix}");
            }
        }
        ProjectAction::Paths {
            manifest,
            id,
            add,
            read_only,
        } => {
            let path = commands::project_add_root(manifest, id, add, read_only)?;
            println!("Updated project manifest {}", path.display());
        }
        ProjectAction::Remove {
            manifest,
            id,
            label,
        } => {
            let path = commands::project_remove_root(manifest, id, &label)?;
            println!("Removed root '{label}' from {}", path.display());
        }
        ProjectAction::Status {
            manifest,
            id,
            limit,
            json,
        } => {
            let rows = commands::project_status(manifest, id, limit)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if rows.is_empty() {
                println!("No runs for this project.");
            } else {
                for r in rows {
                    let roots = if r.root_ids.is_empty() {
                        "—".to_string()
                    } else {
                        r.root_ids.join(", ")
                    };
                    let by_root = if r.agents_by_root.is_empty() {
                        "—".to_string()
                    } else {
                        r.agents_by_root
                            .iter()
                            .map(|(k, v)| format!("{k}={v}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    println!(
                        "{} [{}] agents={} roots=[{}] by_root={{{}}}",
                        r.run_id, r.status, r.agent_count, roots, by_root
                    );
                }
            }
        }
        ProjectAction::Run {
            manifest,
            id,
            cmd,
            agents,
            config,
            dry_run,
        } => {
            let results = commands::project_run(commands::ProjectRunOptions {
                manifest,
                project_id: id,
                cmd,
                agents,
                config,
                dry_run,
            })
            .await?;
            for r in results {
                println!(
                    "project {} run {} (primary {}) roots [{}]",
                    r.project_id,
                    r.run_id,
                    r.primary_root,
                    r.roots.join(", ")
                );
            }
        }
    }
    Ok(())
}

fn run_review(
    run_id: &str,
    refresh: bool,
    discard: bool,
    json: bool,
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
) -> Result<()> {
    if refresh && discard {
        bail!("choose --refresh or --discard, not both");
    }
    if discard {
        pytxo_orchestrate::discard_run_review(config, repo, run_id)?;
        println!("Discarded the prepared change for {run_id}. Nothing was written to the project.");
        return Ok(());
    }
    if refresh {
        let manifest = pytxo_orchestrate::refresh_run_review(config.clone(), repo.clone(), run_id)?;
        println!(
            "Refreshed review for {run_id}: new digest {}",
            manifest.package_digest
        );
    }
    let (_, store) = pytxo_orchestrate::open_store(config, repo)?;
    let contract = store.get_run_contract(run_id)?.with_context(|| {
        format!("run {run_id} has no review (only completed Orbit/Galaxy runs do)")
    })?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": contract.run_id,
                "apply_status": contract.apply_status,
                "base_revision": contract.base_revision,
                "package_digest": contract.prepared_digest,
                "prepared_at": contract.prepared_at,
                "manifest": contract.prepared_manifest,
                "last_apply_error": contract.last_apply_error,
            }))?
        );
        return Ok(());
    }
    println!("run      {}", contract.run_id);
    println!("status   {}", contract.apply_status);
    if let Some(base) = &contract.base_revision {
        println!("base     {base}");
    }
    let Some(manifest) = &contract.prepared_manifest else {
        println!(
            "No prepared change. Status {} has nothing to review.",
            contract.apply_status
        );
        return Ok(());
    };
    let s = &manifest.summary;
    println!(
        "change   {} added · {} modified · {} deleted · {} bytes",
        s.added, s.modified, s.deleted, s.bytes
    );
    println!("\nfiles");
    for f in &manifest.files {
        println!(
            "  {:<9} {}  ({})",
            format!("{:?}", f.kind).to_lowercase(),
            f.path,
            f.task_id
        );
    }
    if let Some(v) = &manifest.candidate_verification {
        println!("\nchecks on the combined change");
        for c in &v.checks {
            println!(
                "  {}  {}",
                if c.passed { "pass" } else { "FAIL" },
                c.command
            );
        }
    } else {
        println!("\nchecks on the combined change: none recorded");
    }
    if let Some(e) = &contract.last_apply_error {
        println!("\nlast Apply error: {} ({})", e.message, e.code);
    }
    if let Some(digest) = &contract.prepared_digest {
        println!("\ndigest   {digest}");
        if contract.apply_status == "ready" {
            println!("apply    pytxo apply {run_id} --digest {digest}");
        } else if matches!(contract.apply_status.as_str(), "stale" | "review_failed") {
            println!("refresh  pytxo review {run_id} --refresh");
        }
    }
    Ok(())
}

fn run_apply(
    run_id: &str,
    digest: Option<String>,
    yes: bool,
    config: Option<PathBuf>,
    repo: Option<PathBuf>,
) -> Result<()> {
    let digest = match digest {
        Some(d) => d,
        None => {
            // Authorize exactly the package shown here, never "whatever is latest".
            let (_, store) = pytxo_orchestrate::open_store(config.clone(), repo.clone())?;
            let contract = store
                .get_run_contract(run_id)?
                .with_context(|| format!("run {run_id} has no review"))?;
            let shown = contract
                .prepared_digest
                .clone()
                .context("run has no prepared change to apply")?;
            let files = contract
                .prepared_manifest
                .as_ref()
                .map_or(0, |m| m.files.len());
            println!("Apply {files} reviewed file(s) from {run_id}\ndigest {shown}");
            if !yes && !confirm("Write exactly these files to the project?")? {
                println!("Nothing written.");
                return Ok(());
            }
            shown
        }
    };
    let applied = pytxo_orchestrate::apply_run_changes(config, repo, run_id, &digest)?;
    println!(
        "Applied {} file(s) from {run_id} (transaction {}).",
        applied.changes.len(),
        applied.transaction_id
    );
    Ok(())
}

fn confirm(question: &str) -> Result<bool> {
    print!("{question} [y/N] ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "YES"))
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn attempt_host_marker_is_exact_and_ordinary_cli_arguments_are_untouched() {
        use super::is_attempt_host_request;
        use std::ffi::OsStr;

        assert!(is_attempt_host_request(Some(OsStr::new(
            "pytxo-attempt-host/1"
        ))));
        for arg in [
            "--help",
            "run",
            "pytxo-attempt-host",
            "PYTXO-ATTEMPT-HOST/1",
        ] {
            assert!(!is_attempt_host_request(Some(OsStr::new(arg))));
        }
        assert!(!is_attempt_host_request(None));
    }
}
