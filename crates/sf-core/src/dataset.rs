use polars::prelude::{DataFrame, IntoLazy, LazyFrame};

/// Two-mode dataset reference.
///
/// LazyFrame for deferred evaluation (parquet scan, query pushdown).
/// Materialized when a step needs random access or repeated iteration.
#[derive(Clone)]
pub enum DatasetRef {
    Lazy(LazyFrame),
    Materialized(DataFrame),
}

impl std::fmt::Debug for DatasetRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatasetRef::Lazy(_) => write!(f, "DatasetRef::Lazy(...)"),
            DatasetRef::Materialized(df) => {
                write!(
                    f,
                    "DatasetRef::Materialized({} x {})",
                    df.height(),
                    df.width()
                )
            }
        }
    }
}

impl DatasetRef {
    /// Get a LazyFrame. If materialized, convert back to lazy.
    pub fn lazy(&self) -> LazyFrame {
        match self {
            DatasetRef::Lazy(lf) => lf.clone(),
            DatasetRef::Materialized(df) => df.clone().lazy(),
        }
    }

    /// Collect into a DataFrame. If already materialized, return clone.
    pub fn collect(&self) -> Result<DataFrame, polars::error::PolarsError> {
        match self {
            DatasetRef::Lazy(lf) => lf.clone().collect(),
            DatasetRef::Materialized(df) => Ok(df.clone()),
        }
    }
}
