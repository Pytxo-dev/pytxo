//! Landing receipts: one record per landing decision (`pytxo land`) or
//! reviewed Apply, in the in-toto Statement v1 envelope so supply-chain tools
//! can read it. Unsigned in v1.3; `content_digest` makes tampering evident to
//! anyone who recomputes it, it does not prove who wrote the file.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

pub const RECEIPT_STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
pub const RECEIPT_PREDICATE_TYPE: &str = "https://pytxo.com/receipt/v1";
pub const RECEIPT_SHARE_SCHEMA: &str = "pytxo.receipt.share/v1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Receipt {
    #[serde(rename = "_type")]
    pub statement_type: String,
    pub subject: Vec<ReceiptSubject>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: ReceiptPredicate,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReceiptSubject {
    pub name: String,
    pub digest: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReceiptPredicate {
    pub id: String,
    /// `land` or `apply`.
    pub kind: String,
    pub created_at: String,
    pub pytxo_version: String,
    /// land: `ready | landed | blocked | stale`; apply: `applied | refused`.
    pub decision: String,
    /// `shadow` (nothing changed) or `apply` (the decision was carried out).
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(default)]
    pub inputs: Vec<ReceiptInput>,
    #[serde(default)]
    pub files_changed: usize,
    #[serde(default)]
    pub shared_files: Vec<ReceiptSharedFile>,
    #[serde(default)]
    pub checks: Vec<ReceiptCheck>,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub duration_ms: u64,
    /// sha256 of the canonical predicate JSON with this field empty.
    #[serde(default)]
    pub content_digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReceiptInput {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    pub commit: String,
    /// `merged | conflict | empty | applied`.
    pub status: String,
    pub files: Vec<String>,
    #[serde(default)]
    pub conflict_files: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReceiptSharedFile {
    pub path: String,
    pub inputs: Vec<String>,
    /// Named symbols changed by more than one input. Empty means the inputs
    /// touched separate symbols (or the language is not parsed).
    #[serde(default)]
    pub symbol_collisions: Vec<ReceiptSymbolCollision>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReceiptSymbolCollision {
    pub symbol: String,
    pub inputs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ReceiptCheck {
    /// `combined` or `input:<ref>`.
    pub scope: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub passed: bool,
    pub duration_ms: u64,
    #[serde(default)]
    pub timed_out: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_tail: Vec<String>,
    /// The second run of a combined check that failed while every input
    /// passed it alone. Absent (false) on every other run, so older receipts
    /// read and verify unchanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rerun: bool,
}

impl Receipt {
    pub fn new(
        predicate: ReceiptPredicate,
        subject_name: &str,
        subject_commit: Option<&str>,
    ) -> Self {
        let subject = subject_commit
            .map(|commit| ReceiptSubject {
                name: subject_name.to_string(),
                digest: [("gitCommit".to_string(), commit.to_string())].into(),
            })
            .into_iter()
            .collect();
        let mut receipt = Self {
            statement_type: RECEIPT_STATEMENT_TYPE.into(),
            subject,
            predicate_type: RECEIPT_PREDICATE_TYPE.into(),
            predicate,
        };
        receipt.predicate.content_digest = receipt.compute_digest();
        receipt
    }

    pub fn compute_digest(&self) -> String {
        let mut predicate = self.predicate.clone();
        predicate.content_digest.clear();
        // serde_json::Value maps are ordered (no preserve_order feature), so
        // this serialization is canonical for the field set.
        let value = serde_json::to_value(&predicate).expect("receipt predicate serializes");
        let bytes = serde_json::to_vec(&value).expect("receipt value serializes");
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    pub fn digest_matches(&self) -> bool {
        self.predicate.content_digest == self.compute_digest()
    }

    pub fn write_to(&self, dir: &Path) -> std::io::Result<PathBuf> {
        fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.json", self.predicate.id));
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        fs::write(&path, json)?;
        Ok(path)
    }

    /// What leaves the machine when a receipt is shared: counts and outcomes.
    /// Refs, commands, output and repository names never leave; file paths
    /// only with `include_paths`.
    pub fn share_payload(&self, include_paths: bool) -> serde_json::Value {
        let p = &self.predicate;
        let tally = |scope_combined: bool| {
            let checks: Vec<_> = p
                .checks
                .iter()
                .filter(|c| (c.scope == "combined") == scope_combined)
                .collect();
            serde_json::json!({
                "passed": checks.iter().filter(|c| c.passed).count(),
                "total": checks.len(),
            })
        };
        let mut agents: Vec<String> = p.inputs.iter().filter_map(|i| i.agent.clone()).collect();
        agents.sort();
        agents.dedup();
        let mut payload = serde_json::json!({
            "schema": RECEIPT_SHARE_SCHEMA,
            "id": p.id,
            "kind": p.kind,
            "decision": p.decision,
            "mode": p.mode,
            "created_at": p.created_at,
            "pytxo_version": p.pytxo_version,
            "inputs": p.inputs.len(),
            "agents": agents,
            "files_changed": p.files_changed,
            "shared_files": p.shared_files.len(),
            "symbol_collisions": p.shared_files.iter().map(|f| f.symbol_collisions.len()).sum::<usize>(),
            "conflicts": p.inputs.iter().filter(|i| i.status == "conflict").count(),
            "checks": { "combined": tally(true), "per_input": tally(false) },
            "duration_ms": p.duration_ms,
            "content_digest": p.content_digest,
        });
        if include_paths {
            let mut paths: Vec<&str> = p
                .inputs
                .iter()
                .flat_map(|i| i.files.iter().map(String::as_str))
                .collect();
            paths.sort_unstable();
            paths.dedup();
            payload["paths"] = serde_json::json!(paths);
        }
        payload
    }
}

pub fn load_receipt(path: &Path) -> std::io::Result<Receipt> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(std::io::Error::other)
}

/// Newest first. Unreadable files are skipped, not fatal.
pub fn list_receipts(dir: &Path) -> Vec<(PathBuf, Receipt)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| load_receipt(&p).ok().map(|r| (p, r)))
        .collect();
    out.sort_by(|a, b| b.1.predicate.created_at.cmp(&a.1.predicate.created_at));
    out
}

/// Find by full id or unique prefix.
pub fn find_receipt(dir: &Path, id: &str) -> Option<(PathBuf, Receipt)> {
    let mut hits: Vec<_> = list_receipts(dir)
        .into_iter()
        .filter(|(_, r)| r.predicate.id == id || r.predicate.id.starts_with(id))
        .collect();
    (hits.len() == 1).then(|| hits.remove(0))
}

/// `rcpt_` + UTC timestamp + 8 random hex: sortable, URL-safe, unguessable
/// enough for a local file name. Public share ids are issued by the server.
pub fn new_receipt_id() -> String {
    let now = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let rand = uuid::Uuid::new_v4().simple().to_string();
    format!("rcpt_{now}_{}", &rand[..8])
}

/// Receipts live in the repository's git common dir (`.git/pytxo/receipts`):
/// never tracked, shared by every worktree, no `.gitignore` edits.
pub fn receipts_dir(git_common_dir: &Path) -> PathBuf {
    git_common_dir.join("pytxo").join("receipts")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Receipt {
        Receipt::new(
            ReceiptPredicate {
                id: "rcpt_20261010T000000Z_deadbeef".into(),
                kind: "land".into(),
                created_at: "2026-10-10T00:00:00Z".into(),
                pytxo_version: "1.3.0".into(),
                decision: "blocked".into(),
                mode: "shadow".into(),
                target: Some("main".into()),
                base: "a".repeat(40),
                result: Some("b".repeat(40)),
                inputs: vec![ReceiptInput {
                    reference: "agent/a".into(),
                    agent: Some("codex".into()),
                    commit: "c".repeat(40),
                    status: "merged".into(),
                    files: vec!["src/secret-name.js".into()],
                    conflict_files: vec![],
                }],
                files_changed: 1,
                shared_files: vec![],
                checks: vec![ReceiptCheck {
                    scope: "combined".into(),
                    command: "npm test -- --token=abc".into(),
                    exit_code: Some(1),
                    passed: false,
                    duration_ms: 12,
                    timed_out: false,
                    output_tail: vec![
                        "FAIL".into(),
                        r"npm error log: C:\Users\someone\AppData\npm\debug.log".into(),
                        "at /home/someone/repo/test.js:3".into(),
                    ],
                    rerun: false,
                }],
                reasons: vec!["combined check failed".into()],
                duration_ms: 40,
                content_digest: String::new(),
            },
            "git:main",
            Some(&"b".repeat(40)),
        )
    }

    #[test]
    fn digest_detects_edits_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let receipt = sample();
        assert!(receipt.digest_matches());
        // Fields added after v1 are left out when unset, so older receipts verify.
        assert!(!serde_json::to_string(&receipt).unwrap().contains("rerun"));
        let path = receipt.write_to(dir.path()).unwrap();
        let loaded = load_receipt(&path).unwrap();
        assert_eq!(loaded, receipt);
        let mut edited = loaded.clone();
        edited.predicate.decision = "ready".into();
        assert!(!edited.digest_matches());
        assert_eq!(find_receipt(dir.path(), "rcpt_2026").unwrap().1, receipt);
    }

    #[test]
    fn share_payload_never_carries_commands_refs_or_output() {
        let payload = sample().share_payload(false).to_string();
        // Check output (including user-profile paths) stays local.
        let local = ["npm test", "token=abc", "agent/a", "secret-name", "FAIL"];
        for secret in local.iter().chain(&["someone", "Users", "/home/"]) {
            assert!(!payload.contains(secret), "{secret} leaked: {payload}");
        }
        assert!(payload.contains("\"agents\":[\"codex\"]"));
        let with_paths = sample().share_payload(true).to_string();
        assert!(with_paths.contains("secret-name.js"));
        for secret in ["npm test", "FAIL", "someone", "Users", "/home/"] {
            assert!(
                !with_paths.contains(secret),
                "{secret} leaked: {with_paths}"
            );
        }
    }
}
