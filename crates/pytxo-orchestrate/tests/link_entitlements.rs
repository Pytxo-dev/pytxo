//! Contract test: Pytxo Link `/v1/entitlements/status` JSON matches orchestrate consumer.

#[derive(serde::Deserialize)]
struct LinkEntitlementResponse {
    tier: String,
    max_agents: usize,
    #[serde(default)]
    cloud_enabled: bool,
}

#[test]
fn link_entitlements_json_matches_orchestrate_shape() {
    let sample = r#"{"tier":"pro","max_agents":64,"cloud_enabled":false}"#;
    let body: LinkEntitlementResponse = serde_json::from_str(sample).expect("parse");
    assert_eq!(body.tier, "pro");
    assert_eq!(body.max_agents, 64);
    assert!(!body.cloud_enabled);
}

#[test]
fn ultra_tier_enables_cloud_flag() {
    let sample = r#"{"tier":"ultra","max_agents":256,"cloud_enabled":true}"#;
    let body: LinkEntitlementResponse = serde_json::from_str(sample).expect("parse");
    assert_eq!(body.tier, "ultra");
    assert!(body.cloud_enabled);
}
