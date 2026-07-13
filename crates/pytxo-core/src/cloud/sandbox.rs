use serde::{Deserialize, Serialize};

use crate::cloud::CloudConfig;
use crate::{PytxoError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartSandboxRequest {
    pub domain_id: String,
    pub run_id: String,
    pub agent_id: String,
    pub repo_fingerprint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartSandboxResponse {
    pub sandbox_id: String,
    pub ws_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncFile {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecRequest {
    pub cmd: String,
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CloudDispatcher: Send + Sync {
    fn start_sandbox(&self, req: &StartSandboxRequest) -> Result<StartSandboxResponse>;
    fn sync_delta(&self, sandbox_id: &str, files: &[SyncFile]) -> Result<()>;
    fn exec(&self, sandbox_id: &str, req: &ExecRequest) -> Result<ExecResponse>;
    fn teardown(&self, sandbox_id: &str) -> Result<()>;
}

#[derive(Clone, Debug, Default)]
pub struct NoopCloudDispatcher;

impl CloudDispatcher for NoopCloudDispatcher {
    fn start_sandbox(&self, _: &StartSandboxRequest) -> Result<StartSandboxResponse> {
        Err(PytxoError::Other("cloud dispatcher not configured".into()))
    }
    fn sync_delta(&self, _: &str, _: &[SyncFile]) -> Result<()> {
        Ok(())
    }
    fn exec(&self, _: &str, _: &ExecRequest) -> Result<ExecResponse> {
        Err(PytxoError::Other("cloud dispatcher not configured".into()))
    }
    fn teardown(&self, _: &str) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct HttpCloudDispatcher {
    pub base_url: String,
}

impl HttpCloudDispatcher {
    pub fn from_config(cfg: &CloudConfig) -> Self {
        Self {
            base_url: cfg.sandbox_url.trim_end_matches('/').to_string(),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn auth_headers(&self) -> Vec<(String, String)> {
        let mut h = vec![("Content-Type".to_string(), "application/json".to_string())];
        if let Ok(token) = std::env::var("PYTXO_CLOUD_SESSION") {
            if !token.is_empty() {
                h.push(("Authorization".to_string(), format!("Bearer {token}")));
            }
        }
        h
    }

    fn send_json<T: Serialize>(&self, method: &str, url: &str, body: Option<&T>) -> Result<String> {
        self.ping()?;
        #[cfg(feature = "cloud-http")]
        {
            let agent = ureq::AgentBuilder::new().build();
            let mut req = match method {
                "POST" => agent.post(url),
                "DELETE" => agent.delete(url),
                _ => agent.get(url),
            };
            for (k, v) in self.auth_headers() {
                req = req.set(&k, &v);
            }
            let resp = if let Some(b) = body {
                req.send_json(b)
            } else {
                req.call()
            }
            .map_err(|e| PytxoError::Other(format!("cloud http: {e}")))?;
            let status = resp.status();
            let text = resp.into_string().unwrap_or_default();
            if !(200..300).contains(&status) {
                return Err(PytxoError::Other(format!(
                    "cloud http {url}: status {status} {text}"
                )));
            }
            Ok(text)
        }
        #[cfg(not(feature = "cloud-http"))]
        {
            let _ = (method, url, body);
            Ok(String::new())
        }
    }

    pub fn ping(&self) -> Result<()> {
        if self.base_url.is_empty() {
            return Err(PytxoError::Other("cloud base_url empty".into()));
        }
        Ok(())
    }

    pub fn ping_health(&self) -> Result<()> {
        self.ping()?;
        #[cfg(feature = "cloud-http")]
        {
            let url = if self.base_url.ends_with("/v1") {
                format!("{}/health", self.base_url.trim_end_matches("/v1"))
            } else {
                format!("{}/health", self.base_url.trim_end_matches('/'))
            };
            let agent = ureq::AgentBuilder::new().build();
            let mut req = agent.get(&url);
            for (k, v) in self.auth_headers() {
                req = req.set(&k, &v);
            }
            let resp = req
                .call()
                .map_err(|e| PytxoError::Other(format!("cloud health: {e}")))?;
            if (200..300).contains(&resp.status()) {
                Ok(())
            } else {
                Err(PytxoError::Other(format!(
                    "cloud health: status {}",
                    resp.status()
                )))
            }
        }
        #[cfg(not(feature = "cloud-http"))]
        {
            Ok(())
        }
    }
}

impl CloudDispatcher for HttpCloudDispatcher {
    fn start_sandbox(&self, req: &StartSandboxRequest) -> Result<StartSandboxResponse> {
        let url = self.endpoint("sandboxes/start");
        let text = self.send_json("POST", &url, Some(req))?;
        serde_json::from_str(&text).map_err(|e| PytxoError::Other(format!("cloud parse: {e}")))
    }

    fn sync_delta(&self, sandbox_id: &str, files: &[SyncFile]) -> Result<()> {
        let url = self.endpoint(&format!("sandboxes/{sandbox_id}/sync"));
        self.send_json("POST", &url, Some(&serde_json::json!({ "files": files })))?;
        Ok(())
    }

    fn exec(&self, sandbox_id: &str, req: &ExecRequest) -> Result<ExecResponse> {
        let url = self.endpoint(&format!("sandboxes/{sandbox_id}/exec"));
        let text = self.send_json("POST", &url, Some(req))?;
        serde_json::from_str(&text).map_err(|e| PytxoError::Other(format!("cloud parse: {e}")))
    }

    fn teardown(&self, sandbox_id: &str) -> Result<()> {
        let url = self.endpoint(&format!("sandboxes/{sandbox_id}"));
        self.send_json("DELETE", &url, None::<&serde_json::Value>)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_errors_on_exec() {
        let n = NoopCloudDispatcher;
        assert!(n
            .exec(
                "x",
                &ExecRequest {
                    cmd: "echo".into(),
                    cwd: None,
                    env: std::collections::HashMap::new(),
                }
            )
            .is_err());
    }
}
