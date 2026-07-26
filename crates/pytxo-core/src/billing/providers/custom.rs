//! Optional custom OpenAI-compatible providers from `~/.pytxo/providers.json`.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CustomProviderSpec {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub api_key_env: String,
    #[serde(default)]
    pub openai_base_url: Option<String>,
    #[serde(default)]
    pub models_url: Option<String>,
    #[serde(default = "default_true")]
    pub openai_compatible: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct ProvidersFile {
    #[serde(default)]
    providers: Vec<CustomProviderSpec>,
}

fn pytxo_home() -> PathBuf {
    std::env::var_os("PYTXO_HOME")
        .or_else(|| std::env::var_os("HOME"))
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".pytxo")
}

pub fn providers_json_path() -> PathBuf {
    pytxo_home().join("providers.json")
}

pub fn load_custom_providers() -> Vec<CustomProviderSpec> {
    let path = providers_json_path();
    let Ok(raw) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let parsed: Result<ProvidersFile, _> = serde_json::from_str(&raw);
    let Ok(file) = parsed else {
        // Also accept a bare array.
        let Ok(list) = serde_json::from_str::<Vec<CustomProviderSpec>>(&raw) else {
            return Vec::new();
        };
        return normalize(list);
    };
    normalize(file.providers)
}

pub fn find_custom_provider(id: &str) -> Option<CustomProviderSpec> {
    let needle = id.to_ascii_lowercase();
    load_custom_providers()
        .into_iter()
        .find(|p| p.id.eq_ignore_ascii_case(&needle))
}

fn normalize(mut list: Vec<CustomProviderSpec>) -> Vec<CustomProviderSpec> {
    for p in &mut list {
        p.id = p.id.trim().to_ascii_lowercase();
        if p.display_name.trim().is_empty() {
            p.display_name = p.id.clone();
        }
    }
    list.retain(|p| !p.id.is_empty());
    list
}
