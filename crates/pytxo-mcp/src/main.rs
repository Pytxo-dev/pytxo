use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use pytxo_core::FidelityTier;
use pytxo_orchestrate::{
    dry_run_json, logs, open_store, read_file, read_file_scaffolded, run, RunOptions,
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
                        "content": [{ "type": "text", "text": text }],
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
        "pytxo_read_scaffolded" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("path required"))?;
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
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
        "pytxo_read" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("path required"))?;
            let repo = args.get("repo").and_then(|v| v.as_str()).map(PathBuf::from);
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
