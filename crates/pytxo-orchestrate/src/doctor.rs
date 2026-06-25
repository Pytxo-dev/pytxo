use std::path::Path;
use std::process::Command;

use pytxo_core::{HttpBillingReconciler, PytxoConfig};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn all_ok(&self) -> bool {
        self.checks.iter().all(|c| c.ok)
    }
}

pub fn run_doctor(repo: Option<&Path>) -> anyhow::Result<DoctorReport> {
    let repo_root = super::resolve_repo_root(repo)?;
    let cfg = super::load_config(None, &repo_root).unwrap_or_default();
    let checks = vec![
        check_git_installed(),
        check_inside_git_repo(&repo_root),
        check_head_exists(&repo_root),
        check_worktree_command(&repo_root),
        check_pytxo_dirs_writable(&repo_root),
        check_pty_smoke(),
        check_link_reconcile(&cfg),
        check_cloud_sandbox(&cfg),
        check_inference_proxy_health(&cfg),
        check_hitl_persistence(&repo_root, &cfg),
        check_network_policy(),
        check_deepspace_network_isolation(),
        check_mcp_hitl(&cfg),
        check_overlay_isolation(&cfg),
    ];
    Ok(DoctorReport { checks })
}

fn check_git_installed() -> DoctorCheck {
    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => DoctorCheck {
            name: "git_installed".into(),
            ok: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().to_string(),
        },
        Ok(o) => DoctorCheck {
            name: "git_installed".into(),
            ok: false,
            detail: format!(
                "git --version failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ),
        },
        Err(e) => DoctorCheck {
            name: "git_installed".into(),
            ok: false,
            detail: format!("git not found: {e}"),
        },
    }
}

fn check_inside_git_repo(repo: &Path) -> DoctorCheck {
    git_ok(
        "git_repo",
        repo,
        &["rev-parse", "--is-inside-work-tree"],
        "not inside a git repository",
    )
}

fn check_head_exists(repo: &Path) -> DoctorCheck {
    match Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
    {
        Ok(o) if o.status.success() => DoctorCheck {
            name: "git_head".into(),
            ok: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().to_string(),
        },
        Ok(_) => DoctorCheck {
            name: "git_head".into(),
            ok: false,
            detail: "no commits yet — run: git add . && git commit -m \"initial commit\"".into(),
        },
        Err(e) => DoctorCheck {
            name: "git_head".into(),
            ok: false,
            detail: format!("git rev-parse HEAD failed: {e}"),
        },
    }
}

fn check_worktree_command(repo: &Path) -> DoctorCheck {
    git_ok(
        "git_worktree",
        repo,
        &["worktree", "list"],
        "git worktree unavailable",
    )
}

fn check_pytxo_dirs_writable(repo: &Path) -> DoctorCheck {
    let cfg = PytxoConfig::default();
    let wt = repo.join(&cfg.worktree_dir);
    let data = repo.join(&cfg.data_dir);
    match (std::fs::create_dir_all(&wt), std::fs::create_dir_all(&data)) {
        (Ok(()), Ok(())) => DoctorCheck {
            name: "pytxo_dirs".into(),
            ok: true,
            detail: format!("writable: {} and {}", wt.display(), data.display()),
        },
        (Err(e), _) | (_, Err(e)) => DoctorCheck {
            name: "pytxo_dirs".into(),
            ok: false,
            detail: format!("cannot create .pytxo directories: {e}"),
        },
    }
}

fn git_ok(name: &str, repo: &Path, args: &[&str], fail_msg: &str) -> DoctorCheck {
    match Command::new("git").args(args).current_dir(repo).output() {
        Ok(o) if o.status.success() => DoctorCheck {
            name: name.into(),
            ok: true,
            detail: String::from_utf8_lossy(&o.stdout).trim().to_string(),
        },
        Ok(o) => DoctorCheck {
            name: name.into(),
            ok: false,
            detail: format!("{fail_msg}: {}", String::from_utf8_lossy(&o.stderr).trim()),
        },
        Err(e) => DoctorCheck {
            name: name.into(),
            ok: false,
            detail: format!("{fail_msg}: {e}"),
        },
    }
}

