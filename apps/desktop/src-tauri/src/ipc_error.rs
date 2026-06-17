use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct PytxoIpcError {
    pub code: String,
    pub message: String,
}

impl PytxoIpcError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn from_err(code: &str, err: impl ToString) -> Self {
        Self::new(code, err.to_string())
    }
}

pub type IpcResult<T> = Result<T, PytxoIpcError>;

pub fn map_store_err(e: impl ToString) -> PytxoIpcError {
    PytxoIpcError::from_err("store", e)
}

pub fn map_config_err(e: impl ToString) -> PytxoIpcError {
    PytxoIpcError::from_err("config", e)
}

pub fn map_orch_err(e: impl ToString) -> PytxoIpcError {
    PytxoIpcError::from_err("orchestrate", e)
}

pub fn map_lock_err(e: impl ToString) -> PytxoIpcError {
    PytxoIpcError::from_err("lock", e)
}

pub fn map_io_err(e: impl ToString) -> PytxoIpcError {
    PytxoIpcError::from_err("io", e)
}
