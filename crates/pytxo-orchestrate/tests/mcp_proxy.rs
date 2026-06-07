use pytxo_runner::{spawn_test_mcp_child, McpHub};
use serde_json::json;

#[test]
fn mcp_hub_proxy_forwards_to_child() {
    let hub = McpHub::new();
    let (session, handle) = spawn_test_mcp_child().expect("fixture");
    hub.register("run1:agent-0", session);

    let tools = hub.aggregate_tools().expect("aggregate");
    assert!(tools
        .iter()
        .any(|t| t.get("name").and_then(|n| n.as_str()) == Some("agent:agent-0/echo_fixture")));

    let result = hub
        .proxy_call(
            "run1:agent-0",
            "tools/call",
            json!({ "name": "echo_fixture" }),
        )
        .expect("proxy");
    assert!(result.get("content").is_some());

    hub.deregister("run1:agent-0");
    handle.join().ok();
}
