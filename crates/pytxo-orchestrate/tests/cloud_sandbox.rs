#[cfg(feature = "cloud-http")]
mod cloud_http {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use pytxo_core::{CloudConfig, CloudDispatcher, ExecutionBackend, HttpCloudDispatcher, PytxoConfig};
    use pytxo_orchestrate::cloud::ping_cloud;

    #[test]
    fn cloud_health_ping() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits2 = Arc::clone(&hits);
        let server = thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = vec![0u8; 4096];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n > 0 && String::from_utf8_lossy(&buf[..n]).contains("/health") {
                        hits2.fetch_add(1, Ordering::SeqCst);
                    }
                    let body = "ok";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        let cfg = CloudConfig {
            enabled: true,
            sandbox_url: format!("http://127.0.0.1:{port}/v1"),
            ..Default::default()
        };
        ping_cloud(&cfg).unwrap();
        server.join().unwrap();
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cloud_start_sandbox_mock() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = vec![0u8; 8192];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let body = if req.contains("sandboxes/start") {
                        r#"{"sandbox_id":"sb-1","ws_url":null}"#
                    } else {
                        "{}"
                    };
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        let cfg = CloudConfig {
            sandbox_url: format!("http://127.0.0.1:{port}/v1"),
            ..Default::default()
        };
        let dispatcher = HttpCloudDispatcher::from_config(&cfg);
        let start = dispatcher
            .start_sandbox(&pytxo_core::StartSandboxRequest {
                domain_id: "d".into(),
                run_id: "r".into(),
                agent_id: "a".into(),
                repo_fingerprint: "fp".into(),
            })
            .unwrap();
        assert_eq!(start.sandbox_id, "sb-1");
        server.join().unwrap();
    }

    #[test]
    fn execution_backend_parses_cloud() {
        assert_eq!(
            ExecutionBackend::parse("cloud"),
            Some(ExecutionBackend::Cloud)
        );
        let cfg = PytxoConfig {
            execution_backend: ExecutionBackend::Cloud,
            ..Default::default()
        };
        assert_eq!(cfg.execution_backend, ExecutionBackend::Cloud);
    }
}

#[cfg(not(feature = "cloud-http"))]
#[test]
fn cloud_http_feature_disabled_placeholder() {}
