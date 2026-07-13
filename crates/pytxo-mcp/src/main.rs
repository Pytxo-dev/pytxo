use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use pytxo_core::FidelityTier;
use pytxo_orchestrate::{
    audit_mcp_tool, dry_run_json, enqueue_agent_stdin, fleet_run, fleet_status, list_live_agents,
    logs, mcp_proxy_call, mcp_tools_list, open_store, project_load, project_run, read_file,
    read_file_scaffolded, resolve_repo_root, run, FleetRunOptions, ProjectRunOptions, RunOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn sanitize_enabled() -> bool {
    if let Ok(v) = std::env::var("PYTXO_SANITIZE") {
        if v == "0" || v.eq_ignore_ascii_case("false") {
            return false;
        }
        if v == "1" || v.eq_ignore_ascii_case("true") {
            return true;
        }
    }
    true
}

fn sanitize_tool_text(text: &str) -> String {
    if sanitize_enabled() {
        pytxo_sanitize::sanitize_line(text)
    } else {
        text.to_string()
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                write_response(
                    &mut stdout,
                    None,
                    None,
                    Some(JsonRpcError {
                        code: -32700,
                        message: format!("parse error: {e}"),
                    }),
                )?;
                continue;
            }
        };

        if req.method.as_deref() == Some("initialize") {
            write_response(
                &mut stdout,
                req.id,
                Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "pytxo-mcp", "version": env!("CARGO_PKG_VERSION") }
                })),
                None,
            )?;
            continue;
        }

        if req.method.as_deref() == Some("tools/list") {
            write_response(
                &mut stdout,
                req.id,
                Some(json!({
                    "tools": [
                        tool_def("pytxo_dry_run", "Dry-run execution plan"),
                        tool_def("pytxo_run", "Start a Pytxo run"),
                        tool_def("pytxo_status", "Recent runs and agents"),
                        tool_def("pytxo_logs", "Tail agent log events"),
                        tool_def(
                            "pytxo_read",
                            "Read repo file (Signal Core skeleton when signal_core=true)",
                        ),
                        tool_def(
                            "pytxo_read_scaffolded",
                            "Signal Core: read file as AST skeleton (always scaffold)",
                        ),
                        tool_def(
                            "pytxo_stdin",
                            "Enqueue stdin bytes for a live agent (PTY / Race Shield)",
                        ),
                        tool_def(
                            "pytxo_route_stdin",
                            "Route stdin to a live agent (validates session is active)",
                        ),
                        tool_def(
                            "pytxo_list_live_agents",
                            "List live agent sessions in the domain Race Shield registry",
                        ),
                        tool_def(
                            "pytxo_mcp_proxy",
                            "Forward JSON-RPC to a live child agent MCP session (v2 hub)",
                        ),
                        tool_def(
                            "pytxo_mcp_tools_list",
                            "Aggregate tools from live child MCP sessions (namespaced)",
                        ),
                        tool_def(
                            "pytxo_project_run",
                            "Run a command across a modular project's writable roots (project_id or manifest)",
                        ),
                        tool_def(
                            "pytxo_fleet_run",
                            "Run a cross-repo fleet DAG (fleet_id or manifest)",
                        ),
                        tool_def(
                            "pytxo_fleet_status",
                            "List recent fleet runs from the hypervisor catalog",
                        ),
                    ]
                })),
                None,
            )?;
            continue;
        }

        if req.method.as_deref() == Some("tools/call") {
            let result = handle_tool_call(req.params);
            match result {
                Ok(text) => write_response(
                    &mut stdout,
                    req.id,
                    Some(json!({
                        "content": [{ "type": "text", "text": sanitize_tool_text(&text) }],
                        "isError": false
                    })),
                    None,
                )?,
                Err(e) => write_response(
                    &mut stdout,
                    req.id,
                    Some(json!({
                        "content": [{ "type": "text", "text": e.to_string() }],
                        "isError": true
                    })),
                    None,
                )?,
            }
            continue;
        }

        if req.id.is_some() {
            write_response(
                &mut stdout,
                req.id,
                None,
                Some(JsonRpcError {
                    code: -32601,
                    message: "method not found".into(),
                }),
            )?;
        }
    }
    Ok(())
}

/// Resolve a repo path from `repo`, or from `project_id` (+ optional `root` label)
/// by loading the modular project manifest ([[modular-projects]]).
fn resolve_repo(args: &Value) -> anyhow::Result<Option<PathBuf>> {
    if let Some(r) = args.get("repo").and_then(|v| v.as_str()) {
        return Ok(Some(PathBuf::from(r)));
    }
    if let Some(pid) = args.get("project_id").and_then(|v| v.as_str()) {
        let manifest = project_load(None, Some(pid.to_string()))?;
        let root = match args.get("root").and_then(|v| v.as_str()) {
            Some(label) => manifest
                .root_by_label(label)
                .ok_or_else(|| anyhow::anyhow!("root '{label}' not in project '{pid}'"))?,
            None => manifest.primary_root(),
        };
        return Ok(Some(root.path.clone()));
    }
    Ok(None)
}

fn tool_def(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": {} }
    })
}

