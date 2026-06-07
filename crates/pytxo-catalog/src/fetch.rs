use pytxo_core::{get_provider, list_static_models, ProviderId};
use serde::Deserialize;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
    pub provider: String,
}

pub fn fetch_models(provider: ProviderId) -> anyhow::Result<Vec<CatalogModel>> {
    let spec = get_provider(provider).ok_or_else(|| anyhow::anyhow!("unknown provider"))?;
    let pid = provider.as_str().to_string();

    if let Some(url) = spec.models_url {
        if provider == ProviderId::Ollama {
            return fetch_ollama(url, &pid);
        }
        if provider == ProviderId::Openrouter {
            return fetch_openrouter(url, spec.api_key_env, &pid);
        }
        if spec.openai_compatible && !spec.api_key_env.is_empty() {
            return fetch_openai_compatible(url, spec.api_key_env, &pid);
        }
    }

    Ok(list_static_models(provider)
        .into_iter()
        .map(|id| CatalogModel {
            id: id.clone(),
            name: id,
            provider: pid.clone(),
        })
        .collect())
}

fn fetch_openai_compatible(
    url: &str,
    key_env: &str,
    provider: &str,
) -> anyhow::Result<Vec<CatalogModel>> {
    let key = std::env::var(key_env).map_err(|_| anyhow::anyhow!("{key_env} not set"))?;
    let resp = ureq::get(url)
        .set("Authorization", &format!("Bearer {key}"))
        .call()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    #[derive(Deserialize)]
    struct Resp {
        data: Vec<ModelRow>,
    }
    #[derive(Deserialize)]
    struct ModelRow {
        id: String,
    }
    let body: Resp = resp.into_json().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(body
        .data
        .into_iter()
        .map(|m| CatalogModel {
            name: m.id.clone(),
            id: m.id,
            provider: provider.to_string(),
        })
        .collect())
}

fn fetch_openrouter(url: &str, key_env: &str, provider: &str) -> anyhow::Result<Vec<CatalogModel>> {
    let key = std::env::var(key_env).map_err(|_| anyhow::anyhow!("{key_env} not set"))?;
    let resp = ureq::get(url)
        .set("Authorization", &format!("Bearer {key}"))
        .call()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    #[derive(Deserialize)]
    struct Resp {
        data: Vec<OrRow>,
    }
    #[derive(Deserialize)]
    struct OrRow {
        id: String,
        name: Option<String>,
    }
    let body: Resp = resp.into_json().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(body
        .data
        .into_iter()
        .map(|m| CatalogModel {
            id: m.id.clone(),
            name: m.name.unwrap_or_else(|| m.id.clone()),
            provider: provider.to_string(),
        })
        .collect())
}

fn fetch_ollama(url: &str, provider: &str) -> anyhow::Result<Vec<CatalogModel>> {
    let resp = ureq::get(url).call().map_err(|e| anyhow::anyhow!("{e}"))?;
    #[derive(Deserialize)]
    struct Resp {
        models: Vec<OllamaRow>,
    }
    #[derive(Deserialize)]
    struct OllamaRow {
        name: String,
    }
    let body: Resp = resp.into_json().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(body
        .models
        .into_iter()
        .map(|m| CatalogModel {
            id: m.name.clone(),
            name: m.name,
            provider: provider.to_string(),
        })
        .collect())
}
