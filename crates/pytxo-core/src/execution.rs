use serde::{Deserialize, Serialize};

/// How the execution yard spawns agent processes ([[ADR-0010-pty-default-execution-backend]]).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionBackend {
    /// Managed pseudo-terminal via `portable-pty` (default).
    #[default]
    Pty,
    /// Piped stdout/stderr subprocess (CI / fallback).
    Subprocess,
    /// Remote Pytxo Cloud sandbox ([[cloud-sandbox-service]]).
    Cloud,
}

impl ExecutionBackend {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "pty" => Some(Self::Pty),
            "subprocess" => Some(Self::Subprocess),
            "cloud" => Some(Self::Cloud),
            _ => None,
        }
    }
}
