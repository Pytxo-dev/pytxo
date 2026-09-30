use pytxo_orchestrate::{
    dispatch_desktop_beta_flow, dispatch_experimental_routed_flow,
    enable_experimental_hosted_advisor_local_consent, enable_experimental_routed_advisor_consent,
    list_flow_drafts_with_routed_recovery, preview_desktop_beta_flow,
    preview_experimental_claude_hosted_shadow_flow_with_facts,
    preview_experimental_claude_proposal_flow_with_facts,
    preview_experimental_routed_advisor_packet, preview_proposed_hosted_advisor_packet,
    preview_reviewed_hosted_advisor_packet, read_experimental_hosted_advisor_local_consent,
    read_experimental_routed_advisor_consent, request_stop_experimental_routed_flow,
    revoke_experimental_hosted_advisor_local_consent, revoke_experimental_routed_advisor_consent,
    save_flow_draft, save_reviewed_flow_plan, stop_exact_routed, FlowDraftInput, FlowPlan,
    ProposedHostedAdvisorPacketPreview, ReviewedDemandFacts, ReviewedHostedAdvisorPacketPreview,
    RoutedAdvisorConsentStatus, RoutedAdvisorPacketPreview,
};
use pytxo_store::capacity::CapacityPoolConfig;
use pytxo_store::{Catalog, FlowDraftRecord};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ipc::emit_domain_changed;
use crate::ipc_error::{map_orch_err, map_store_err, IpcResult};

#[tauri::command]
pub fn flow_save_draft(input: FlowDraftInput) -> IpcResult<FlowDraftRecord> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    save_flow_draft(&catalog, input).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_preview(input: FlowDraftInput) -> IpcResult<FlowPlan> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    preview_desktop_beta_flow(&catalog, input).map_err(map_orch_err)
}

