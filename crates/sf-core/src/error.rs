/// Core error type for Stats Workbench.
///
/// External crate errors are converted at boundaries via `map_err`.
#[derive(thiserror::Error, Debug)]
pub enum SfError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("source error: {0}")]
    Source(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("recipe error: {0}")]
    Recipe(String),

    #[error("schema error: {0}")]
    Schema(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("execution error: {0}")]
    Execution(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type SfResult<T> = Result<T, SfError>;

// Convenience conversions at boundaries
impl From<std::io::Error> for SfError {
    fn from(e: std::io::Error) -> Self {
        SfError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for SfError {
    fn from(e: serde_json::Error) -> Self {
        SfError::Internal(format!("JSON error: {}", e))
    }
}

impl From<polars::error::PolarsError> for SfError {
    fn from(e: polars::error::PolarsError) -> Self {
        SfError::Internal(format!("Polars error: {}", e))
    }
}

impl From<toml::de::Error> for SfError {
    fn from(e: toml::de::Error) -> Self {
        SfError::Recipe(format!("TOML parse error: {}", e))
    }
}
