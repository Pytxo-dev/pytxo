use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use pytxo_core::{PytxoError, Result};
use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct ChildMcpSession {
    pub addr: SocketAddr,
}

impl ChildMcpSession {
    pub fn call(&self, method: &str, params: Value) -> Result<Value> {
        let mut stream = TcpStream::connect_timeout(&self.addr, Duration::from_secs(2))
            .map_err(|e| PytxoError::Runner(format!("mcp child connect: {e}")))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(PytxoError::Io)?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(PytxoError::Io)?;
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        writeln!(stream, "{req}").map_err(PytxoError::Io)?;
        stream.flush().map_err(PytxoError::Io)?;
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = line.map_err(PytxoError::Io)?;
            if line.trim().is_empty() {
                continue;
            }
            let resp: Value = serde_json::from_str(&line)
                .map_err(|e| PytxoError::Other(format!("mcp child parse: {e}")))?;
            if let Some(err) = resp.get("error") {
                return Err(PytxoError::Runner(format!("child mcp error: {err}")));
            }
            return Ok(resp.get("result").cloned().unwrap_or(Value::Null));
        }
        Err(PytxoError::Runner("child mcp: empty response".into()))
    }

    pub fn list_tools(&self) -> Result<Vec<Value>> {
        let result = self.call("tools/list", json!({}))?;
        Ok(result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default())
    }
}

#[derive(Default)]
struct HubInner {
    sessions: HashMap<String, ChildMcpSession>,
    /// MCP v3 resource subscriptions (Phase 67): agent_key -> resource URI.
    subscriptions: HashMap<String, Vec<String>>,
}

/// Per-domain registry of live child MCP servers ([[mcp-router]] v2).
#[derive(Clone, Default)]
pub struct McpHub {
    inner: Arc<RwLock<HubInner>>,
}

impl McpHub {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, agent_key: &str, session: ChildMcpSession) {
        if let Ok(mut guard) = self.inner.write() {
            guard.sessions.insert(agent_key.to_string(), session);
        }
    }

    pub fn deregister(&self, agent_key: &str) {
        if let Ok(mut guard) = self.inner.write() {
            guard.sessions.remove(agent_key);
        }
    }

    /// Forward JSON-RPC to a live child session. Supports multi-hop routes
    /// (`agent:a->agent:b`) when the final hop receives `method`/`params`.
    pub fn proxy_call(&self, agent_key: &str, method: &str, params: Value) -> Result<Value> {
        if agent_key.contains("->") {
            return self.proxy_multi_hop(agent_key, method, params);
        }
        let session = self
            .inner
            .read()
            .map_err(|_| PytxoError::Runner("mcp hub lock poisoned".into()))?
            .sessions
            .get(agent_key)
            .cloned()
            .ok_or_else(|| {
                PytxoError::Runner(format!("no live child MCP session for {agent_key}"))
            })?;
        session.call(method, params)
    }

    fn proxy_multi_hop(&self, route: &str, method: &str, params: Value) -> Result<Value> {
        let hops: Vec<&str> = route.split("->").map(str::trim).filter(|s| !s.is_empty()).collect();
        if hops.len() < 2 {
            return self.proxy_call(route, method, params);
        }
        let mut carry = params;
        for (i, hop) in hops.iter().enumerate() {
            let is_last = i + 1 == hops.len();
            if is_last {
                return self.proxy_call(hop, method, carry);
            }
            carry = json!({
                "name": format!("agent:{}/relay", hop.split(':').nth(1).unwrap_or(hop)),
                "arguments": carry,
            });
            carry = self.proxy_call(hop, "tools/call", carry)?;
        }
        Err(PytxoError::Runner("multi-hop route produced no result".into()))
    }

    /// MCP v3: register a resource URI subscription for an agent session.
    pub fn subscribe_resource(&self, agent_key: &str, uri: &str) -> Result<()> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| PytxoError::Runner("mcp hub lock poisoned".into()))?;
        guard
            .subscriptions
            .entry(agent_key.to_string())
            .or_default()
            .push(uri.to_string());
        Ok(())
    }

    /// MCP v3: list active subscriptions for an agent.
    pub fn list_subscriptions(&self, agent_key: &str) -> Vec<String> {
        self.inner
            .read()
            .ok()
            .and_then(|g| g.subscriptions.get(agent_key).cloned())
            .unwrap_or_default()
    }

    /// MCP v3: forward `notifications/resources/updated` to subscribed agents.
    pub fn notify_resource_updated(&self, uri: &str) -> Result<usize> {
        let guard = self
            .inner
            .read()
            .map_err(|_| PytxoError::Runner("mcp hub lock poisoned".into()))?;
        let mut notified = 0usize;
        for (agent_key, uris) in &guard.subscriptions {
            if uris.iter().any(|u| u == uri) {
                if let Some(session) = guard.sessions.get(agent_key) {
                    let _ = session.call(
                        "notifications/resources/updated",
                        json!({ "uri": uri }),
                    );
                    notified += 1;
                }
            }
        }
        Ok(notified)
    }

    pub fn aggregate_tools(&self) -> Result<Vec<Value>> {
        let guard = self
            .inner
            .read()
            .map_err(|_| PytxoError::Runner("mcp hub lock poisoned".into()))?;
        let mut out = Vec::new();
        for (agent_key, session) in &guard.sessions {
            let agent_id = agent_key
                .split_once(':')
                .map(|(_, a)| a)
                .unwrap_or(agent_key.as_str());
            for tool in session.list_tools().unwrap_or_default() {
                let name = tool
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                let mut entry = tool.clone();
                if let Some(obj) = entry.as_object_mut() {
                    obj.insert(
                        "name".into(),
                        Value::String(format!("agent:{agent_id}/{name}")),
                    );
                    obj.insert(
                        "description".into(),
                        Value::String(format!(
                            "Proxied from {agent_key}: {}",
                            tool.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                        )),
                    );
                }
                out.push(entry);
            }
        }
        Ok(out)
    }
}

