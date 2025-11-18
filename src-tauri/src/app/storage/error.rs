use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration not found")]
    NotFound,

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Migration error: {0}")]
    Migration(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
