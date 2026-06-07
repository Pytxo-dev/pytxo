use std::io::Write;

use pytxo_catalog::ModelCatalog;
use pytxo_core::{all_providers, key_configured, ProviderId};

fn writeln_stdout(args: std::fmt::Arguments<'_>) -> bool {
    std::io::stdout()
        .write_fmt(format_args!("{args}\n"))
        .is_ok()
}

pub fn providers_list(json: bool) -> anyhow::Result<()> {
    let rows: Vec<_> = all_providers()
        .iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id.as_str(),
                "name": p.display_name,
                "api_key_env": p.api_key_env,
                "key_configured": key_configured(p),
            })
        })
        .collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for p in all_providers() {
            let mark = if key_configured(p) { "✓" } else { "·" };
            if !writeln_stdout(format_args!(
                "{mark} {:<12} {:<20} {}",
                p.id.as_str(),
                p.display_name,
                p.api_key_env
            )) {
                break;
            }
        }
    }
    Ok(())
}

pub fn models_list(provider: Option<&str>, refresh: bool, json: bool) -> anyhow::Result<()> {
    let catalog = ModelCatalog::open_default()?;
    let pid = provider
        .and_then(ProviderId::parse)
        .unwrap_or(ProviderId::Deepseek);
    let models = catalog.list(pid, refresh)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&models)?);
    } else {
        for m in &models {
            println!("{:<12} {}", m.provider, m.id);
        }
        println!("{} model(s)", models.len());
    }
    Ok(())
}

pub fn models_search(query: &str, provider: Option<&str>, json: bool) -> anyhow::Result<()> {
    let catalog = ModelCatalog::open_default()?;
    let pid = provider.and_then(ProviderId::parse);
    let models = catalog.search(query, pid, false)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&models)?);
    } else {
        for m in &models {
            println!("{:<12} {:<36} {}", m.provider, m.id, m.name);
        }
        println!("{} match(es)", models.len());
    }
    Ok(())
}

pub fn models_refresh(provider: &str) -> anyhow::Result<()> {
    let catalog = ModelCatalog::open_default()?;
    let pid = ProviderId::parse(provider).ok_or_else(|| anyhow::anyhow!("unknown provider"))?;
    let n = catalog.list(pid, true)?;
    println!("Refreshed {} models for {}", n.len(), provider);
    Ok(())
}