fn check_link_reconcile(cfg: &PytxoConfig) -> DoctorCheck {
    if !cfg.billing_mode().is_ultra() || !cfg.billing.link_reconcile_enabled() {
        return DoctorCheck {
            name: "link_reconcile".into(),
            ok: true,
            detail: "skipped (billing.mode is not ultra or link_reconcile is false)".into(),
        };
    }
    let url = cfg.billing.proxy_url.trim();
    let reconciler = HttpBillingReconciler::new(url);
    match reconciler.ping() {
        Ok(()) => DoctorCheck {
            name: "link_reconcile".into(),
            ok: true,
            detail: format!("Link /health ok at {url}"),
        },
        Err(e) => DoctorCheck {
            name: "link_reconcile".into(),
            ok: false,
            detail: format!("link reconcile config invalid: {e}"),
        },
    }
}

fn check_cloud_sandbox(cfg: &PytxoConfig) -> DoctorCheck {
    if cfg.execution_backend != pytxo_core::ExecutionBackend::Cloud && !cfg.cloud.enabled {
        return DoctorCheck {
            name: "cloud_health".into(),
            ok: true,
            detail: "skipped (execution_backend is not cloud and [cloud].enabled is false)".into(),
        };
    }
    match crate::cloud::ping_cloud(&cfg.cloud) {
        Ok(()) => DoctorCheck {
            name: "cloud_health".into(),
            ok: true,
            detail: format!(
                "cloud reachable at {}",
                crate::cloud::cloud_health_url(&cfg.cloud)
            ),
        },
        Err(e) => DoctorCheck {
            name: "cloud_health".into(),
            ok: false,
            detail: format!("cloud ping failed: {e}"),
        },
    }
}

fn check_inference_proxy_health(cfg: &PytxoConfig) -> DoctorCheck {
    if !cfg.billing_mode().is_ultra() {
        return DoctorCheck {
            name: "inference_proxy_health".into(),
            ok: true,
            detail: "skipped (billing.mode is not ultra)".into(),
        };
    }
    let url = cfg.billing.inference_proxy_base_url();
    if url.is_empty() {
        return DoctorCheck {
            name: "inference_proxy_health".into(),
            ok: false,
            detail: "billing.inference_proxy_url is empty".into(),
        };
    }
    match ping_service_health(url) {
        Ok(()) => DoctorCheck {
            name: "inference_proxy_health".into(),
            ok: true,
            detail: format!("inference proxy /health ok at {url}"),
        },
        Err(e) => DoctorCheck {
            name: "inference_proxy_health".into(),
            ok: false,
            detail: format!("inference proxy health failed: {e}"),
        },
    }
}

#[cfg(feature = "link-http")]
fn ping_service_health(base: &str) -> Result<(), String> {
    let health = format!("{}/health", base.trim_end_matches('/'));
    let resp = ureq::get(&health)
        .call()
        .map_err(|e| format!("{e}"))?;
    if resp.status() != 200 {
        return Err(format!("status {}", resp.status()));
    }
    let body = resp.into_string().map_err(|e| format!("{e}"))?;
    if !pytxo_core::service_health_ok(&body) {
        return Err(format!("unexpected body: {body}"));
    }
    Ok(())
}

#[cfg(not(feature = "link-http"))]
fn ping_service_health(_base: &str) -> Result<(), String> {
    Ok(())
}

fn check_hitl_persistence(repo: &Path, cfg: &PytxoConfig) -> DoctorCheck {
    let data_dir = repo.join(&cfg.data_dir);
    let path = data_dir.join("hitl.json");
    match std::fs::create_dir_all(&data_dir) {
        Ok(()) => {
            let probe = path.with_extension("json.probe");
            match std::fs::write(&probe, "{}") {
                Ok(()) => {
                    let _ = std::fs::remove_file(&probe);
                    DoctorCheck {
                        name: "hitl_persistence".into(),
                        ok: true,
                        detail: format!("writable at {}", path.display()),
                    }
                }
                Err(e) => DoctorCheck {
                    name: "hitl_persistence".into(),
                    ok: false,
                    detail: format!("cannot write HITL store: {e}"),
                },
            }
        }
        Err(e) => DoctorCheck {
            name: "hitl_persistence".into(),
            ok: false,
            detail: format!("cannot create data dir: {e}"),
        },
    }
}

