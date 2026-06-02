//! Print Signal Core scaffold stats for a file path (benchmark helper).

use std::env;
use std::path::PathBuf;

use pytxo_core::FidelityTier;
use pytxo_signal::scaffold_source;

fn main() -> anyhow::Result<()> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: scaffold_report <file> [low|medium|high]"))?;
    let tier = env::args()
        .nth(2)
        .and_then(|s| FidelityTier::parse(&s))
        .unwrap_or(FidelityTier::Low);

    let source = std::fs::read_to_string(&path)?;
    let result = scaffold_source(&path, &source, tier)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
