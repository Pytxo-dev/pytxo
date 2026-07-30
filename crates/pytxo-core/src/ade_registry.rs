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
        default_cmd: "claude -p",
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
        default_cmd: "codex exec --sandbox workspace-write",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "cursor",
        display_name: "Cursor Agent",
        probe_bin: "cursor-agent",
        default_cmd: "cursor-agent -p --trust",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "opencode",
        display_name: "OpenCode",
        probe_bin: "opencode",
        default_cmd: "opencode run",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "gemini",
        display_name: "Gemini CLI",
        probe_bin: "gemini",
        default_cmd: "gemini --skip-trust -p",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "copilot",
        display_name: "GitHub Copilot CLI",
        probe_bin: "copilot",
        default_cmd: "copilot -p",
        cli_adapter: CliAdapter::Generic,
    },
    AdeCliSpec {
        id: "aider",
        display_name: "Aider",
        probe_bin: "aider",
        default_cmd: "aider --message",
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
                "gemini_cli" => s.id == "gemini",
                "github_copilot" | "copilot_cli" => s.id == "copilot",
                other => s.id == other,
            }
    })
}

pub fn ade_on_path(spec: &AdeCliSpec) -> bool {
    which_binary(spec.probe_bin)
}

pub fn detect_on_path() -> Vec<(&'static str, bool)> {
    REGISTRY.iter().map(|s| (s.id, ade_on_path(s))).collect()
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
    // Prefer PATH walk — no console flash from GUI parents (Desktop).
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            #[cfg(windows)]
            {
                if ["exe", "com", "cmd", "bat"]
                    .iter()
                    .any(|extension| dir.join(format!("{name}.{extension}")).is_file())
                    || dir.join(name).is_file()
                {
                    return true;
                }
            }
            #[cfg(not(windows))]
            {
                if dir.join(name).is_file() {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_aliases() {
        assert!(resolve_ade("claude_code").is_some());
        assert!(resolve_ade("cursor").is_some());
        assert!(resolve_ade("gemini_cli").is_some());
        assert!(resolve_ade("github_copilot").is_some());
        assert!(resolve_ade("unknown-xyz").is_none());
    }

    #[test]
    fn flow_defaults_are_headless_prompt_commands() {
        let expected = [
            ("claude", "claude -p"),
            ("codex", "codex exec --sandbox workspace-write"),
            ("cursor", "cursor-agent -p --trust"),
            ("opencode", "opencode run"),
            ("gemini", "gemini --skip-trust -p"),
            ("copilot", "copilot -p"),
            ("aider", "aider --message"),
        ];
        for (id, command) in expected {
            assert_eq!(resolve_ade(id).map(|spec| spec.default_cmd), Some(command));
        }
    }
}
