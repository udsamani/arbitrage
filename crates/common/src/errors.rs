use thiserror::Error;

pub type ArbitrageResult<T> = Result<T, ArbitrageError>;

#[derive(Error, Debug)]
pub enum ArbitrageError {
    #[error("{0}")]
    JsonError(serde_json::Error),
    #[error("{0}")]
    Warning(String),
    #[error("{0}")]
    UnrecoverableError(String),
    #[error("exit")]
    Exit,
    #[error("{0}")]
    GenericError(String),
}
