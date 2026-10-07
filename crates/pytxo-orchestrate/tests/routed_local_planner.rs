//! One process-local environment fixture: hostile ambient LLM settings must not
//! change the experimental routed preview's local planner selection.

use std::fs;
use std::process::Command;

use pytxo_core::PytxoConfig;
use pytxo_orchestrate::{
    preview_experimental_routed_flow, preview_flow, FlowDraftInput, FlowSource,
};
use pytxo_store::Catalog;

#[test]
fn routed_preview_stays_local_under_ambient_llm_opt_in() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let home = temp.path().join("home");
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::create_dir_all(home.join(".pytxo")).unwrap();
    fs::write(repo.join("src/lib.rs"), "pub fn value() {}\n").unwrap();
    fs::write(
        repo.join("pytxo.toml"),
        "[coordinator]\nprovider = 'unreachable-test'\nmodel = 'fake'\ntransport = 'direct'\n",
    )
    .unwrap();
    fs::write(
        home.join(".pytxo/providers.json"),
        r#"{"providers":[{"id":"unreachable-test","api_key_env":"","openai_base_url":"http://127.0.0.1:1/v1","openai_compatible":true}]}"#,
    )
    .unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@pytxo.local",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        let output = Command::new("git")
            .args(args)
            .current_dir(&repo)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // This integration binary contains one test and owns these process-local
    // variables; no real provider, credential, or public network is contacted.
    unsafe {
        std::env::set_var("PYTXO_HOME", &home);
        std::env::set_var("PYTXO_PLANNER_LLM", "1");
        std::env::remove_var("PYTXO_PLANNER");
    }
    let config = PytxoConfig::load(&repo.join("pytxo.toml")).unwrap();
    assert!(pytxo_planner::llm_planner_enabled(&config));
    let catalog = Catalog::open(&repo.join(".git/catalog.db")).unwrap();
    let request = FlowDraftInput {
        id: "routing-local".into(),
        title: "Local routing".into(),
        mission_text: "Update src/lib.rs".into(),
        source: FlowSource::Text,
        domain_id: Some(repo.to_string_lossy().into_owned()),
        project_id: None,
        ade_id: None,
        ade_ids: Vec::new(),
        task_ades: Default::default(),
        max_workers: Some(1),
        verification_commands: vec!["echo local-check".into()],
    };
    let local = preview_experimental_routed_flow(&catalog, request.clone(), |_, _, _| {
        anyhow::bail!("local planner reached the trusted builder")
    })
    .unwrap_err();
    assert!(
        local.to_string().contains("local planner reached"),
        "{local:#}"
    );

    // The preexisting Flow entrypoint still honors explicit LLM planner opt-in.
    // Its configured endpoint is an unreachable loopback fixture.
    let legacy = preview_flow(&catalog, request).unwrap_err();
    assert!(
        legacy.to_string().contains("coordinator request failed"),
        "{legacy:#}"
    );
}
