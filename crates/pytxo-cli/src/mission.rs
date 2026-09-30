//! `pytxo mission` — one-command mission loop (ADR-0031 / Phase 77).

use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context};
use pytxo_core::{ade_can_dispatch, ade_on_path, all_ade_clis, PermissionProfile};
use pytxo_orchestrate::flow::{
    dispatch_flow, preview_flow, save_reviewed_flow_plan, FlowDraftInput, FlowPlan, FlowSource,
    FlowStatus,
};
use pytxo_orchestrate::{is_repo_trusted, resolve_repo_root, status_json, trust_repo};
use pytxo_store::Catalog;
use uuid::Uuid;

pub struct MissionOptions {
    pub text: String,
    pub repo: Option<PathBuf>,
    pub yes: bool,
    pub json: bool,
    pub ade: Option<String>,
    pub plan_file: Option<PathBuf>,
}

pub async fn run_mission(opts: MissionOptions) -> anyhow::Result<()> {
    let repo = resolve_repo_root(opts.repo.as_deref())?;
    if !is_repo_trusted(&repo)? {
        eprintln!(
            "Folder not trusted. Trusting at Orbit for this mission (isolated changes with reviewed Apply)."
        );
        trust_repo(&repo, PermissionProfile::Orbit)?;
    }

    let installed: Vec<_> = all_ade_clis()
        .iter()
        .filter(|s| ade_can_dispatch(s) && ade_on_path(s))
        .collect();
    if installed.is_empty() {
        bail!("No coding agent CLIs found on PATH. Install Claude Code, Codex, or OpenCode, then retry.");
    }
    eprintln!(
        "Agents found: {}",
        installed
            .iter()
            .map(|s| s.display_name)
            .collect::<Vec<_>>()
            .join(", ")
    );

    let catalog = Catalog::open_default().context("open pytxo catalog")?;
    let draft_id = Uuid::new_v4().to_string();
    let input = FlowDraftInput {
        id: draft_id.clone(),
        title: opts.text.chars().take(72).collect(),
        mission_text: opts.text.clone(),
        source: FlowSource::Text,
        domain_id: Some(repo.to_string_lossy().into_owned()),
        project_id: None,
        ade_id: opts.ade.clone(),
        max_workers: None,
        verification_commands: vec![],
    };

    let mut plan = preview_flow(&catalog, input)?;
    if let Some(path) = opts.plan_file.as_ref() {
        std::fs::write(path, serde_json::to_string_pretty(&plan)?)?;
        eprintln!("Plan written to {}", path.display());
    }

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    print_preflight(&plan);

    if plan.status != FlowStatus::Ready {
        bail!(
            "Mission blocked: {}",
            plan.blocked_reasons
                .iter()
                .map(|r| format!("{r:?}"))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }

    if !opts.yes {
        eprint!("Approve mission? [Y/n] ");
        let _ = io::stderr().flush();
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let ans = line.trim();
        if !ans.is_empty() && !ans.eq_ignore_ascii_case("y") && !ans.eq_ignore_ascii_case("yes") {
            bail!("Mission cancelled");
        }
    }

    plan = save_reviewed_flow_plan(&catalog, plan)?;
    let run_id = dispatch_flow(&catalog, &plan.draft_id)?;
    eprintln!("Dispatched run {run_id}");

    for _ in 0..3600 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let snap = status_json(None, Some(repo.clone()), 20)?;
        if let Some(run) = snap.runs.iter().find(|r| r.id == run_id) {
            let state = run.status.as_str();
            if matches!(
                state,
                "completed" | "failed" | "failed_startup" | "stopped" | "cancelled"
            ) {
                print_mission_report(&run_id, run, &plan);
                if state != "completed" {
                    bail!("Mission run {run_id} finished with status={state}");
                }
                eprintln!(
                    "Open Desktop > History > Run Review to inspect the run, its task checks, and any prepared changes before Apply.\nGalaxy approval queue: pytxo hitl list"
                );
                return Ok(());
            }
        }
    }
    bail!("Timed out waiting for run {run_id}");
}

fn print_preflight(plan: &FlowPlan) {
    eprintln!();
    eprintln!("Mission plan");
    eprintln!("------------");
    eprintln!(
        "Isolated workspace until reviewed Apply: {} via {}",
        plan.isolation_mode, plan.isolation_backend_intent
    );
    eprintln!(
        "Overlapping paths wait in later stages: {} stage(s)",
        plan.waves.len()
    );
    eprintln!("Tasks: {}", plan.tasks.len());
    eprintln!("Permission: {}", plan.permission_profile);
    if let Some(usd) = plan.estimated_cost_usd {
        eprintln!("Estimated model budget: ${usd:.2}");
    }
    for (i, wave) in plan.waves.iter().enumerate() {
        eprintln!("Stage {}", i + 1);
        for tid in wave {
            if let Some(task) = plan.tasks.iter().find(|t| &t.id == tid) {
                let verify = if task.verify.is_empty() {
                    "none".into()
                } else {
                    task.verify.join(", ")
                };
                eprintln!(
                    "  · {} — {} (paths: {}; verify: {})",
                    task.id,
                    task.prompt.chars().take(80).collect::<String>(),
                    if task.paths.is_empty() {
                        "·".into()
                    } else {
                        task.paths.join(", ")
                    },
                    verify
                );
            }
        }
    }
    for w in &plan.warnings {
        if w.code == "path_claim_overlap" {
            eprintln!("Overlapping paths wait (Race): {}", w.message);
        } else {
            eprintln!("Note [{}]: {}", w.code, w.message);
        }
    }
    eprintln!();
}

fn print_mission_report(run_id: &str, run: &pytxo_orchestrate::RunStatusJson, plan: &FlowPlan) {
    eprintln!();
    eprintln!("Mission report");
    eprintln!("--------------");
    eprintln!("Run: {run_id}");
    eprintln!("Status: {}", run.status);
    let accepted = run
        .agents
        .iter()
        .filter(|a| a.status == "completed")
        .count();
    eprintln!(
        "Tasks accepted: {}/{}",
        accepted,
        run.agents.len().max(plan.tasks.len())
    );
    let overlaps = plan
        .warnings
        .iter()
        .filter(|w| w.code == "path_claim_overlap")
        .count();
    if overlaps > 0 {
        eprintln!("Predicted path overlaps scheduled apart: {overlaps} (see stage layout above)");
    }
    eprintln!();
}
