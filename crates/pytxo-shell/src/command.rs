/// Parsed operator input for the Hypervisor Shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShellInput {
    Slash(SlashCommand),
    Mission(String),
    ReplExit,
    Empty,
}

/// Slash commands routed to `pytxo-orchestrate`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Doctor,
    DryRun {
        agents: usize,
    },
    Run {
        agents: usize,
        cmd: String,
        keep_worktrees: bool,
        ade: Option<String>,
    },
    Logs {
        agent: String,
        tail: usize,
    },
    Stop {
        all: bool,
    },
    Status {
        limit: usize,
    },
    Trust {
        tier: Option<String>,
    },
    Models {
        sub: ModelsSub,
    },
    Agents,
    Use {
        ade: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelsSub {
    List {
        provider: Option<String>,
        refresh: bool,
    },
    Search {
        query: String,
        provider: Option<String>,
    },
    Refresh {
        provider: Option<String>,
    },
}

/// Parse one prompt line into shell input.
pub fn parse_line(line: &str) -> ShellInput {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return ShellInput::Empty;
    }
    if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
        return ShellInput::ReplExit;
    }
    if let Some(rest) = trimmed.strip_prefix('/') {
        return ShellInput::Slash(parse_slash(rest.trim()));
    }
    ShellInput::Mission(trimmed.to_string())
}

