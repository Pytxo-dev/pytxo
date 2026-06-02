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

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, PytxoError>;
