//! Print Signal Core scaffold stats for a file path (benchmark helper).

use std::env;
use std::path::PathBuf;

use pytxo_core::FidelityTier;
use pytxo_signal::scaffold_source;

fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let path = args.first().map(PathBuf::from).ok_or_else(|| {
        anyhow::anyhow!("usage: scaffold_report <file> [low|medium|high] [--stats-only]")
    })?;
    let tier = args
        .get(1)
        .and_then(|s| FidelityTier::parse(s))
        .unwrap_or(FidelityTier::Low);
    let stats_only = args.iter().any(|arg| arg == "--stats-only");

    let source = std::fs::read_to_string(&path)?;
    let result = scaffold_source(&path, &source, tier)?;
    if stats_only {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "path": result.path,
                "stats": result.stats,
                "fallback_raw": result.fallback_raw,
            }))?
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
}
