//! Seeds an isolated Pytxo home with a long, realistic run history so Desktop
//! startup, History, Review and log views can be measured against real store
//! sizes. Never point it at a real home: it refuses an existing `--home`.
//!
//!   cargo run --release -p pytxo-store --example seed_history -- \
//!     --home D:/fixture/home --repos D:/fixture/repos [--domains 6] [--runs 400] [--agents 3] [--events 80] [--tail-events 5000]
use chrono::{Duration, Utc};
use pytxo_core::{
    canonical_repo_root, PreparedRunFile, PreparedRunFileKind, PreparedRunManifest,
    PreparedRunSummary, PytxoConfig,
};
use pytxo_store::{Catalog, FlowDraftRecord, PytxoStore};
use rusqlite::params;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let num = |name: &str, default: usize| arg(name).map_or(Ok(default), |v| v.parse());
    let home = PathBuf::from(arg("--home").ok_or("--home is required")?);
    let repos = PathBuf::from(arg("--repos").ok_or("--repos is required")?);
    let (domains, runs, agents, events) = (
        num("--domains", 6)?,
        num("--runs", 400)?,
        num("--agents", 3)?,
        num("--events", 80)?,
    );
    // The newest run in each workspace carries a long worker log.
    let tail_events = num("--tail-events", 5000)?;
    if home.exists() {
        return Err(format!("refusing to reuse existing home {}", home.display()).into());
    }
    std::fs::create_dir_all(&home)?;
    std::env::set_var("PYTXO_HOME", &home);
    let catalog = Catalog::open_default()?;
    let cfg = PytxoConfig::default();
    let now = Utc::now();
    let line = "worker: compiled crate, ran 214 tests, 0 failed; next step writes src/lib.rs and updates the review summary".repeat(2);
    let manifest = |run_id: &str, at: &str| {
        let files: Vec<PreparedRunFile> = (0..60u64)
            .map(|f| PreparedRunFile {
                path: format!("src/module_{f}/file_{f}.rs"),
                kind: PreparedRunFileKind::Modify,
                before_sha256: Some(format!("{f:064x}")),
                after_sha256: Some(format!("{:064x}", f + 1)),
                byte_count: 4096,
                task_id: format!("task-{}", f % 3),
                agent_id: format!("agent-{}", f % 3),
                blob_digest: Some(format!("{:064x}", f + 2)),
                before_mode: None,
                after_mode: None,
                before_byte_count: 4000,
                after_byte_count: 4096,
                before_is_binary: Some(false),
                after_is_binary: Some(false),
                before_chunks: Vec::new(),
                after_chunks: Vec::new(),
            })
            .collect();
        let changes: Vec<String> = files.iter()
            .map(|f| format!(r#"{{"path":"{}","kind":"modify","source_agent_id":"{}","source_task_id":"{}","base_digest":"{}","result_digest":"{}"}}"#,
                f.path, f.agent_id, f.task_id, f.before_sha256.as_deref().unwrap_or(""), f.after_sha256.as_deref().unwrap_or("")))
            .collect();
        let prepared = PreparedRunManifest {
            version: 1,
            run_id: run_id.into(),
            base_revision: format!("{:040x}", 7),
            prepared_at: at.into(),
            package_digest: format!("{:064x}", 9),
            summary: PreparedRunSummary {
                added: 0,
                modified: 60,
                deleted: 0,
                bytes: 60 * 4096,
            },
            files,
            candidate_verification: None,
        };
        (
            serde_json::to_string(&prepared).expect("manifest json"),
            format!(
                r#"{{"transaction_id":"tx-{run_id}","changes":[{}]}}"#,
                changes.join(",")
            ),
        )
    };
    let enforcement = r#"{"run":{"workspace_isolation":{"status":"enforced","mechanism":"git-worktree"},"filesystem":{"status":"enforced","mechanism":"overlay"},"network":{"status":"advisory","mechanism":"none"},"process":{"status":"enforced","mechanism":"job-object"}},"agents":[]}"#;
    for d in 0..domains {
        let repo = repos.join(format!("workspace-{d:02}"));
        std::fs::create_dir_all(&repo)?;
        let repo = canonical_repo_root(&repo)?;
        let domain_id = repo.to_string_lossy().into_owned();
        let db_path = cfg.db_path_at(&repo);
        drop(PytxoStore::open(&db_path)?);
        catalog.upsert_domain(&domain_id, &domain_id, &db_path.to_string_lossy(), None)?;
        let mut conn = rusqlite::Connection::open(&db_path)?;
        let tx = conn.transaction()?;
        for r in 0..runs {
            let run_id = uuid::Uuid::new_v4().to_string();
            let started = now - Duration::minutes(((runs - r) * 300 + d * 7) as i64);
            let status = match r % 17 {
                0 => "failed",
                5 => "stopped",
                _ => "completed",
            };
            tx.execute(
                "INSERT INTO runs (id, started_at, finished_at, status, repo_root, permission_profile, estimated_cost_usd) VALUES (?1, ?2, ?3, ?4, ?5, 'orbit', 0.42)",
                params![run_id, started.to_rfc3339(), (started + Duration::minutes(9)).to_rfc3339(), status, domain_id],
            )?;
            tx.execute("INSERT INTO domain_changes(entity_kind, entity_id, changed_at) VALUES ('run', ?1, ?2)", params![run_id, started.to_rfc3339()])?;
            let tasks: Vec<String> = (0..agents)
                .map(|a| format!(r#"{{"id":"task-{a}","prompt":"Refactor module {a} of workspace {d}, keep the public API, add tests for the edge cases listed in the issue, and update the docs page.","depends_on":[]}}"#))
                .collect();
            let plan = format!(
                r#"{{"title":"Run {r} in workspace {d}","tasks":[{}]}}"#,
                tasks.join(",")
            );
            if r % 7 != 0 {
                let apply = ["applied", "applied", "pending", "prepared", "applied"][r % 5];
                let has_manifest = apply != "pending";
                let (prepared, applied) = manifest(&run_id, &started.to_rfc3339());
                tx.execute(
                    "INSERT INTO run_contracts (run_id, base_revision, plan_json, apply_status, apply_manifest_json, applied_at, enforcement_json, prepared_manifest_json, prepared_digest, prepared_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![run_id, format!("{:040x}", r), plan, apply,
                        (apply == "applied").then_some(applied.as_str()),
                        (apply == "applied").then(|| started.to_rfc3339()),
                        enforcement,
                        has_manifest.then_some(prepared.as_str()),
                        has_manifest.then(|| format!("{:064x}", r)),
                        has_manifest.then(|| started.to_rfc3339())],
                )?;
            }
            catalog.upsert_flow_draft(&FlowDraftRecord {
                id: uuid::Uuid::new_v4().to_string(),
                title: format!("Run {r} in workspace {d}"),
                mission_text: format!(
                    "Refactor workspace {d} module set {r} and keep the tests green."
                ),
                source: "text".into(),
                domain_id: Some(domain_id.clone()),
                project_id: None,
                status: "dispatched".into(),
                plan_json: Some(plan.clone()),
                dispatched_run_id: Some(run_id.clone()),
                created_at: started.to_rfc3339(),
                updated_at: started.to_rfc3339(),
            })?;
            for a in 0..agents {
                let agent_id = uuid::Uuid::new_v4().to_string();
                let agent_status = if status == "failed" && a == 0 {
                    "failed"
                } else {
                    "completed"
                };
                tx.execute(
                    "INSERT INTO agents (id, run_id, task_id, wave, worktree_path, cmd, exit_code, status) VALUES (?1, ?2, ?3, 0, NULL, 'codex exec', ?4, ?5)",
                    params![agent_id, run_id, format!("task-{a}"), i32::from(agent_status == "failed"), agent_status],
                )?;
                tx.execute("INSERT INTO domain_changes(entity_kind, entity_id, changed_at) VALUES ('agent', ?1, ?2)", params![agent_id, started.to_rfc3339()])?;
                // Like append_event: every event also lands in the change log.
                let mut insert = tx.prepare_cached("INSERT INTO events (agent_id, ts, kind, payload) VALUES (?1, ?2, 'stdout', ?3)")?;
                let mut change = tx.prepare_cached("INSERT INTO domain_changes(entity_kind, entity_id, changed_at) VALUES ('event', ?1, ?2)")?;
                for e in 0..if r + 1 == runs { tail_events } else { events } {
                    let ts = (started + Duration::seconds(e as i64)).to_rfc3339();
                    insert.execute(params![agent_id, ts, format!("{e:05} {line}")])?;
                    change.execute(params![tx.last_insert_rowid().to_string(), ts])?;
                }
            }
        }
        tx.commit()?;
        println!("{domain_id}: {runs} runs");
    }
    for f in 0..50 {
        let started = (now - Duration::hours(f * 6)).to_rfc3339();
        catalog.insert_fleet_run(&format!("fleet-run-{f}"), "nightly", &started)?;
        catalog.finish_fleet_run(&format!("fleet-run-{f}"), "completed", &started)?;
    }
    Ok(())
}
