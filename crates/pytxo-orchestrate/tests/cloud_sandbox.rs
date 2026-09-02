#[cfg(feature = "cloud-http")]
mod cloud_http {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use pytxo_core::{
        content_hash, CachePut, CloudConfig, CloudDispatcher, ContextCache, ExecutionBackend,
        HttpCloudDispatcher, HttpContextCache, PytxoConfig, SyncFile,
    };
    use pytxo_orchestrate::cloud::ping_cloud;

    fn read_http_request(stream: &mut std::net::TcpStream) -> String {
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = Vec::new();
        loop {
            let mut chunk = [0u8; 4096];
            let n = stream.read(&mut chunk).unwrap();
            if n == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..n]);
            let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if request.len() >= header_end + 4 + content_length {
                break;
            }
        }
        String::from_utf8(request).unwrap()
    }

    #[test]
    fn cloud_health_ping() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let hits = Arc::new(AtomicUsize::new(0));
        let hits2 = Arc::clone(&hits);
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let _ = ready_tx.send(());
            listener.set_nonblocking(true).ok();
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                if let Ok((mut stream, _)) = listener.accept() {
                    let request = read_http_request(&mut stream);
                    if request.contains("/health") {
                        hits2.fetch_add(1, Ordering::SeqCst);
                    }
                    let body = "ok";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        ready_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("mock health server ready");

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
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let _ = ready_tx.send(());
            listener.set_nonblocking(true).ok();
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                if let Ok((mut stream, _)) = listener.accept() {
                    let req = read_http_request(&mut stream);
                    let body = if req.contains("sandboxes/start") {
                        r#"{"sandbox_id":"sb-1","ws_url":null}"#
                    } else {
                        "{}"
                    };
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        ready_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("mock sandbox server ready");

        let cfg = CloudConfig {
            upload_consent: true,
            sandbox_url: format!("http://127.0.0.1:{port}/v1"),
            ..Default::default()
        };
        let dispatcher = HttpCloudDispatcher::from_config_with_upload_consent(&cfg, true);
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
    fn cloud_sync_uploads_allowed_source_with_hash_manifest() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (request_tx, request_rx) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            request_tx.send(read_http_request(&mut stream)).unwrap();
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        });
        let cfg = CloudConfig {
            enabled: true,
            upload_consent: true,
            sandbox_url: format!("http://127.0.0.1:{port}/v1"),
            ..Default::default()
        };
        let dispatcher = HttpCloudDispatcher::from_config_with_upload_consent(&cfg, true);
        let source = "pub fn allowed() {}\n";
        dispatcher
            .sync_delta(
                "sb-allowed",
                &[SyncFile {
                    path: "src/lib.rs".into(),
                    content: source.into(),
                }],
            )
            .unwrap();

        let request = request_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        server.join().unwrap();
        assert!(request.contains("/v1/sandboxes/sb-allowed/sync"));
        assert!(request.contains("src/lib.rs"));
        assert!(request.contains(&content_hash(source.as_bytes())));
        assert!(request.contains("manifest"));
    }

    #[test]
    fn cloud_cache_uploads_allowed_source_after_consent() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (request_tx, request_rx) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            request_tx.send(read_http_request(&mut stream)).unwrap();
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        });
        let cfg = CloudConfig {
            upload_consent: true,
            cache_url: format!("http://127.0.0.1:{port}/v1"),
            ..Default::default()
        };
        let cache = HttpContextCache::from_config_with_upload_consent(&cfg, true);
        let source = "pub fn allowed() {}\n";
        cache
            .put(&CachePut {
                domain_id: "domain".into(),
                path: "src/lib.rs".into(),
                fidelity: "low".into(),
                content_hash: content_hash(source.as_bytes()),
                content: source.into(),
                scaffolded_bytes: source.len(),
                language: Some("rs".into()),
            })
            .unwrap();

        let request = request_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        server.join().unwrap();
        assert!(request.contains("PUT /v1/cache/scaffold"));
        assert!(request.contains("src/lib.rs"));
        assert!(request.contains("pub fn allowed"));
    }

    #[test]
    fn denied_paths_and_secret_content_never_reach_mock_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let cfg = CloudConfig {
            upload_consent: true,
            sandbox_url: format!("http://127.0.0.1:{port}/v1"),
            cache_url: format!("http://127.0.0.1:{port}/v1"),
            ..Default::default()
        };
        let dispatcher = HttpCloudDispatcher::from_config_with_upload_consent(&cfg, true);
        let cache = HttpContextCache::from_config_with_upload_consent(&cfg, true);

        assert!(dispatcher
            .sync_delta(
                "sandbox",
                &[SyncFile {
                    path: ".env".into(),
                    content: "OPENAI_API_KEY=not-uploaded".into(),
                }],
            )
            .is_err());
        assert!(dispatcher
            .sync_delta(
                "sandbox",
                &[SyncFile {
                    path: "src/config.txt".into(),
                    content: "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456".into(),
                }],
            )
            .is_err());
        assert!(cache
            .put(&CachePut {
                domain_id: "domain".into(),
                path: "nested/private-key.pem".into(),
                fidelity: "low".into(),
                content_hash: content_hash(b"private"),
                content: "private".into(),
                scaffolded_bytes: 7,
                language: None,
            })
            .is_err());
        assert!(matches!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        ));
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
