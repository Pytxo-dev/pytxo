use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
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

    pub fn proxy_call(&self, agent_key: &str, method: &str, params: Value) -> Result<Value> {
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

/// Test fixture: minimal MCP TCP server on localhost.
pub fn spawn_test_mcp_child() -> Result<(ChildMcpSession, std::thread::JoinHandle<()>)> {
    use std::net::TcpListener;
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| PytxoError::Runner(format!("bind: {e}")))?;
    listener.set_nonblocking(true).map_err(PytxoError::Io)?;
    let addr = listener.local_addr().map_err(PytxoError::Io)?;
    let handle = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if let Ok((mut stream, _)) = listener.accept() {
                let reader = BufReader::new(stream.try_clone().unwrap());
                for line in reader.lines().map_while(|l| l.ok()) {
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
                }
            } else {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    });
    std::thread::sleep(Duration::from_millis(20));
    Ok((ChildMcpSession { addr }, handle))
}
