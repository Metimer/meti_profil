use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Arrow error: {0}")]
    Arrow(String),
    #[error("Unsupported source: {0}")]
    UnsupportedSource(String),
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<arrow::error::ArrowError> for Error {
    fn from(e: arrow::error::ArrowError) -> Self {
        Error::Arrow(e.to_string())
    }
}
