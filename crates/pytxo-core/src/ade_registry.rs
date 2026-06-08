use crate::billing::CliAdapter;

#[derive(Clone, Copy, Debug)]
pub struct AdeCliSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub probe_bin: &'static str,
    pub default_cmd: &'static str,
    pub cli_adapter: CliAdapter,
}

const REGISTRY: &[AdeCliSpec] = &[
    AdeCliSpec {
        id: "claude",
        display_name: "Claude Code",
        probe_bin: "claude",
        default_cmd: "claude",
        cli_adapter: CliAdapter::ClaudeCode,
    },
    AdeCliSpec {
        id: "agy",
        display_name: "Antigravity",
        probe_bin: "agy",
        default_cmd: "agy",
        cli_adapter: CliAdapter::Antigravity,
    },
    AdeCliSpec {
        id: "codex",
        display_name: "OpenAI Codex",
        probe_bin: "codex",
        default_cmd: "codex",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "cursor",
        display_name: "Cursor Agent",
        probe_bin: "cursor",
        default_cmd: "cursor agent",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "opencode",
        display_name: "OpenCode",
        probe_bin: "opencode",
        default_cmd: "opencode",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "aider",
        display_name: "Aider",
        probe_bin: "aider",
        default_cmd: "aider",
        cli_adapter: CliAdapter::Generic,
    },
];

pub fn all_ade_clis() -> &'static [AdeCliSpec] {
    REGISTRY
}

pub fn resolve_ade(id: &str) -> Option<&'static AdeCliSpec> {
    let key = id.to_ascii_lowercase();
    REGISTRY.iter().find(|s| {
        s.id == key.as_str()
            || match key.as_str() {
                "claude_code" | "claude" => s.id == "claude",
                "antigravity" | "agy" => s.id == "agy",
                "openai_codex" => s.id == "codex",
                "cursor_agent" => s.id == "cursor",
                other => s.id == other,
            }
    })
}

pub fn ade_on_path(spec: &AdeCliSpec) -> bool {
    which_binary(spec.probe_bin)
}

pub fn detect_on_path() -> Vec<(&'static str, bool)> {
    REGISTRY
        .iter()
        .map(|s| (s.id, ade_on_path(s)))
        .collect()
}

pub fn format_agents_list() -> String {
    let mut lines = vec!["ADE CLIs (terminal agents Pytxo can spawn):".to_string()];
    for spec in REGISTRY {
        let mark = if ade_on_path(spec) { "✓" } else { "·" };
        lines.push(format!(
            "  {mark} {:<10} {:<16} cmd: {}",
            spec.id, spec.display_name, spec.default_cmd
        ));
    }
    lines.push(String::new());
    lines.push("  /use <id>     Set default /run command".into());
    lines.push("  /run --ade X  Dispatch with registry default cmd".into());
    lines.join("\n")
}

fn which_binary(name: &str) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("where")
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("sh")
            .args(["-c", &format!("command -v {name}")])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_aliases() {
        assert!(resolve_ade("claude_code").is_some());
        assert!(resolve_ade("cursor").is_some());
        assert!(resolve_ade("unknown-xyz").is_none());
    }
}
