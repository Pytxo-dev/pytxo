use std::path::PathBuf;

use crate::{parse_line, ShellEvent, ShellSession};

/// Execute one shell line headlessly (no TUI). Used by `pytxo shell --eval`.
pub async fn eval_line(
    config_path: Option<PathBuf>,
    repo: Option<PathBuf>,
    line: &str,
) -> anyhow::Result<Vec<ShellEvent>> {
    let mut session = ShellSession::new(config_path, repo)?;
    Ok(session.handle(parse_line(line)).await)
}

/// Print shell events to stdout/stderr for CLI surfaces.
pub fn emit_shell_events(events: &[ShellEvent]) {
    for ev in events {
        match ev {
            ShellEvent::Output(s) | ShellEvent::PlanPreview(s) => println!("{s}"),
            ShellEvent::Error(s) => eprintln!("error: {s}"),
            ShellEvent::RunStarted(id) => println!("run started: {}", id.0),
            ShellEvent::RunFinished(id) => println!("run finished: {}", id.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn eval_help_prints_commands() {
        let events = eval_line(None, Some(std::env::current_dir().unwrap()), "/help")
            .await
            .unwrap();
        assert!(events
            .iter()
            .any(|e| { matches!(e, ShellEvent::Output(s) if s.contains("/dry-run")) }));
    }

    #[tokio::test]
    async fn eval_agents_lists_registry() {
        let events = eval_line(None, Some(std::env::current_dir().unwrap()), "/agents")
            .await
            .unwrap();
        assert!(events
            .iter()
            .any(|e| { matches!(e, ShellEvent::Output(s) if s.contains("cursor")) }));
    }
}
