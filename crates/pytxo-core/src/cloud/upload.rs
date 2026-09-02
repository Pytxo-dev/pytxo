use std::path::{Component, Path};
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::cloud::{content_hash, SyncFile};
use crate::{PytxoError, Result};

/// Hash-bound description of the exact files proposed for one cloud sync.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudSyncManifest {
    pub files: Vec<CloudSyncManifestEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudSyncManifestEntry {
    pub path: String,
    pub content_hash: String,
    pub bytes: usize,
}

static SECRET_PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        (
            "private key",
            Regex::new(r"-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----")
                .expect("private key pattern"),
        ),
        (
            "AWS access key",
            Regex::new(r"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b").expect("AWS key pattern"),
        ),
        (
            "GitHub token",
            Regex::new(r"\b(?:gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")
                .expect("GitHub token pattern"),
        ),
        (
            "OpenAI or Anthropic token",
            Regex::new(r"\bsk-(?:ant-|proj-|svcacct-)?[A-Za-z0-9_-]{20,}\b")
                .expect("provider token pattern"),
        ),
        (
            "provider token",
            Regex::new(r"\b(?:xox[baprs]-[A-Za-z0-9-]{16,}|AIza[0-9A-Za-z_-]{30,}|sk_live_[0-9A-Za-z]{16,})\b")
                .expect("provider token pattern"),
        ),
        (
            "bearer token",
            Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9._~-]{16,}\b")
                .expect("bearer token pattern"),
        ),
        (
            "database credential URL",
            Regex::new(r"(?i)\b(?:postgres(?:ql)?|mysql|mongodb(?:\+srv)?|redis|rediss)://[^\s:/]+:[^\s/@]+@")
                .expect("database URL pattern"),
        ),
    ]
});

static CREDENTIAL_ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|secret(?:[_-]?key)?|password)\s*[:=]\s*(?P<value>[^\s,;]+)"#,
    )
    .expect("credential assignment pattern")
});

/// Returns true for repository paths that must never leave the machine.
///
/// This denylist is independent of user sparse-exclude settings and applies to
/// both initial sync, overlay deltas, and context-cache writes.
pub fn cloud_path_denied(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return true;
    }

    let components = normalized
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    if components.iter().any(|part| {
        matches!(
            part.as_str(),
            ".git" | ".pytxo" | ".ssh" | ".aws" | ".azure" | ".gnupg"
        )
    }) {
        return true;
    }

    let Some(file_name) = components.last().map(String::as_str) else {
        return true;
    };
    if file_name == ".env"
        || file_name.starts_with(".env.")
        || matches!(
            file_name,
            ".git-credentials"
                | ".netrc"
                | ".npmrc"
                | ".pypirc"
                | "credentials"
                | "credentials.json"
                | "secrets.json"
                | "token"
                | "tokens"
                | "id_rsa"
                | "id_dsa"
                | "id_ecdsa"
                | "id_ed25519"
        )
        || file_name.ends_with(".pem")
        || file_name.ends_with(".key")
        || file_name.ends_with(".p12")
        || file_name.ends_with(".pfx")
        || file_name.ends_with(".jks")
        || file_name.ends_with(".keystore")
        || file_name.contains("private_key")
        || file_name.contains("private-key")
        || file_name.starts_with("service-account")
        || file_name.starts_with("service_account")
    {
        return true;
    }

    components.windows(2).any(|parts| {
        matches!(parts, [config, provider] if config == ".config" && provider == "gcloud")
            || matches!(parts, [docker, config] if docker == ".docker" && config == "config.json")
    })
}

