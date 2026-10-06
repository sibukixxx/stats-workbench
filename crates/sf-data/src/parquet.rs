use polars::prelude::*;
use sf_core::{SfError, SfResult};
use std::path::Path;

/// Read a Parquet file into a Polars LazyFrame.
pub fn read_parquet(path: &str) -> SfResult<LazyFrame> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(SfError::Source(format!(
            "Parquet file not found: {}",
            path.display()
        )));
    }

    let args = ScanArgsParquet::default();
    let lf = LazyFrame::scan_parquet(path, args)
        .map_err(|e| SfError::Source(format!("Failed to scan parquet: {}", e)))?;
    tracing::info!("Loaded parquet: {}", path.display());
    Ok(lf)
}
