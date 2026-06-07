use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use pytxo_core::ProviderId;
use serde::{Deserialize, Serialize};

use crate::fetch::CatalogModel;

const TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    fetched_at: DateTime<Utc>,
    models: Vec<CatalogModel>,
}

pub struct ModelCache {
    dir: PathBuf,
}

impl ModelCache {
    pub fn open_default() -> anyhow::Result<Self> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(PathBuf::from)
            .map_err(|_| anyhow::anyhow!("cannot resolve home for model cache"))?;
        let dir = home.join(".pytxo").join("model-cache");
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    #[cfg(test)]
    pub fn open(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    fn path_for(&self, provider: ProviderId) -> PathBuf {
        self.dir.join(format!("{}.json", provider.as_str()))
    }

    pub fn get(&self, provider: ProviderId) -> anyhow::Result<Option<Vec<CatalogModel>>> {
        let path = self.path_for(provider);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)?;
        let entry: CacheEntry = serde_json::from_str(&raw)?;
        let age = Utc::now().signed_duration_since(entry.fetched_at);
        if age.to_std().unwrap_or(TTL) > TTL {
            return Ok(None);
        }
        Ok(Some(entry.models))
    }

    pub fn put(&self, provider: ProviderId, models: &[CatalogModel]) -> anyhow::Result<()> {
        let path = self.path_for(provider);
        let entry = CacheEntry {
            fetched_at: Utc::now(),
            models: models.to_vec(),
        };
        fs::write(path, serde_json::to_string_pretty(&entry)?)?;
        Ok(())
    }

    pub fn invalidate(&self, provider: ProviderId) -> anyhow::Result<()> {
        let path = self.path_for(provider);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cache = ModelCache::open(dir.path().to_path_buf());
        let models = vec![CatalogModel {
            id: "deepseek-chat".into(),
            name: "DeepSeek Chat".into(),
            provider: "deepseek".into(),
        }];
        cache.put(ProviderId::Deepseek, &models).unwrap();
        let got = cache.get(ProviderId::Deepseek).unwrap().unwrap();
        assert_eq!(got, models);
    }
}