/// Reject a likely secret instead of redacting source and changing its semantics.
pub fn validate_cloud_content(path: &str, content: &str) -> Result<()> {
    for (kind, pattern) in SECRET_PATTERNS.iter() {
        if pattern.is_match(content) {
            return Err(PytxoError::CloudPolicy(format!(
                "cloud upload denied: likely {kind} in {path}"
            )));
        }
    }
    for capture in CREDENTIAL_ASSIGNMENT.captures_iter(content) {
        let raw = capture
            .name("value")
            .map(|value| value.as_str())
            .unwrap_or("");
        let quoted = raw.starts_with(['\'', '"']);
        let value = raw.trim_matches(['\'', '"', ')', ']', '}']);
        let lower = value.to_ascii_lowercase();
        let is_reference_or_placeholder = [
            "process.env",
            "std::env",
            "os.environ",
            "getenv",
            "response.",
            "example",
            "placeholder",
            "changeme",
            "redacted",
            "dummy",
            "your_",
            "${",
        ]
        .iter()
        .any(|marker| lower.contains(marker));
        let looks_literal = value.len() >= 16
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "_./+~-".contains(character))
            && (quoted
                || value.chars().any(|character| character.is_ascii_digit())
                || value
                    .chars()
                    .any(|character| character.is_ascii_uppercase()));
        if looks_literal && !is_reference_or_placeholder {
            return Err(PytxoError::CloudPolicy(format!(
                "cloud upload denied: likely credential assignment in {path}"
            )));
        }
    }
    Ok(())
}

/// Validate a single outbound file. Denied paths are errors at the final egress
/// boundary even though repository collectors omit them earlier.
pub fn validate_cloud_upload(path: &str, content: &str) -> Result<()> {
    if cloud_path_denied(path) {
        return Err(PytxoError::CloudPolicy(format!(
            "cloud upload denied for protected path: {path}"
        )));
    }
    validate_cloud_content(path, content)
}

pub fn cloud_sync_manifest(files: &[SyncFile]) -> Result<CloudSyncManifest> {
    let mut entries = Vec::with_capacity(files.len());
    for file in files {
        validate_cloud_upload(&file.path, &file.content)?;
        entries.push(CloudSyncManifestEntry {
            path: file.path.clone(),
            content_hash: content_hash(file.content.as_bytes()),
            bytes: file.content.len(),
        });
    }
    Ok(CloudSyncManifest { files: entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_secret_and_hidden_control_paths() {
        for path in [
            ".env",
            ".env.production",
            ".pytxo/data/run.db",
            ".git/config",
            "nested/.ssh/id_ed25519",
            "config/private-key.pem",
            "nested/credentials.json",
            ".config/gcloud/application_default_credentials.json",
        ] {
            assert!(cloud_path_denied(path), "expected {path} to be denied");
        }
        assert!(!cloud_path_denied("src/token.rs"));
        assert!(!cloud_path_denied(".github/workflows/test.yml"));
    }

    #[test]
    fn rejects_representative_secret_content() {
        for content in [
            "OPENAI_API_KEY=sk-proj-abcdefghijklmnopqrstuvwxyz1234567890",
            "ANTHROPIC_API_KEY=sk-ant-abcdefghijklmnopqrstuvwxyz1234567890",
            "GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456",
            "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE",
            "DATABASE_URL=postgres://user:verysecretpassword@db.example/app",
            "-----BEGIN OPENSSH PRIVATE KEY-----\nabc",
        ] {
            assert!(
                validate_cloud_content("src/config.txt", content).is_err(),
                "expected representative secret to be rejected"
            );
        }
    }

    #[test]
    fn allowed_source_has_hash_bound_manifest() {
        let files = vec![SyncFile {
            path: "src/lib.rs".into(),
            content: concat!(
                "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
                "let access_token = response.access_token;\n",
                "let password = std::env::var(\"DATABASE_PASSWORD\");\n",
            )
            .into(),
        }];
        let manifest = cloud_sync_manifest(&files).unwrap();
        assert_eq!(manifest.files[0].path, "src/lib.rs");
        assert_eq!(manifest.files[0].bytes, files[0].content.len());
        assert_eq!(
            manifest.files[0].content_hash,
            content_hash(files[0].content.as_bytes())
        );
    }
}
