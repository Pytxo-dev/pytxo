//! Optional Pytxo account for the CLI: browser sign-in with PKCE, shared
//! receipts, the public Pybench board. Nothing local depends on it.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SIGN_IN_TIMEOUT: Duration = Duration::from_secs(300);

/// Mirrors Link's free-account limit (`services/pytxo-link`, FREE_RECEIPT_LIMIT).
const FREE_RECEIPT_LIMIT: usize = 25;

fn web_url() -> String {
    std::env::var("PYTXO_WEB_URL").unwrap_or_else(|_| "https://pytxo.com".into())
}

fn link_url() -> String {
    std::env::var("PYTXO_LINK_URL").unwrap_or_else(|_| "https://link.pytxo.com".into())
}

#[derive(Debug, Deserialize, Serialize)]
struct StoredAccount {
    account_id: String,
    token: String,
    expires_at: String,
    link_url: String,
}

fn account_path() -> Result<PathBuf> {
    let home = std::env::var_os("PYTXO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
        .context("cannot find a home directory")?;
    Ok(home.join(".pytxo").join("account.json"))
}

fn load() -> Result<Option<StoredAccount>> {
    let path = account_path()?;
    match std::fs::read(&path) {
        Ok(bytes) => Ok(Some(
            serde_json::from_slice(&bytes).context("account.json is unreadable")?,
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn save(account: &StoredAccount) -> Result<()> {
    let path = account_path()?;
    std::fs::create_dir_all(path.parent().expect("has parent"))?;
    std::fs::write(&path, serde_json::to_vec_pretty(account)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn random_b64() -> Result<String> {
    let mut b = [0u8; 32];
    getrandom::getrandom(&mut b).map_err(|e| anyhow!("no randomness: {e}"))?;
    Ok(URL_SAFE_NO_PAD.encode(b))
}

fn open_browser(url: &str) {
    // Headless machines and SSH sessions: print the address only.
    if std::env::var_os("PYTXO_NO_BROWSER").is_some() {
        return;
    }
    #[cfg(windows)]
    let result = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();
    if result.is_err() {
        eprintln!("Could not open a browser. Open this address yourself:");
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("pytxo-cli/", env!("CARGO_PKG_VERSION")))
        .build()
}

pub fn login() -> Result<()> {
    let verifier = random_b64()?;
    let state = random_b64()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let listener = TcpListener::bind("127.0.0.1:0")
        .context("cannot listen on localhost for the sign-in callback")?;
    listener.set_nonblocking(true)?;
    let port = listener.local_addr()?.port();
    let url = format!(
        "{}/account/connect?client=cli&state={state}&code_challenge={challenge}&port={port}",
        web_url()
    );
    println!("Opening your browser to sign in to Pytxo. Local work never needs an account.");
    open_browser(&url);
    println!("{url}\nWaiting for the browser (5 minutes)…");

    let started = Instant::now();
    let (code, returned_state) = loop {
        if started.elapsed() > SIGN_IN_TIMEOUT {
            bail!("sign-in timed out; run `pytxo login` again");
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(false)?;
                let mut line = String::new();
                BufReader::new(&stream).read_line(&mut line)?;
                let path = line.split_whitespace().nth(1).unwrap_or("");
                let query = path.strip_prefix("/callback?").unwrap_or("");
                let mut code = None;
                let mut got_state = None;
                for pair in query.split('&') {
                    match pair.split_once('=') {
                        Some(("code", v)) => code = Some(v.to_string()),
                        Some(("state", v)) => got_state = Some(v.to_string()),
                        _ => {}
                    }
                }
                let ok = code.is_some() && got_state.as_deref() == Some(state.as_str());
                let body = if ok {
                    "<!doctype html><title>Pytxo</title><body style=\"font-family:system-ui;background:#09090b;color:#f5f5f7;display:grid;place-items:center;height:90vh\"><p>Signed in to the Pytxo CLI. You can close this tab.</p></body>"
                } else {
                    "<!doctype html><title>Pytxo</title><body style=\"font-family:system-ui\"><p>This sign-in link did not match. Run <code>pytxo login</code> again.</p></body>"
                };
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                if ok {
                    break (code.expect("checked"), got_state.expect("checked"));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100))
            }
            Err(e) => return Err(e.into()),
        }
    };

    let host = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "this machine".into());
    let response: serde_json::Value = agent()
        .post(&format!("{}/v1/account/exchange", link_url()))
        .send_json(serde_json::json!({
            "code": code,
            "state": returned_state,
            "code_verifier": verifier,
            "label": format!("cli on {host}"),
        }))
        .map_err(|e| anyhow!("sign-in exchange failed: {e}"))?
        .into_json()?;
    let account = StoredAccount {
        account_id: response["account_id"]
            .as_str()
            .context("no account id")?
            .into(),
        token: response["token"].as_str().context("no credential")?.into(),
        expires_at: response["expires_at"].as_str().unwrap_or_default().into(),
        link_url: link_url(),
    };
    save(&account)?;
    println!("Signed in. Perks: share receipts (`pytxo receipts share <id>`), keep them in your account, publish Pybench results (`pytxo bench --submit`).");
    Ok(())
}

pub fn logout() -> Result<()> {
    let Some(account) = load()? else {
        println!("Not signed in.");
        return Ok(());
    };
    let _ = agent()
        .delete(&format!("{}/v1/account/session", account.link_url))
        .set("Authorization", &format!("Bearer {}", account.token))
        .call();
    std::fs::remove_file(account_path()?)?;
    println!("Signed out. Local Pytxo keeps working the same.");
    Ok(())
}

pub fn whoami() -> Result<()> {
    let Some(account) = load()? else {
        println!("Not signed in. Everything local works without an account; `pytxo login` adds sharing perks.");
        return Ok(());
    };
    match agent()
        .get(&format!("{}/v1/account/session", account.link_url))
        .set("Authorization", &format!("Bearer {}", account.token))
        .call()
    {
        Ok(res) => {
            let session: serde_json::Value = res.into_json().unwrap_or_default();
            let plan = match session["plan"].as_str() {
                Some("pro") => "Pytxo Pro",
                _ => "free account",
            };
            println!(
                "Signed in as {} ({plan}; credential expires {}).",
                account.account_id, account.expires_at
            );
        }
        Err(ureq::Error::Status(401, _)) => {
            println!("Your sign-in expired or was revoked. Run `pytxo login`.")
        }
        Err(e) => println!(
            "Signed in as {} (could not reach Pytxo to confirm: {e}).",
            account.account_id
        ),
    }
    Ok(())
}

fn require() -> Result<StoredAccount> {
    load()?.context("this is an account perk: run `pytxo login` first (local work never needs it)")
}

fn visibility(private: bool) -> &'static str {
    if private {
        "?private=true"
    } else {
        ""
    }
}

pub fn share_receipt(payload: serde_json::Value, private: bool) -> Result<String> {
    let account = require()?;
    let res: serde_json::Value = agent()
        .post(&format!("{}/v1/receipts{}", account.link_url, visibility(private)))
        .set("Authorization", &format!("Bearer {}", account.token))
        .send_json(payload)
        .map_err(|e| match e {
            ureq::Error::Status(401, _) => anyhow!("sign-in expired; run `pytxo login`"),
            ureq::Error::Status(402, _) if private => anyhow!(
                "private receipts are a Pytxo Pro perk: {}/plans",
                web_url()
            ),
            ureq::Error::Status(402, _) => anyhow!(
                "free accounts keep {FREE_RECEIPT_LIMIT} shared receipts; delete one on {}/account or upgrade to Pro: {}/plans",
                web_url(),
                web_url()
            ),
            other => anyhow!("sharing failed: {other}"),
        })?
        .into_json()?;
    Ok(res["url"].as_str().unwrap_or_default().to_string())
}

pub fn submit_bench(report: &pytxo_land::bench::BenchReport, private: bool) -> Result<()> {
    let account = require()?;
    let land = report
        .land
        .as_ref()
        .filter(|l| l.skipped.is_none())
        .context("--submit needs the land suite (and node on PATH)")?;
    let s = &land.summary;
    let payload = serde_json::json!({
        "schema": "pybench.share/v1",
        "pytxo_version": report.pytxo_version,
        "created_at": report.created_at,
        "os": report.machine.os,
        "arch": report.machine.arch,
        "cpus": report.machine.cpus,
        "scenarios": s.scenarios,
        "correct": s.correct,
        "naive_landed_broken": s.naive_landed_broken,
        "pytxo_prevented": s.pytxo_prevented,
        "false_refusals": s.false_refusals,
        "median_gate_ms": s.median_gate_ms,
        "signal_reduction_pct": report.signal.as_ref().map(|g| g.weighted_reduction_pct),
    });
    agent()
        .post(&format!(
            "{}/v1/bench{}",
            account.link_url,
            visibility(private)
        ))
        .set("Authorization", &format!("Bearer {}", account.token))
        .send_json(payload)
        .map_err(|e| match e {
            ureq::Error::Status(402, _) => anyhow!(
                "private Pybench results are a Pytxo Pro perk: {}/plans",
                web_url()
            ),
            other => anyhow!("submitting failed: {other}"),
        })?;
    if private {
        println!(
            "Saved privately to your account at {}/account (not on the public board).",
            web_url()
        );
    } else {
        println!(
            "Published to {}/bench (summary numbers and machine type only).",
            web_url()
        );
    }
    Ok(())
}

/// Accepts `id` (prefix) of a receipt in this repository.
pub fn receipts_share(
    repo: Option<PathBuf>,
    id: &str,
    include_paths: bool,
    private: bool,
) -> Result<()> {
    let dir = crate::land_cmd::receipts_dir(repo)?;
    let (_, receipt) = pytxo_core::find_receipt(&dir, id)
        .with_context(|| format!("no single receipt matches {id}"))?;
    if !receipt.digest_matches() {
        bail!("receipt {id} was edited after it was written; refusing to share it");
    }
    let payload = receipt.share_payload(include_paths);
    println!("Sharing counts and outcomes only{}. Commands, refs, output and repository names stay local.", if include_paths { " plus file paths" } else { "" });
    let url = share_receipt(payload, private)?;
    if private {
        println!("{url} (private: only you can open it, signed in)");
    } else {
        println!("{url}");
    }
    Ok(())
}
