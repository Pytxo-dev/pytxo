mod commands;
mod models;

use clap::{CommandFactory, Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "pytxo", version, about = "Pytxo agent control plane")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize local Pytxo directories
    Init {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Verify git repo and Pytxo prerequisites
    Doctor {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
    },
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
    /// List ADE CLIs from the registry (PATH detection)
    Agents {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// One-shot shell command without TUI (`--eval "/dry-run"`)
    Shell {
        #[arg(long)]
        eval: String,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
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
    /// Manage modular projects (multi-path workspaces)
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Galaxy human-in-the-loop approval queue
    Hitl {
        #[command(subcommand)]
        action: HitlAction,
    },
    /// List every execution domain the hypervisor has registered
    Domains {
        #[arg(long)]
        json: bool,
        /// Legacy lightweight output (domain paths only, no run health)
        #[arg(long)]
        paths_only: bool,
    },
    /// Cross-repo fleet DAG orchestration
    Fleet {
        #[command(subcommand)]
        action: FleetAction,
    },
    /// BYOK provider registry and key status
    Providers {
        #[arg(long)]
        json: bool,
    },
    /// Trust the current folder at a permission tier
    Trust {
        #[arg(default_value = "orbit")]
        tier: String,
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
    },
    /// Model catalog (list / search / refresh)
    Models {
        #[command(subcommand)]
        action: ModelsAction,
    },
}

#[derive(Subcommand)]
enum ModelsAction {
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
enum HitlAction {
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
enum FleetAction {
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
enum ProjectAction {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        None => {
            if std::env::var("PYTXO_NO_TUI").ok().as_deref() == Some("1") {
                Cli::command().print_help()?;
                println!();
                return Ok(());
            }
            pytxo_tui::run()?;
        }
        Some(Commands::Init { repo }) => {
            commands::init(repo)?;
            println!("Initialized Pytxo.");
        }
        Some(Commands::Doctor { repo, json }) => commands::doctor(repo, json)?,
        Some(Commands::Run {
            agents,
            mut cmd,
            config,
            dry_run,
            keep_worktrees,
            repo,
            ade,
            execution,
        }) => {
            if let Some(ref ade_id) = ade {
                if cmd == "echo pytxo" {
                    let spec = pytxo_core::resolve_ade(ade_id)
                        .ok_or_else(|| anyhow::anyhow!("unknown ADE {ade_id} — try `pytxo agents`"))?;
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
        Some(Commands::Status {
            config,
            repo,
            limit,
            json,
        }) => commands::status(config, repo, limit, json)?,
        Some(Commands::Logs {
            agent,
            tail,
            config,
            repo,
        }) => {
            for line in commands::logs(config, repo, &agent, tail)? {
                println!("{line}");
            }
        }
        Some(Commands::Stop {
            all,
            config,
            repo,
            cleanup_worktrees,
        }) => commands::stop(config, repo, all, cleanup_worktrees).await?,
        Some(Commands::Project { action }) => run_project(action).await?,
        Some(Commands::Hitl { action }) => run_hitl(action)?,
        Some(Commands::Domains { json, paths_only }) => {
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
                        let latest = d
                            .latest_run_status
                            .as_deref()
                            .unwrap_or("—");
                        println!(
                            "{} [{}] active={} latest={}{proj}",
                            d.repo_root, d.status, d.active_runs, latest
                        );
                    }
                }
            }
        }
        Some(Commands::Fleet { action }) => run_fleet(action).await?,
        Some(Commands::Providers { json }) => models::providers_list(json)?,
        Some(Commands::Agents { repo, json }) => {
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
        Some(Commands::Shell { eval, config, repo }) => {
            let events = pytxo_shell::eval_line(config, repo, &eval).await?;
            pytxo_shell::emit_shell_events(&events);
        }
        Some(Commands::Trust { tier, repo }) => {
            let line = if tier.eq_ignore_ascii_case("orbit") {
                "/trust".into()
            } else {
                format!("/trust {tier}")
            };
            let events = pytxo_shell::eval_line(None, repo, &line).await?;
            pytxo_shell::emit_shell_events(&events);
        }
        Some(Commands::Models { action }) => match action {
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
    use pytxo_orchestrate::{fleet_dry_run_json, fleet_init, fleet_run, fleet_status, FleetRunOptions};

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
                    println!("{} [{}] fleet={} started={} finished={}", r.id, r.status, r.fleet_id, r.started_at, fin);
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