fn handle_tool_call(params: Option<Value>) -> anyhow::Result<String> {
    let params = params.ok_or_else(|| anyhow::anyhow!("missing params"))?;
    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing tool name"))?;
    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
        "pytxo_dry_run" => {
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            let config = args
                .get("config_path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let agents = args.get("agents").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            Ok(dry_run_json(config, repo, agents)?)
        }
        "pytxo_run" => {
            let rt = tokio::runtime::Runtime::new()?;
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            let config = args
                .get("config_path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let cmd = args
                .get("cmd")
                .and_then(|v| v.as_str())
                .unwrap_or("echo pytxo")
                .to_string();
            let agents = args.get("agents").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let run_id = rt.block_on(run(RunOptions {
                agents,
                cmd,
                config,
                dry_run: false,
                keep_worktrees: false,
                repo,
                execution: None,
                project: None,
                tasks: None,
                task_cmd_template: None,
                task_prompts: None,
            }))?;
            Ok(json!({ "run_id": run_id.0 }).to_string())
        }
        "pytxo_status" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let (_cfg, store) = open_store(None, None)?;
            let runs = store.list_runs(limit)?;
            Ok(serde_json::to_string_pretty(&runs)?)
        }
        "pytxo_logs" => {
            let agent = args
                .get("agent_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("agent_id required"))?;
            let tail = args.get("tail").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            Ok(logs(None, None, agent, tail)?.join("\n"))
        }
        "pytxo_project_run" => {
            let rt = tokio::runtime::Runtime::new()?;
            let project_id = args
                .get("project_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let manifest = args
                .get("manifest")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let cmd = args
                .get("cmd")
                .and_then(|v| v.as_str())
                .unwrap_or("echo pytxo")
                .to_string();
            let agents = args.get("agents").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let results = rt.block_on(project_run(ProjectRunOptions {
                manifest,
                project_id,
                cmd,
                agents,
                config: None,
                dry_run: false,
            }))?;
            Ok(serde_json::to_string_pretty(&results)?)
        }
        "pytxo_read_scaffolded" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("path required"))?;
            let repo = resolve_repo(&args)?;
            let config = args
                .get("config_path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let fidelity = args
                .get("fidelity")
                .and_then(|v| v.as_str())
                .and_then(FidelityTier::parse);
            let result = read_file_scaffolded(config, repo, path, fidelity)?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        "pytxo_stdin" => {
            let agent_key = args
                .get("agent_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("agent_key required"))?;
            let data = args
                .get("data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("data required"))?;
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            enqueue_agent_stdin(repo.clone(), agent_key, data.as_bytes())?;
            Ok(json!({ "ok": true }).to_string())
        }
        "pytxo_route_stdin" => {
            let agent_key = args
                .get("agent_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("agent_key required"))?;
            let data = args
                .get("data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("data required"))?;
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            enqueue_agent_stdin(repo, agent_key, data.as_bytes())?;
            Ok(json!({ "ok": true, "routed": true }).to_string())
        }
        "pytxo_list_live_agents" => {
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            let run_id = args.get("run_id").and_then(|v| v.as_str());
            let sessions = list_live_agents(repo.clone(), run_id)?;
            if let Ok(repo_root) = resolve_repo_root(repo.as_deref()) {
                if let Ok((cfg, _)) = open_store(None, Some(repo_root.clone())) {
                    let _ = audit_mcp_tool(
                        &repo_root,
                        &cfg,
                        "mcp:hub",
                        "pytxo_list_live_agents",
                        &serde_json::to_string(&sessions).unwrap_or_default(),
                    );
                }
            }
            Ok(serde_json::to_string_pretty(&sessions)?)
        }
        "pytxo_mcp_proxy" => {
            let agent_key = args
                .get("agent_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("agent_key required"))?;
            let method = args
                .get("method")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("method required"))?;
            let params = args.get("params").cloned().unwrap_or(json!({}));
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            let result = mcp_proxy_call(repo, agent_key, method, params)?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        "pytxo_mcp_tools_list" => {
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
            let tools = mcp_tools_list(repo)?;
            Ok(serde_json::to_string_pretty(&tools)?)
        }
        "pytxo_fleet_run" => {
            let rt = tokio::runtime::Runtime::new()?;
            let manifest = args
                .get("manifest")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let fleet_id = args
                .get("fleet_id")
                .and_then(|v| v.as_str())
                .map(String::from);
            let dry_run = args
                .get("dry_run")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let continue_on_error = args
                .get("continue_on_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let result = rt.block_on(fleet_run(FleetRunOptions {
                manifest,
                fleet_id,
                dry_run,
                continue_on_error,
                ..FleetRunOptions::default()
            }))?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        "pytxo_fleet_status" => {
            let fleet_id = args.get("fleet_id").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let rows = fleet_status(fleet_id, limit)?;
            Ok(serde_json::to_string_pretty(&rows)?)
        }
        "pytxo_read" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("path required"))?;
            let repo = resolve_repo(&args)?;
            let config = args
                .get("config_path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let force_raw = args.get("raw").and_then(|v| v.as_bool()).unwrap_or(false);
            let result = read_file(config, repo, path, force_raw)?;
            Ok(serde_json::to_string_pretty(&result)?)
        }
        other => anyhow::bail!("unknown tool: {other}"),
    }
}

fn write_response(
    stdout: &mut io::Stdout,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
) -> io::Result<()> {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result,
        error,
    };
    let line = serde_json::to_string(&resp).expect("serialize");
    writeln!(stdout, "{line}")?;
    stdout.flush()?;
    Ok(())
}
