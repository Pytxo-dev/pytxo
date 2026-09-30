//! Test-only Claude wire shape for the routed owned-host integration seam.
//! The routed-test-faults feature cannot be included in a release build.

use std::io::Read;
use std::time::Duration;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.windows(2).any(|pair| pair == ["auth", "status"]) {
        println!(
            "{}",
            serde_json::json!({
                "loggedIn": true,
                "authMethod": "claude.ai",
                "subscriptionType": "local-test-fixture"
            })
        );
        return;
    }
    if !args.iter().any(|arg| arg == "--safe-mode")
        || !args.windows(2).any(|pair| pair == ["--tools", ""])
        || !args.iter().any(|arg| arg == "--json-schema")
    {
        std::process::exit(2);
    }
    let mut prompt = String::new();
    if std::io::stdin().read_to_string(&mut prompt).is_err() {
        std::process::exit(3);
    }
    std::thread::sleep(Duration::from_millis(500));
    let receipt_repair = prompt.contains("pytxo-test-claude-receipt-repair");
    let reviewed_prompt = prompt.split_once('\n').and_then(|(_, body)| {
        serde_json::Deserializer::from_str(body)
            .into_iter::<serde_json::Value>()
            .next()
            .and_then(Result::ok)
    });
    let exact_check_shape = reviewed_prompt.as_ref().is_some_and(|body| {
        let check = &body["previous"]["observed_check"];
        check["check_id"] == body["checks"][0]["id"]
            && check["exit_code"] == 1
            && check["receipt_digest"].as_str().is_some_and(|digest| {
                digest.len() == 64 && digest.chars().all(|ch| ch.is_ascii_hexdigit())
            })
    });
    let first_repair_check_fails = (prompt.contains("pytxo-test-claude-repair") || receipt_repair)
        && args.windows(2).any(|pair| pair == ["--model", "haiku"]);
    let content = if first_repair_check_fails {
        "pub fn value() -> i32 { 41 }\n"
    } else if receipt_repair {
        if exact_check_shape && !prompt.contains("hunter2") && !prompt.contains("EXPECTED: 42") {
            "pub fn value() -> i32 { 42 }\n"
        } else {
            "pub fn value() -> i32 { 41 }\n"
        }
    } else if prompt.contains("src/lib.rs") {
        "pub fn value() -> i32 { 42 }\n"
    } else {
        "changed\n"
    };
    println!(
        "{}",
        serde_json::json!({
            "is_error": false,
            "permission_denials": [],
            "structured_output": {"content": content}
        })
    );
}