fn parse_slash(rest: &str) -> SlashCommand {
    let mut parts = split_args(rest);
    if parts.is_empty() {
        return SlashCommand::Help;
    }
    let head = parts.remove(0).to_ascii_lowercase();
    match head.as_str() {
        "help" | "?" => SlashCommand::Help,
        "doctor" => SlashCommand::Doctor,
        "dry-run" | "dry_run" | "plan" => {
            let mut agents = 0usize;
            if parts.first().map(|s| s.as_str()) == Some("dry-run") {
                parts.remove(0);
            }
            for chunk in parts.chunks(2) {
                if chunk.len() == 2 && chunk[0] == "--agents" {
                    agents = chunk[1].parse().unwrap_or(0);
                }
            }
            SlashCommand::DryRun { agents }
        }
        "run" => {
            let mut agents = 0usize;
            let mut cmd = String::from("echo pytxo");
            let mut cmd_explicit = false;
            let mut ade: Option<String> = None;
            let mut keep_worktrees = false;
            let mut i = 0;
            while i < parts.len() {
                match parts[i].as_str() {
                    "--agents" if i + 1 < parts.len() => {
                        agents = parts[i + 1].parse().unwrap_or(0);
                        i += 2;
                    }
                    "--cmd" if i + 1 < parts.len() => {
                        cmd = parts[i + 1].clone();
                        cmd_explicit = true;
                        i += 2;
                    }
                    "--ade" if i + 1 < parts.len() => {
                        ade = Some(parts[i + 1].clone());
                        i += 2;
                    }
                    "--keep-worktrees" => {
                        keep_worktrees = true;
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
            if let Some(ref id) = ade {
                if !cmd_explicit {
                    if let Some(spec) = pytxo_core::resolve_ade(id) {
                        cmd = spec.default_cmd.to_string();
                    }
                }
            }
            SlashCommand::Run {
                agents,
                cmd,
                keep_worktrees,
                ade,
            }
        }
        "logs" => {
            let agent = parts.first().cloned().unwrap_or_else(|| "agent-0".into());
            let mut tail = 40usize;
            for chunk in parts[1..].chunks(2) {
                if chunk.len() == 2 && chunk[0] == "--tail" {
                    tail = chunk[1].parse().unwrap_or(tail);
                }
            }
            SlashCommand::Logs { agent, tail }
        }
        "stop" => SlashCommand::Stop {
            all: parts.iter().any(|p| p == "--all"),
        },
        "status" => {
            let mut limit = 10usize;
            for chunk in parts.chunks(2) {
                if chunk.len() == 2 && chunk[0] == "--limit" {
                    limit = chunk[1].parse().unwrap_or(limit);
                }
            }
            SlashCommand::Status { limit }
        }
        "trust" => {
            let tier = parts.first().cloned();
            SlashCommand::Trust { tier }
        }
        "models" => SlashCommand::Models {
            sub: parse_models_sub(&parts),
        },
        "agents" => SlashCommand::Agents,
        "use" => {
            let ade = parts.first().cloned().unwrap_or_default();
            SlashCommand::Use { ade }
        }
        _ => SlashCommand::Help,
    }
}

fn parse_models_sub(parts: &[String]) -> ModelsSub {
    let sub = parts
        .first()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    let mut provider = None;
    let mut refresh = false;
    for chunk in parts[1..].chunks(2) {
        if chunk.len() == 2 && chunk[0] == "--provider" {
            provider = Some(chunk[1].clone());
        }
    }
    if parts.iter().any(|p| p == "--refresh") {
        refresh = true;
    }
    match sub.as_str() {
        "search" => ModelsSub::Search {
            query: parts.get(1).cloned().unwrap_or_default(),
            provider,
        },
        "refresh" => ModelsSub::Refresh { provider },
        _ => ModelsSub::List { provider, refresh },
    }
}

fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote: Option<char> = None;
    for ch in s.chars() {
        match (in_quote, ch) {
            (Some(q), c) if c == q => in_quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => in_quote = Some(ch),
            (None, ' ') | (None, '\t') if !cur.is_empty() => {
                out.push(std::mem::take(&mut cur));
            }
            (None, ' ') | (None, '\t') => {}
            (None, c) => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Tab-completion for the operator prompt (prefix extension).
pub fn complete_line(buffer: &str) -> Option<String> {
    let trimmed = buffer.trim_start();
    if !trimmed.starts_with('/') {
        return None;
    }
    let rest = trimmed.strip_prefix('/').unwrap_or(trimmed);
    if !rest.contains(' ') {
        let prefix = rest.to_ascii_lowercase();
        let cmds = [
            "help", "doctor", "dry-run", "run", "status", "logs", "stop", "trust", "models",
            "agents", "use",
        ];
        let matches: Vec<&str> = cmds
            .iter()
            .copied()
            .filter(|c| c.starts_with(&prefix))
            .collect();
        return extend_unique(&matches, buffer, '/');
    }
    let parts = split_args(rest);
    if parts.is_empty() {
        return None;
    }
    let head = parts[0].to_ascii_lowercase();
    match head.as_str() {
        "models" => {
            if parts.len() == 1 {
                return Some(format!("{buffer} list"));
            }
            if parts.len() == 2 && !parts[1].contains('-') {
                let sub = parts[1].to_ascii_lowercase();
                for candidate in ["list", "search", "refresh"] {
                    if candidate.starts_with(&sub) {
                        let base = buffer.trim_end();
                        let without_sub = base.strip_suffix(&parts[1]).unwrap_or(base);
                        return Some(format!("{without_sub}{candidate} "));
                    }
                }
            }
        }
        "trust" => {
            if parts.len() == 1 {
                return Some(format!("{buffer} orbit"));
            }
            if parts.len() == 2 {
                let sub = parts[1].to_ascii_lowercase();
                for tier in ["orbit", "galaxy", "deep_space", "supernova"] {
                    if tier.starts_with(&sub) {
                        let base = buffer.trim_end();
                        let without = base.strip_suffix(&parts[1]).unwrap_or(base);
                        return Some(format!("{without}{tier}"));
                    }
                }
            }
        }
        "use" => {
            if parts.len() == 1 {
                return Some(format!("{buffer} cursor"));
            }
            if parts.len() == 2 {
                let sub = parts[1].to_ascii_lowercase();
                for spec in pytxo_core::all_ade_clis() {
                    if spec.id.starts_with(&sub) {
                        let base = buffer.trim_end();
                        let without = base.strip_suffix(&parts[1]).unwrap_or(base);
                        return Some(format!("{without}{} ", spec.id));
                    }
                }
            }
        }
        "run" => {
            if parts.len() == 1 {
                return Some(format!("{buffer} --ade "));
            }
            if parts.len() >= 2 && parts[1] == "--ade" && parts.len() == 2 {
                return Some(format!("{buffer} cursor"));
            }
        }
        _ => {}
    }
    None
}

fn extend_unique(matches: &[&str], buffer: &str, slash: char) -> Option<String> {
    if matches.is_empty() {
        return None;
    }
    if matches.len() == 1 {
        let trail = if buffer.ends_with(' ') { "" } else { " " };
        return Some(format!("/{}{trail}", matches[0]));
    }
    let mut prefix = matches[0].to_string();
    for m in &matches[1..] {
        while !m.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                break;
            }
        }
    }
    if prefix.len() > buffer.trim_start().strip_prefix('/').unwrap_or("").len() {
        Some(format!("{slash}{prefix}"))
    } else {
        None
    }
}

pub fn help_text() -> String {
    [
        "Hypervisor Shell — plan and run your coding agents from the terminal",
        "",
        "Type a request in plain words to plan it, then /run to start the agents.",
        "Review and Apply the result in Pytxo Desktop.",
        "",
        "  /help              Show this help",
        "  /doctor            Check this machine and project",
        "  /dry-run           Preview the plan's steps (optional --agents N)",
        "  /agents            List agent CLIs found on PATH (codex, claude, cursor, …)",
        "  /use <agent>       Set the agent /run starts",
        "  /run               Start the plan (--agents N --cmd \"…\" | --ade <agent>)",
        "  /status            Recent runs (--limit N)",
        "  /logs <agent>      Recent output from one agent (--tail N)",
        "  /stop [--all]      Stop the active run, or all agent processes",
        "  /trust [profile]   Trust this folder (orbit|galaxy|deep_space|supernova)",
        "  /models list       List models (--provider deepseek|openrouter|...)",
        "  /models search Q   Search models",
        "",
        "exit / quit          Leave the shell",
        "",
        "Discord: https://discord.gg/AUFRPFjSYv",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_run_with_quoted_cmd() {
        let input = parse_line(r#"/run --agents 2 --cmd "echo hello world""#);
        match input {
            ShellInput::Slash(SlashCommand::Run { agents, cmd, .. }) => {
                assert_eq!(agents, 2);
                assert_eq!(cmd, "echo hello world");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn run_ade_resolves_default_cmd() {
        let input = parse_line("/run --ade cursor");
        match input {
            ShellInput::Slash(SlashCommand::Run { cmd, ade, .. }) => {
                assert_eq!(ade.as_deref(), Some("cursor"));
                assert_eq!(cmd, "cursor-agent -p --trust --output-format text");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn plan_alias_maps_to_dry_run() {
        let input = parse_line("/plan dry-run --agents 3");
        match input {
            ShellInput::Slash(SlashCommand::DryRun { agents }) => assert_eq!(agents, 3),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn mission_line_when_not_slash() {
        let input = parse_line("fix auth tests");
        assert!(matches!(input, ShellInput::Mission(_)));
    }
}
