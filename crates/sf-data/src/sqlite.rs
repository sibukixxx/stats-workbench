use polars::prelude::*;
use sf_core::{SfError, SfResult};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use crate::query::build_select_query;

/// Read from SQLite into a Polars LazyFrame.
///
/// The database is opened read-only (and never created): Stats Workbench only
/// analyses data, and a recipe must not be able to write to or create files
/// through a `query`, a `?mode=rwc` URI suffix, or `ATTACH`.
pub async fn read_sqlite(
    path: &str,
    table: Option<&str>,
    query: Option<&str>,
) -> SfResult<LazyFrame> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .create_if_missing(false);
    let pool = SqlitePool::connect_with(options)
        .await
        .map_err(|e| SfError::Database(format!("SQLite connection failed: {}", e)))?;

    let sql = build_select_query(table, query, None)?;

    let rows = sqlx::query(&sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| SfError::Database(format!("SQLite query failed: {}", e)))?;

    if rows.is_empty() {
        return Err(SfError::Source("No rows returned from SQLite query".into()));
    }

    let df = crate::normalize::rows_to_dataframe(&rows)?;
    tracing::info!(
        "Loaded SQLite ({}): {} rows, {} cols",
        path,
        df.height(),
        df.width()
    );
    Ok(df.lazy())
}
