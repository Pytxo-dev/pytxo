//! Galaxy HITL gates for high-risk agent actions ([[permission-profile-engine]], ADR-0016/0018).

use std::path::Path;
use std::time::Duration;

use pytxo_core::{PermissionProfile, PytxoError, Result};

use crate::hitl::{HitlDecision, HitlQueue};

const HITL_GATE_TIMEOUT: Duration = Duration::from_secs(300);

/// Classify a spawn command for Galaxy HITL review.
pub fn classify_risky_command(cmd: &str) -> Option<(&'static str, &'static str)> {
    let lower = cmd.to_ascii_lowercase();
    if lower.contains("git rebase")
        || lower.contains("git reset --hard")
        || lower.contains("git push --force")
        || lower.contains("git push -f")
    {
        return Some(("git.destructive", "destructive git operation detected"));
    }
    if lower.contains("kubectl ") || lower.contains("terraform apply") {
        return Some(("proc.infrastructure", "infrastructure mutation detected"));
    }
    if lower.contains("chmod ") || lower.contains("chown ") {
        return Some(("fs.permission", "permission change detected in agent command"));
    }
    if lower.contains("rm -rf")
        || lower.contains("rm -fr")
        || lower.contains("rm -r ")
        || lower.contains("del /s")
        || lower.contains("rd /s")
    {
        return Some(("fs.delete", "recursive delete detected in agent command"));
    }
    if lower.contains("git push") {
        return Some(("git.push", "git push detected in agent command"));
    }
    if lower.contains("docker ")
        || lower.starts_with("docker")
        || lower.contains("podman ")
        || lower.contains("docker compose")
    {
        return Some(("proc.docker", "container runtime invocation detected"));
    }
    if lower.contains("curl ")
        || lower.contains("wget ")
        || lower.contains("invoke-webrequest")
        || lower.contains("iwr ")
    {
        return Some(("net.egress", "network fetch detected in agent command"));
    }
    if lower.contains("nc ") || lower.contains("ncat ") {
        return Some(("net.bind", "network tool detected in agent command"));
    }
    if classify_package_install(&lower) {
        return Some((
            "proc.package_install",
            "package install detected in agent command",
        ));
    }
    None
}

fn classify_package_install(lower: &str) -> bool {
    lower.contains("npm install")
        || lower.contains("npm i ")
        || lower.contains("yarn add")
        || lower.contains("pnpm add")
        || lower.contains("pip install")
        || lower.contains("cargo install")
        || lower.contains("brew install")
        || lower.contains("apt-get install")
        || lower.contains("apt install")
}

/// Classify an MCP proxy JSON-RPC method for Galaxy HITL review (ADR-0018).
pub fn classify_mcp_proxy(method: &str, params: &serde_json::Value) -> Option<(&'static str, String)> {
    match method {
        "tools/call" => {
            let tool = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown>");
            Some((
                "mcp.tool",
                format!("MCP tools/call: {tool}"),
            ))
        }
        "resources/read" | "prompts/get" | "resources/subscribe" => Some((
            "mcp.tool",
            format!("MCP {method}"),
        )),
        _ => None,
    }
}

/// True when a workspace flush would materialize outside `repo_root`.
pub fn workspace_writes_outside_root(workspace_cwd: &Path, repo_root: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let repo = canon(repo_root);
    let cwd = canon(workspace_cwd);
    !cwd.starts_with(&repo)
}

/// Block until a human approves or denies a classified action (Galaxy + queue).
pub fn gate_hitl_action(
    hitl: Option<&HitlQueue>,
    profile: PermissionProfile,
    agent_key: &str,
    action: &str,
    reason: &str,
) -> Result<()> {
    if profile != PermissionProfile::Galaxy {
        return Ok(());
    }
    let Some(hitl) = hitl else {
        return Err(PytxoError::Runner(format!(
            "Galaxy profile blocked action ({action}): no HITL queue configured"
        )));
    };
    let id = hitl.submit(agent_key, action, reason);
    match hitl.wait_blocking(&id, HITL_GATE_TIMEOUT) {
        HitlDecision::Approved => Ok(()),
        HitlDecision::Denied => Err(PytxoError::Runner(format!(
            "action denied by human reviewer ({action})"
        ))),
        HitlDecision::Pending => Err(PytxoError::Runner(format!(
            "action approval timed out ({action})"
        ))),
    }
}

