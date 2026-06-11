use thiserror::Error;

#[derive(Debug, Error)]
pub enum HarError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Entry {index} not found (file has {total} entries)")]
    IndexOutOfRange { index: usize, total: usize },
    #[error("Invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),
    #[error("{0}")]
    Usage(String),
}

pub type Result<T> = std::result::Result<T, HarError>;
