use pytxo_core::{CloudConfig, HttpCloudDispatcher};

pub struct CloudClients {
    pub dispatcher: std::sync::Arc<dyn pytxo_core::CloudDispatcher>,
    pub cache: std::sync::Arc<dyn pytxo_core::ContextCache>,
}

pub fn cloud_clients(cfg: &pytxo_core::PytxoConfig) -> CloudClients {
    use pytxo_core::{ExecutionBackend, HttpContextCache, NoopCloudDispatcher, NoopContextCache};
    let use_cloud = cfg.cloud.enabled || cfg.execution_backend == ExecutionBackend::Cloud;
    if !use_cloud {
        return CloudClients {
            dispatcher: std::sync::Arc::new(NoopCloudDispatcher),
            cache: std::sync::Arc::new(NoopContextCache),
        };
    }
    #[cfg(feature = "cloud-http")]
    {
        CloudClients {
            dispatcher: std::sync::Arc::new(HttpCloudDispatcher::from_config(&cfg.cloud)),
            cache: if cfg.cloud.cache_enabled {
                std::sync::Arc::new(HttpContextCache::from_config(&cfg.cloud))
            } else {
                std::sync::Arc::new(NoopContextCache)
            },
        }
    }
    #[cfg(not(feature = "cloud-http"))]
    {
        let _ = cfg;
        CloudClients {
            dispatcher: std::sync::Arc::new(NoopCloudDispatcher),
            cache: std::sync::Arc::new(NoopContextCache),
        }
    }
}

pub fn cloud_health_url(cfg: &CloudConfig) -> String {
    let base = cfg.sandbox_url.trim_end_matches('/');
    if let Some(stripped) = base.strip_suffix("/v1") {
        format!("{stripped}/health")
    } else {
        format!("{base}/health")
    }
}

pub fn ping_cloud(cfg: &CloudConfig) -> Result<(), String> {
    if !cfg.enabled && cfg.sandbox_url.is_empty() {
        return Ok(());
    }
    HttpCloudDispatcher::from_config(cfg)
        .ping_health()
        .map_err(|e| e.to_string())
}
