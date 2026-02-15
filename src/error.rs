use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskVimError {
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, TaskVimError>;
