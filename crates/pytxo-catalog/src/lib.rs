mod cache;
mod fetch;
mod search;

pub use cache::ModelCache;
pub use fetch::{fetch_models, CatalogModel};
pub use search::search_models;

use pytxo_core::ProviderId;

pub struct ModelCatalog {
    cache: ModelCache,
}

impl ModelCatalog {
    pub fn open_default() -> anyhow::Result<Self> {
        Ok(Self {
            cache: ModelCache::open_default()?,
        })
    }

    pub fn list(&self, provider: ProviderId, refresh: bool) -> anyhow::Result<Vec<CatalogModel>> {
        if !refresh {
            if let Some(cached) = self.cache.get(provider)? {
                return Ok(cached);
            }
        }
        let models = fetch_models(provider)?;
        self.cache.put(provider, &models)?;
        Ok(models)
    }

    pub fn search(
        &self,
        query: &str,
        provider: Option<ProviderId>,
        refresh: bool,
    ) -> anyhow::Result<Vec<CatalogModel>> {
        let providers: Vec<ProviderId> = if let Some(p) = provider {
            vec![p]
        } else {
            pytxo_core::all_providers()
                .iter()
                .map(|s| s.id)
                .filter(|id| *id != ProviderId::Generic)
                .collect()
        };
        let mut all = Vec::new();
        for p in providers {
            if let Ok(mut m) = self.list(p, refresh) {
                all.append(&mut m);
            }
        }
        Ok(search_models(&all, query))
    }
}
