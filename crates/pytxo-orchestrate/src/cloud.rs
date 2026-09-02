use pytxo_core::{CloudConfig, HttpCloudDispatcher};

pub struct CloudClients {
    pub dispatcher: std::sync::Arc<dyn pytxo_core::CloudDispatcher>,
    pub cache: std::sync::Arc<dyn pytxo_core::ContextCache>,
}

#[cfg(feature = "cloud-http")]
fn out_of_band_upload_consent() -> bool {
    std::env::var("PYTXO_CLOUD_UPLOAD_CONSENT")
        .is_ok_and(|value| value == "I_UNDERSTAND_REPOSITORY_CONTENT_WILL_BE_UPLOADED")
}

pub(crate) fn out_of_band_local_fallback_consent() -> bool {
    std::env::var("PYTXO_CLOUD_FALLBACK_LOCAL")
        .is_ok_and(|value| value == "I_UNDERSTAND_CLOUD_FAILURE_WILL_RUN_LOCALLY")
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
        let upload_consent = out_of_band_upload_consent();
        CloudClients {
            dispatcher: std::sync::Arc::new(HttpCloudDispatcher::from_config_with_upload_consent(
                &cfg.cloud,
                upload_consent,
            )),
            cache: if cfg.cloud.cache_enabled {
                std::sync::Arc::new(HttpContextCache::from_config_with_upload_consent(
                    &cfg.cloud,
                    upload_consent,
                ))
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
