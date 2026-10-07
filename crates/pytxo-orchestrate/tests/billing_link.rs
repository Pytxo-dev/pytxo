//! Ultra billing reconcile integration (requires `link-http` on orchestrate).

#[cfg(feature = "link-http")]
mod link_http {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use pytxo_core::{BillingMode, DomainId, PytxoConfig, RunId};
    use pytxo_orchestrate::billing::setup_ultra_billing;
    use pytxo_store::SharedStore;
    use tempfile::TempDir;

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
    fn ultra_run_start_hits_link_http() {
        let hits = Arc::new(AtomicUsize::new(0));
        let hits2 = Arc::clone(&hits);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base = format!("http://127.0.0.1:{port}");

        let server = thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline && hits2.load(Ordering::SeqCst) == 0 {
                if let Ok((mut stream, _)) = listener.accept() {
                    let req = read_http_request(&mut stream);
                    if req.contains("runs/start") {
                        hits2.fetch_add(1, Ordering::SeqCst);
                    }
                    let resp = if req.contains("/health") {
                        "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
                    } else {
                        "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    };
                    stream.write_all(resp.as_bytes()).unwrap();
                    stream.flush().unwrap();
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });

        let tmp = TempDir::new().unwrap();
        let db = tmp.path().join("pytxo.db");
        let store = Arc::new(SharedStore::open(&db).unwrap());

        let mut cfg = PytxoConfig::default();
        cfg.billing.mode = BillingMode::Ultra;
        cfg.billing.link_reconcile = Some(true);
        cfg.billing.proxy_url = base;

        let domain = DomainId("/test/repo".into());
        let ultra = setup_ultra_billing(store, &cfg, domain.clone())
            .unwrap()
            .expect("ultra billing");
        let run_id = RunId("run-link-test".into());
        ultra.hybrid.on_run_start(&domain, &run_id).unwrap();

        server.join().unwrap();
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }
}

#[cfg(not(feature = "link-http"))]
#[test]
fn link_http_feature_disabled_placeholder() {}
