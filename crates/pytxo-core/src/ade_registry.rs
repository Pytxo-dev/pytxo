use crate::billing::CliAdapter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdeAuthPolicy {
    /// Pytxo can ask the vendor CLI for a redacted, boolean session status.
    VerifiedSession,
    /// The CLI owns authentication, but does not expose a supported status probe.
    VendorManaged,
    /// Credentials are supplied by the explicitly selected Pytxo provider route.
    ProviderManaged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdeDispatchPolicy {
    /// Pytxo has a reviewed non-interactive command for this harness.
    Ready,
    /// The executable can be detected, but its permission model is not mapped yet.
    DetectionOnly,
}

#[derive(Clone, Copy, Debug)]
pub struct AdeCliSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub probe_bin: &'static str,
    pub default_cmd: &'static str,
    pub cli_adapter: CliAdapter,
    pub auth_policy: AdeAuthPolicy,
    pub dispatch_policy: AdeDispatchPolicy,
    pub auth_owner: &'static str,
    pub docs_url: &'static str,
    pub detail: &'static str,
}

const REGISTRY: &[AdeCliSpec] = &[
    AdeCliSpec {
        id: "claude",
        display_name: "Claude Code",
        probe_bin: "claude",
        default_cmd: "claude -p",
        cli_adapter: CliAdapter::ClaudeCode,
        auth_policy: AdeAuthPolicy::VerifiedSession,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Claude Code",
        docs_url: "https://code.claude.com/docs/en/authentication",
        detail: "Pytxo opens Claude Code's official sign-in and never receives its token.",
    },
    AdeCliSpec {
        id: "agy",
        display_name: "Antigravity",
        probe_bin: "agy",
        default_cmd: "agy",
        cli_adapter: CliAdapter::Antigravity,
        auth_policy: AdeAuthPolicy::VendorManaged,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Antigravity",
        docs_url: "https://antigravity.google/docs/cli/headless/",
        detail: "Authentication and model access stay inside Antigravity.",
    },
    AdeCliSpec {
        id: "codex",
        display_name: "OpenAI Codex",
        probe_bin: "codex",
        default_cmd: "codex exec --sandbox workspace-write",
        cli_adapter: CliAdapter::Generic,
        auth_policy: AdeAuthPolicy::VerifiedSession,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Codex",
        docs_url: "https://developers.openai.com/codex/auth",
        detail: "Codex owns the browser session, token storage, and refresh.",
    },
    AdeCliSpec {
        id: "cursor",
        display_name: "Cursor Agent",
        probe_bin: "cursor-agent",
        default_cmd: "cursor-agent -p --trust",
        cli_adapter: CliAdapter::Generic,
        auth_policy: AdeAuthPolicy::VerifiedSession,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Cursor Agent",
        docs_url: "https://docs.cursor.com/en/cli/reference/authentication",
        detail: "Cursor Agent keeps its account credential outside Pytxo.",
    },
    AdeCliSpec {
        id: "opencode",
        display_name: "OpenCode",
        probe_bin: "opencode",
        default_cmd: "opencode run",
        cli_adapter: CliAdapter::Generic,
        auth_policy: AdeAuthPolicy::VerifiedSession,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "OpenCode",
        docs_url: "https://opencode.ai/docs/providers/",
        detail: "Provider-specific credentials remain owned by OpenCode.",
    },
    AdeCliSpec {
        id: "gemini",
        display_name: "Gemini CLI",
        probe_bin: "gemini",
        default_cmd: "gemini --skip-trust -p",
        cli_adapter: CliAdapter::Generic,
        auth_policy: AdeAuthPolicy::VendorManaged,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Gemini CLI",
        docs_url: "https://geminicli.com/docs/get-started/authentication/",
        detail: "Gemini CLI owns Google OAuth; Pytxo does not reuse its cached token.",
    },
    AdeCliSpec {
        id: "copilot",
        display_name: "GitHub Copilot CLI",
        probe_bin: "copilot",
        default_cmd: "copilot -p",
        cli_adapter: CliAdapter::Generic,
        auth_policy: AdeAuthPolicy::VendorManaged,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Copilot CLI",
        docs_url: "https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/authenticate-copilot-cli",
        detail: "Copilot CLI owns the GitHub device flow and stores its token in the OS keychain.",
    },
    AdeCliSpec {
        id: "aider",
        display_name: "Aider",
        probe_bin: "aider",
        default_cmd: "aider --message",
        cli_adapter: CliAdapter::Generic,
        auth_policy: AdeAuthPolicy::ProviderManaged,
        dispatch_policy: AdeDispatchPolicy::Ready,
        auth_owner: "Pytxo run policy",
        docs_url: "https://aider.chat/docs/config/api-keys.html",
        detail: "Choose one explicit BYOK credential for the run; unrelated keys stay hidden.",
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

pub fn ade_can_dispatch(spec: &AdeCliSpec) -> bool {
    spec.dispatch_policy == AdeDispatchPolicy::Ready
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
        let mark = if ade_on_path(spec) && ade_can_dispatch(spec) {
            "✓"
        } else if ade_on_path(spec) {
            "!"
        } else {
            "·"
        };
        let policy = if ade_can_dispatch(spec) {
            ""
        } else {
            " [detection only]"
        };
        lines.push(format!(
            "  {mark} {:<10} {:<16} cmd: {}{}",
            spec.id, spec.display_name, spec.default_cmd, policy
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

    #[test]
    fn registered_harnesses_are_dispatch_ready() {
        assert!(all_ade_clis().iter().all(ade_can_dispatch));
    }
}