fn configured_claude_experiment() -> Result<(PathBuf, String), String> {
    if !cfg!(windows)
        || std::env::var("PYTXO_EXPERIMENTAL_ROUTED_CLAUDE_PROPOSAL").as_deref() != Ok("1")
    {
        return Err("experimental Claude proposal route is disabled".into());
    }
    let home = std::env::var_os("PYTXO_ROUTED_CLAUDE_ACCOUNT_HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "selected Claude account home is not configured".to_owned())?;
    let cli = std::env::var_os("PYTXO_ROUTED_CLAUDE_EXE")
        .map(PathBuf::from)
        .ok_or_else(|| "selected Claude executable is not configured".to_owned())?;
    let probes = std::env::var_os("PYTXO_ROUTED_CLAUDE_PROBE_ROOT")
        .map(PathBuf::from)
        .ok_or_else(|| "Claude probe root is not configured".to_owned())?;
    if !cli.is_file() || !probes.is_dir() || !probes.is_absolute() {
        return Err("Claude executable or probe root is unavailable".into());
    }
    let source = pytxo_orchestrate::routed_claude::claude_subscription_account_source_id(&home)
        .map_err(|error| error.to_string())?;
    let pool = format!("claude-account:{}", source.0);
    Ok((home, pool))
}

#[tauri::command]
pub fn flow_experimental_claude_available() -> IpcResult<bool> {
    let Ok((_, pool)) = configured_claude_experiment() else {
        return Ok(false);
    };
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    Ok(catalog
        .capacity_pool_status(&pool)
        .map_err(map_store_err)?
        .is_none_or(|status| status.available_units > 0))
}

#[tauri::command]
pub fn flow_experimental_hosted_review_available() -> IpcResult<bool> {
    Ok(configured_claude_experiment().is_ok()
        && std::env::var("PYTXO_EXPERIMENTAL_ROUTED_HOSTED_SHADOW_REVIEW").as_deref() == Ok("1"))
}

#[tauri::command]
pub fn flow_preview_experimental_claude(
    mut input: FlowDraftInput,
    facts: ReviewedDemandFacts,
) -> IpcResult<FlowPlan> {
    let (home, pool) = configured_claude_experiment().map_err(map_orch_err)?;
    input.ade_id = None;
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    ensure_claude_account_pool(&catalog, &pool)?;
    preview_experimental_claude_proposal_flow_with_facts(&catalog, input, &home, &pool, facts)
        .map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_preview_experimental_claude_hosted(
    mut input: FlowDraftInput,
    facts: ReviewedDemandFacts,
) -> IpcResult<FlowPlan> {
    if !flow_experimental_hosted_review_available()? {
        return Err(map_orch_err(
            "hosted Shadow packet review experiment is disabled",
        ));
    }
    let (home, pool) = configured_claude_experiment().map_err(map_orch_err)?;
    input.ade_id = None;
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    ensure_claude_account_pool(&catalog, &pool)?;
    preview_experimental_claude_hosted_shadow_flow_with_facts(&catalog, input, &home, &pool, facts)
        .map_err(map_orch_err)
}

fn ensure_claude_account_pool(catalog: &Catalog, pool: &str) -> IpcResult<()> {
    if catalog
        .capacity_pool_status(pool)
        .map_err(map_store_err)?
        .is_none()
    {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(map_orch_err)?
            .as_millis()
            .try_into()
            .map_err(map_orch_err)?;
        if let Err(error) = catalog.configure_capacity_pool(&CapacityPoolConfig {
            resource_id: pool.to_owned(),
            capacity_units: 1,
            expected_revision: None,
            configured_at_ms: now_ms,
        }) {
            // A second Desktop window may have created the same single-account
            // pool after our read. Never replace or enlarge an existing pool.
            if catalog
                .capacity_pool_status(pool)
                .map_err(map_store_err)?
                .is_none()
            {
                return Err(map_store_err(error));
            }
        }
    }
    if catalog
        .capacity_pool_status(pool)
        .map_err(map_store_err)?
        .is_none_or(|status| status.available_units == 0)
    {
        return Err(map_orch_err("Claude account capacity pool is unavailable"));
    }
    Ok(())
}

#[cfg(test)]
mod experimental_claude_tests {
    use super::*;

    #[test]
    fn explicit_preview_pool_is_single_slot_idempotent_and_never_reenabled() {
        let home = tempfile::tempdir().unwrap();
        let catalog = Catalog::open(&home.path().join("catalog.db")).unwrap();
        let pool = "claude-account:test-source";
        ensure_claude_account_pool(&catalog, pool).unwrap();
        ensure_claude_account_pool(&catalog, pool).unwrap();
        let first = catalog.capacity_pool_status(pool).unwrap().unwrap();
        assert_eq!(first.capacity_units, 1);
        assert_eq!(first.revision, 1);
        catalog
            .configure_capacity_pool(&CapacityPoolConfig {
                resource_id: pool.into(),
                capacity_units: 0,
                expected_revision: Some(first.revision),
                configured_at_ms: 101,
            })
            .unwrap();
        assert!(ensure_claude_account_pool(&catalog, pool).is_err());
        assert_eq!(
            catalog
                .capacity_pool_status(pool)
                .unwrap()
                .unwrap()
                .capacity_units,
            0
        );
    }
}

#[tauri::command]
pub fn flow_save_reviewed_plan(plan: FlowPlan) -> IpcResult<FlowPlan> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    save_reviewed_flow_plan(&catalog, plan).map_err(map_orch_err)
}

#[tauri::command]
pub async fn flow_dispatch(app: tauri::AppHandle, draft_id: String) -> IpcResult<String> {
    // Native routing probes and owned workers are synchronous. Keep Tauri's
    // async executor free so an independent Stop request can reach Core.
    let (run_id, domain_id) = tokio::task::spawn_blocking(move || -> IpcResult<_> {
        let catalog = Catalog::open_default().map_err(map_store_err)?;
        let draft = catalog.get_flow_draft(&draft_id).map_err(map_store_err)?;
        let domain_id = draft.as_ref().and_then(|draft| draft.domain_id.clone());
        let routed = draft
            .as_ref()
            .and_then(|draft| draft.plan_json.as_deref())
            .map(serde_json::from_str::<FlowPlan>)
            .transpose()
            .map_err(map_orch_err)?
            .is_some_and(|plan| plan.routing.is_some());
        let run_id = if routed {
            dispatch_experimental_routed_flow(&catalog, &draft_id)
        } else {
            dispatch_desktop_beta_flow(&catalog, &draft_id)
        }
        .map_err(map_orch_err)?;
        Ok((run_id, domain_id))
    })
    .await
    .map_err(map_orch_err)??;
    if let Some(domain_id) = domain_id {
        emit_domain_changed(&app, &domain_id, "run", &run_id);
    }
    Ok(run_id)
}

#[tauri::command]
pub async fn flow_stop_routed(
    app: tauri::AppHandle,
    draft_id: String,
    run_id: String,
) -> IpcResult<()> {
    let stop_draft_id = draft_id.clone();
    let stop_run_id = run_id.clone();
    let (active_repo, domain_id) = tokio::task::spawn_blocking(move || -> IpcResult<_> {
        let catalog = Catalog::open_default().map_err(map_store_err)?;
        let domain_id = catalog
            .get_flow_draft(&stop_draft_id)
            .map_err(map_store_err)?
            .and_then(|draft| draft.domain_id);
        let active_repo =
            request_stop_experimental_routed_flow(&catalog, &stop_draft_id, &stop_run_id)
                .map_err(map_orch_err)?;
        Ok((active_repo, domain_id))
    })
    .await
    .map_err(map_orch_err)??;
    if let Some(target) = active_repo {
        stop_exact_routed(target, &run_id)
            .await
            .map_err(map_orch_err)?;
    }
    if let Some(domain_id) = domain_id {
        emit_domain_changed(&app, &domain_id, "run", &run_id);
    }
    Ok(())
}

#[tauri::command]
pub fn flow_history() -> IpcResult<Vec<FlowDraftRecord>> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    list_flow_drafts_with_routed_recovery(&catalog).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_advisor_packet_preview(draft_id: String) -> IpcResult<RoutedAdvisorPacketPreview> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    preview_experimental_routed_advisor_packet(&catalog, &draft_id).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_proposed_hosted_packet_preview(
    draft_id: String,
) -> IpcResult<ProposedHostedAdvisorPacketPreview> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    preview_proposed_hosted_advisor_packet(&catalog, &draft_id).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_reviewed_hosted_packet_preview(
    draft_id: String,
) -> IpcResult<ReviewedHostedAdvisorPacketPreview> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    preview_reviewed_hosted_advisor_packet(&catalog, &draft_id).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_hosted_consent_status(domain_id: String) -> IpcResult<RoutedAdvisorConsentStatus> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    read_experimental_hosted_advisor_local_consent(&catalog, &domain_id).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_hosted_consent_enable(
    draft_id: String,
    domain_id: String,
    request_digest: String,
    scope_digest: String,
    expected_revision: u64,
) -> IpcResult<RoutedAdvisorConsentStatus> {
    if !flow_experimental_hosted_review_available()? {
        return Err(map_orch_err("hosted Shadow consent experiment is disabled"));
    }
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    let inspected =
        preview_reviewed_hosted_advisor_packet(&catalog, &draft_id).map_err(map_orch_err)?;
    if inspected.domain_id != domain_id
        || inspected.request_digest.0 != request_digest
        || inspected.scope_digest.0 != scope_digest
    {
        return Err(map_orch_err(
            "hosted packet differs from the inspected review",
        ));
    }
    enable_experimental_hosted_advisor_local_consent(
        &catalog,
        &draft_id,
        &inspected,
        expected_revision,
    )
    .map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_hosted_consent_revoke(
    domain_id: String,
    expected_revision: u64,
) -> IpcResult<RoutedAdvisorConsentStatus> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    revoke_experimental_hosted_advisor_local_consent(&catalog, &domain_id, expected_revision)
        .map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_advisor_consent_domains() -> IpcResult<Vec<String>> {
    Catalog::open_default()
        .and_then(|catalog| catalog.routing_advisor_consent_domains())
        .map_err(map_store_err)
}

#[tauri::command]
pub fn flow_advisor_consent(domain_id: String) -> IpcResult<RoutedAdvisorConsentStatus> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    read_experimental_routed_advisor_consent(&catalog, &domain_id).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_advisor_consent_enable(
    draft_id: String,
    domain_id: String,
    request_digest: String,
    recipient_identity: String,
    expected_revision: u64,
) -> IpcResult<RoutedAdvisorConsentStatus> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    enable_experimental_routed_advisor_consent(
        &catalog,
        &draft_id,
        &domain_id,
        &request_digest,
        &recipient_identity,
        expected_revision,
    )
    .map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_advisor_consent_revoke(
    domain_id: String,
    expected_revision: u64,
) -> IpcResult<RoutedAdvisorConsentStatus> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    revoke_experimental_routed_advisor_consent(&catalog, &domain_id, expected_revision)
        .map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_delete(draft_id: String) -> IpcResult<()> {
    Catalog::open_default()
        .and_then(|catalog| catalog.delete_flow_draft(&draft_id))
        .map_err(map_store_err)
}
