//! Run ledger idempotency across simulated service restarts.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
struct Ledger {
    runs: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl Ledger {
    fn start(&self, domain_id: &str, run_id: &str) {
        let mut m = self.runs.lock().unwrap();
        m.entry(run_id.to_string()).or_insert_with(|| {
            serde_json::json!({
                "domain_id": domain_id,
                "status": "started",
            })
        });
    }

    fn end(&self, domain_id: &str, run_id: &str, usage: &serde_json::Value) {
        let mut m = self.runs.lock().unwrap();
        let entry = m.entry(run_id.to_string()).or_insert_with(|| {
            serde_json::json!({
                "domain_id": domain_id,
                "status": "started",
            })
        });
        entry["status"] = serde_json::json!("completed");
        entry["usage"] = usage.clone();
    }

    fn get(&self, run_id: &str) -> Option<serde_json::Value> {
        self.runs.lock().unwrap().get(run_id).cloned()
    }
}

fn spawn_link_stub(ledger: Ledger) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let handle = thread::spawn(move || {
        listener.set_nonblocking(true).ok();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = vec![0u8; 16_384];
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    continue;
                }
                let req = String::from_utf8_lossy(&buf[..n]);
                let status = if req.contains("runs/start") {
                    if let Some(body) = req.split("\r\n\r\n").nth(1) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                            ledger.start(
                                v["domain_id"].as_str().unwrap_or(""),
                                v["run_id"].as_str().unwrap_or(""),
                            );
                        }
                    }
                    "200"
                } else if req.contains("runs/end") {
                    if let Some(body) = req.split("\r\n\r\n").nth(1) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                            ledger.end(
                                v["domain_id"].as_str().unwrap_or(""),
                                v["run_id"].as_str().unwrap_or(""),
                                &v["usage"],
                            );
                        }
                    }
                    "200"
                } else if req.contains("/health") {
                    "200"
                } else {
                    "404"
                };
                let body = if req.contains("/health") { "ok" } else { "" };
                let resp = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
            } else {
                thread::sleep(Duration::from_millis(5));
            }
        }
    });

    (port, handle)
}

fn post_json(port: u16, path: &str, body: &str) {
    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut resp = vec![0u8; 1024];
    let n = stream.read(&mut resp).unwrap_or(0);
    let text = String::from_utf8_lossy(&resp[..n]);
    assert!(text.contains("200"), "unexpected response: {text}");
}

#[test]
fn run_ledger_idempotent_across_restart_simulation() {
    let ledger = Ledger::default();
    let domain = "/repo/restart";
    let run_id = "run-restart-1";
    let start_body = format!(r#"{{"domain_id":"{domain}","run_id":"{run_id}"}}"#);
    let end_body = format!(
        r#"{{"domain_id":"{domain}","run_id":"{run_id}","usage":{{"tokens_in_billed":20,"tokens_in_sent":12,"tokens_out":6,"saved_tokens":8,"cost_micro_usd":500}}}}"#
    );

    let (port1, server1) = spawn_link_stub(ledger.clone());
    thread::sleep(Duration::from_millis(20));
    post_json(port1, "/v1/runs/start", &start_body);
    drop(server1);

    let (port2, server2) = spawn_link_stub(ledger.clone());
    thread::sleep(Duration::from_millis(20));
    post_json(port2, "/v1/runs/start", &start_body);
    post_json(port2, "/v1/runs/end", &end_body);
    drop(server2);

    let (port3, server3) = spawn_link_stub(ledger.clone());
    thread::sleep(Duration::from_millis(20));
    post_json(port3, "/v1/runs/end", &end_body);
    server3.join().unwrap();

    let record = ledger.get(run_id).expect("run persisted across restarts");
    assert_eq!(record["status"], "completed");
    assert_eq!(record["usage"]["cost_micro_usd"], 500);
}
