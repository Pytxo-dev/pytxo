//! A landing receipt for every committed reviewed Apply, so Desktop and CLI
//! Applies leave the same record as `pytxo land`. Best effort: the Apply has
//! already happened when this runs, so a failure here is logged, not raised.

use std::collections::BTreeMap;
use std::path::Path;

use pytxo_core::{
    new_receipt_id, ExecutionPlan, Receipt, ReceiptCheck, ReceiptInput, ReceiptPredicate,
};
use pytxo_runner::RunApplyManifest;
use pytxo_store::PytxoStore;

pub(crate) fn write_apply_receipt(
    repo_root: &Path,
    store: &PytxoStore,
    run_id: &str,
    applied: &RunApplyManifest,
) -> anyhow::Result<std::path::PathBuf> {
    let contract = store
        .get_run_contract(run_id)?
        .ok_or_else(|| anyhow::anyhow!("run has no contract: {run_id}"))?;
    let plan: Option<ExecutionPlan> = contract
        .plan_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok());
    let agent_of: BTreeMap<String, String> = plan
        .iter()
        .flat_map(|p| p.waves.iter().flatten())
        .map(|t| (t.task_id.0.clone(), t.agent.clone()))
        .collect();

    let mut by_task: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for change in &applied.changes {
        by_task
            .entry(change.source_task_id.clone())
            .or_default()
            .push(change.path.clone());
    }
    let inputs = by_task
        .into_iter()
        .map(|(task, files)| ReceiptInput {
            agent: agent_of.get(&task).cloned(),
            reference: task,
            commit: String::new(),
            status: "applied".into(),
            files,
            conflict_files: Vec::new(),
        })
        .collect();
    let checks = contract
        .prepared_manifest
        .as_ref()
        .and_then(|m| m.candidate_verification.as_ref())
        .map(|v| {
            v.checks
                .iter()
                .map(|c| ReceiptCheck {
                    scope: "combined".into(),
                    command: c.command.clone(),
                    exit_code: None,
                    passed: c.passed,
                    duration_ms: 0,
                    timed_out: false,
                    output_tail: Vec::new(),
                })
                .collect()
        })
        .unwrap_or_default();

    let predicate = ReceiptPredicate {
        id: new_receipt_id(),
        kind: "apply".into(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        pytxo_version: env!("CARGO_PKG_VERSION").into(),
        decision: "applied".into(),
        mode: "apply".into(),
        target: Some(format!("run:{run_id}")),
        base: contract.base_revision.clone().unwrap_or_default(),
        // The reviewed package digest identifies exactly what was written.
        result: contract.prepared_digest.clone(),
        inputs,
        files_changed: applied.changes.len(),
        shared_files: Vec::new(),
        checks,
        reasons: Vec::new(),
        duration_ms: 0,
        content_digest: String::new(),
    };
    let receipt = Receipt::new(predicate, &format!("pytxo-run:{run_id}"), None);
    let dir = pytxo_land::repo_receipts_dir(repo_root)?;
    Ok(receipt.write_to(&dir)?)
}
