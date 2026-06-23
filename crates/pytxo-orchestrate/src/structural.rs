//! Structural telemetry graph IPC (Phase 27).

use std::path::PathBuf;

use pytxo_signal::StructuralGraph;

use crate::resolve_repo_root;

/// Build a Signal structural graph for an active or recent run.
pub fn structural_graph(
    repo: Option<PathBuf>,
    run_id: &str,
) -> anyhow::Result<StructuralGraph> {
    let repo_root = resolve_repo_root(repo.as_deref())?;
    let (_, store) = crate::open_store(None, Some(repo_root.clone()))?;
    let paths = store.arbitrage_paths_for_run(run_id)?;
    let agents = store.list_agents_for_run(run_id)?;
    let root_by_agent: std::collections::HashMap<String, Option<String>> = agents
        .into_iter()
        .map(|a| (a.id, a.root_id))
        .collect();
    let edited: Vec<(String, String, Option<String>)> = paths
        .into_iter()
        .map(|(path, agent_id)| {
            let root_id = root_by_agent.get(&agent_id).cloned().flatten();
            (path, agent_id, root_id)
        })
        .collect();
    let g = pytxo_signal::build_structural_graph(&repo_root, &edited);
    Ok(pytxo_signal::enrich_symbol_nodes(g, &repo_root))
}
