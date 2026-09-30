use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Keyring error: {0}")]
    KeyringError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}
