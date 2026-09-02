use thiserror::Error;

#[derive(Debug, Error)]
pub enum PytxoError {
    #[error("configuration: {0}")]
    Config(String),

    #[error("scheduler: {0}")]
    Scheduler(String),

    #[error("runner: {0}")]
    Runner(String),

    #[error("store: {0}")]
    Store(String),

    #[error("billing: {0}")]
    Billing(String),

    /// A local policy boundary rejected data before cloud transport. Callers
    /// must never reinterpret this as a retryable transport outage.
    #[error("cloud policy: {0}")]
    CloudPolicy(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl PytxoError {
    pub fn is_cloud_policy_denial(&self) -> bool {
        matches!(self, Self::CloudPolicy(_))
    }
}

pub type Result<T> = std::result::Result<T, PytxoError>;
