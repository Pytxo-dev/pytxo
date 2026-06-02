mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "pytxo", version, about = "Pytxo agent control plane")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long)]
        repo: Option<std::path::PathBuf>,
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
    },
    Status {
        #[arg(long)]
        config: Option<std::path::PathBuf>,
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
    },
    Stop {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        config: Option<std::path::PathBuf>,
        #[arg(long)]
        cleanup_worktrees: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Init { repo } => {
            commands::init(repo)?;
            println!("Initialized Pytxo.");
        }
        Commands::Run {
            agents,
            cmd,
            config,
            dry_run,
            keep_worktrees,
            repo,
        } => {
            let run_id = commands::run(commands::RunOptions {
                agents,
                cmd,
                config,
                dry_run,
                keep_worktrees,
                repo,
            })
            .await?;
            if !dry_run {
                println!("Run {} finished", run_id);
            }
        }
        Commands::Status { config, limit, json } => commands::status(config, limit, json)?,
        Commands::Logs {
            agent,
            tail,
            config,
        } => {
            for line in commands::logs(config, &agent, tail)? {
                println!("{line}");
            }
        }
        Commands::Stop {
            all,
            config,
            cleanup_worktrees,
        } => commands::stop(config, all, cleanup_worktrees).await?,
    }
    Ok(())
}