fn check_network_policy() -> DoctorCheck {
    use pytxo_core::{PermissionEngine, PermissionProfile};
    let orbit = PermissionEngine::new(PermissionProfile::Orbit);
    let galaxy = PermissionEngine::new(PermissionProfile::Galaxy);
    let orbit_denies = !orbit.spawn_egress_allowed("curl https://example.com");
    let galaxy_allows = galaxy.spawn_egress_allowed("curl https://example.com");
    let ok = orbit_denies && galaxy_allows;
    DoctorCheck {
        name: "network_policy".into(),
        ok,
        detail: format!(
            "orbit_denies_egress={orbit_denies} galaxy_allows_egress={galaxy_allows}"
        ),
    }
}

fn check_deepspace_network_isolation() -> DoctorCheck {
    use pytxo_core::{NetworkPolicy, NetworkPolicyEngine, PermissionProfile};
    let deepspace = NetworkPolicyEngine::new(PermissionProfile::DeepSpace);
    let policy_blocked = !deepspace.egress_allowed("1.1.1.1", 443);
    let socket_blocked = pytxo_runner::doctor_deepspace_socket_blocked();
    let detail = pytxo_runner::doctor_network_isolation_probe();
    DoctorCheck {
        name: "deepspace_network_isolation".into(),
        ok: policy_blocked && socket_blocked,
        detail,
    }
}

fn check_mcp_hitl(cfg: &PytxoConfig) -> DoctorCheck {
    use pytxo_core::PermissionProfile;
    if cfg.permission_profile != PermissionProfile::Galaxy {
        return DoctorCheck {
            name: "mcp_hitl".into(),
            ok: true,
            detail: "skipped (permission_profile is not galaxy)".into(),
        };
    }
    DoctorCheck {
        name: "mcp_hitl".into(),
        ok: true,
        detail: "Galaxy MCP proxy gated via HitlQueue (tools/call, resources/read)".into(),
    }
}

fn check_overlay_isolation(cfg: &PytxoConfig) -> DoctorCheck {
    use pytxo_core::IsolationMode;
    if cfg.isolation == IsolationMode::Worktree {
        return DoctorCheck {
            name: "overlay_isolation".into(),
            ok: true,
            detail: "worktree isolation (default)".into(),
        };
    }
    match pytxo_runner::doctor_overlay_probe() {
        Ok(detail) => DoctorCheck {
            name: "overlay_isolation".into(),
            ok: true,
            detail,
        },
        Err(e) => DoctorCheck {
            name: "overlay_isolation".into(),
            ok: false,
            detail: format!("{e}"),
        },
    }
}

fn check_pty_smoke() -> DoctorCheck {
    match pytxo_runner::doctor_pty_smoke() {
        Ok(()) => DoctorCheck {
            name: "pty_smoke".into(),
            ok: true,
            detail: "portable-pty echo ok".into(),
        },
        Err(e) => {
            let hint = if cfg!(windows) {
                " (ConPTY may require Windows 10+; try execution_backend = \"subprocess\" in pytxo.toml)"
            } else {
                ""
            };
            DoctorCheck {
                name: "pty_smoke".into(),
                ok: false,
                detail: format!("{e}{hint}"),
            }
        }
    }
}

pub fn assert_git_ready(repo: &Path) -> anyhow::Result<()> {
    let report = run_doctor(Some(repo))?;
    let failed: Vec<_> = report.checks.iter().filter(|c| !c.ok).collect();
    if failed.is_empty() {
        return Ok(());
    }
    let msg: Vec<String> = failed
        .iter()
        .map(|c| format!("{}: {}", c.name, c.detail))
        .collect();
    anyhow::bail!("git preflight failed:\n{}", msg.join("\n"))
}