fn handle_test_mcp_conn(stream: &mut TcpStream) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    let Some(Ok(line)) = reader.lines().next() else {
        return;
    };
    let req: Value = serde_json::from_str(&line).unwrap_or(json!({}));
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let result = match method {
        "tools/list" => json!({
            "tools": [{
                "name": "echo_fixture",
                "description": "test fixture tool",
                "inputSchema": { "type": "object" }
            }]
        }),
        "tools/call" => json!({
            "content": [{ "type": "text", "text": "ok" }],
            "isError": false
        }),
        _ => Value::Null,
    };
    let resp = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    let _ = writeln!(stream, "{resp}");
    let _ = stream.flush();
}

/// Spawn a production child MCP session when `PYTXO_AGENT_MCP_ADDR` (+ optional
/// `PYTXO_AGENT_MCP_CMD` launcher) is set; otherwise fall back to the TCP fixture.
pub fn spawn_agent_mcp_child() -> Result<(ChildMcpSession, thread::JoinHandle<()>)> {
    if let Ok(addr_str) = std::env::var("PYTXO_AGENT_MCP_ADDR") {
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| PytxoError::Runner(format!("PYTXO_AGENT_MCP_ADDR parse: {e}")))?;
        let handle = if let Ok(cmd) = std::env::var("PYTXO_AGENT_MCP_CMD") {
            if !cmd.trim().is_empty() {
                let cmd_line = cmd;
                thread::spawn(move || {
                    let _ = Command::new(if cfg!(windows) { "cmd" } else { "sh" })
                        .arg(if cfg!(windows) { "/C" } else { "-c" })
                        .arg(cmd_line)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                })
            } else {
                thread::spawn(|| {})
            }
        } else {
            thread::spawn(|| {})
        };
        thread::sleep(Duration::from_millis(100));
        return Ok((ChildMcpSession { addr }, handle));
    }
    if std::env::var("PYTXO_MCP_TEST").ok().as_deref() == Some("1") {
        return spawn_test_mcp_child();
    }
    Err(PytxoError::Runner(
        "no agent MCP child: set PYTXO_AGENT_MCP_ADDR or PYTXO_MCP_TEST=1".into(),
    ))
}

/// Test fixture: minimal MCP TCP server on localhost.
pub fn spawn_test_mcp_child() -> Result<(ChildMcpSession, thread::JoinHandle<()>)> {
    use std::net::TcpListener;
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| PytxoError::Runner(format!("bind: {e}")))?;
    let addr = listener.local_addr().map_err(PytxoError::Io)?;
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    listener.set_nonblocking(true).map_err(PytxoError::Io)?;
    let handle = thread::spawn(move || {
        let _ = ready_tx.send(());
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => handle_test_mcp_conn(&mut stream),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
    ready_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| PytxoError::Runner("test mcp child thread did not start".into()))?;
    Ok((ChildMcpSession { addr }, handle))
}
