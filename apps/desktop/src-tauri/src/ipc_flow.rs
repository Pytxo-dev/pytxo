use pytxo_orchestrate::{
    dispatch_flow, preview_flow, save_flow_draft, save_reviewed_flow_plan, FlowDraftInput, FlowPlan,
};
use pytxo_store::{Catalog, FlowDraftRecord};

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
    preview_flow(&catalog, input).map_err(map_orch_err)
}

#[tauri::command]
pub fn flow_save_reviewed_plan(plan: FlowPlan) -> IpcResult<FlowPlan> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    save_reviewed_flow_plan(&catalog, plan).map_err(map_orch_err)
}

#[tauri::command]
pub async fn flow_dispatch(app: tauri::AppHandle, draft_id: String) -> IpcResult<String> {
    let catalog = Catalog::open_default().map_err(map_store_err)?;
    let domain_id = catalog
        .get_flow_draft(&draft_id)
        .map_err(map_store_err)?
        .and_then(|draft| draft.domain_id);
    let run_id = dispatch_flow(&catalog, &draft_id).map_err(map_orch_err)?;
    if let Some(domain_id) = domain_id {
        emit_domain_changed(&app, &domain_id, "run", &run_id);
    }
    Ok(run_id)
}

#[tauri::command]
pub fn flow_history() -> IpcResult<Vec<FlowDraftRecord>> {
    Catalog::open_default()
        .and_then(|catalog| catalog.list_flow_drafts())
        .map_err(map_store_err)
}

#[tauri::command]
pub fn flow_delete(draft_id: String) -> IpcResult<()> {
    Catalog::open_default()
        .and_then(|catalog| catalog.delete_flow_draft(&draft_id))
        .map_err(map_store_err)
}