/// When profile is Galaxy and `hitl` is set, block until a human approves risky spawns.
pub fn gate_spawn_command(
    hitl: Option<&HitlQueue>,
    profile: PermissionProfile,
    agent_key: &str,
    cmd: &str,
) -> Result<()> {
    let Some((action, reason)) = classify_risky_command(cmd) else {
        return Ok(());
    };
    gate_hitl_action(hitl, profile, agent_key, action, reason)
}

/// Galaxy MCP proxy gate ([[ADR-0018-network-and-mcp-policy]]).
pub fn gate_mcp_proxy(
    hitl: Option<&HitlQueue>,
    profile: PermissionProfile,
    agent_key: &str,
    method: &str,
    params: &serde_json::Value,
) -> Result<()> {
    let Some((action, reason)) = classify_mcp_proxy(method, params) else {
        return Ok(());
    };
    gate_hitl_action(hitl, profile, agent_key, action, &reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_destructive_commands() {
        assert!(classify_risky_command("rm -rf /tmp/x").is_some());
        assert!(classify_risky_command("git push origin main").is_some());
        assert!(classify_risky_command("npm install lodash").is_some());
        assert!(classify_risky_command("echo safe").is_none());
    }

    #[test]
    fn classifies_git_destructive() {
        let (action, _) = classify_risky_command("git rebase origin/main").unwrap();
        assert_eq!(action, "git.destructive");
        let (action, _) = classify_risky_command("git reset --hard HEAD~1").unwrap();
        assert_eq!(action, "git.destructive");
        let (action, _) = classify_risky_command("git push --force origin main").unwrap();
        assert_eq!(action, "git.destructive");
        let (action, _) = classify_risky_command("git push -f origin main").unwrap();
        assert_eq!(action, "git.destructive");
    }

    #[test]
    fn classifies_infrastructure_and_permissions() {
        let (action, _) = classify_risky_command("kubectl apply -f deploy.yaml").unwrap();
        assert_eq!(action, "proc.infrastructure");
        let (action, _) = classify_risky_command("terraform apply -auto-approve").unwrap();
        assert_eq!(action, "proc.infrastructure");
        let (action, _) = classify_risky_command("chmod 777 /tmp/x").unwrap();
        assert_eq!(action, "fs.permission");
        let (action, _) = classify_risky_command("chown root:root /etc/x").unwrap();
        assert_eq!(action, "fs.permission");
    }

    #[test]
    fn classifies_docker_net_and_packages() {
        let (action, _) = classify_risky_command("docker compose up -d").unwrap();
        assert_eq!(action, "proc.docker");
        let (action, _) = classify_risky_command("curl https://example.com").unwrap();
        assert_eq!(action, "net.egress");
        let (action, _) = classify_risky_command("nc -l 8080").unwrap();
        assert_eq!(action, "net.bind");
        let (action, _) = classify_risky_command("pip install requests").unwrap();
        assert_eq!(action, "proc.package_install");
        let (action, _) = classify_risky_command("cargo install ripgrep").unwrap();
        assert_eq!(action, "proc.package_install");
    }

    #[test]
    fn classifies_mcp_tool_calls() {
        let params = serde_json::json!({ "name": "bash" });
        assert!(classify_mcp_proxy("tools/call", &params).is_some());
        assert!(classify_mcp_proxy("resources/subscribe", &serde_json::json!({})).is_some());
        assert!(classify_mcp_proxy("initialize", &serde_json::json!({})).is_none());
    }
}
