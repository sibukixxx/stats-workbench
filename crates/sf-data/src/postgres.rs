use polars::prelude::*;
use sf_core::{SfError, SfResult};
use sqlx::postgres::PgPool;

use crate::query::build_select_query;

/// Read from PostgreSQL into a Polars LazyFrame.
///
/// The query runs inside a `READ ONLY` transaction that is always rolled
/// back, so a recipe-supplied `query` cannot modify data even when the
/// connection role has write privileges.
pub async fn read_postgres(
    uri: &str,
    table: Option<&str>,
    query: Option<&str>,
    filter: Option<&str>,
) -> SfResult<LazyFrame> {
    let pool = PgPool::connect(uri)
        .await
        .map_err(|e| SfError::Database(format!("PostgreSQL connection failed: {}", e)))?;

    let sql = build_select_query(table, query, filter)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| SfError::Database(format!("PostgreSQL transaction failed: {}", e)))?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(|e| SfError::Database(format!("PostgreSQL read-only setup failed: {}", e)))?;

    let rows = sqlx::query(&sql)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| SfError::Database(format!("PostgreSQL query failed: {}", e)))?;

    // Nothing to commit; rolling back also releases the connection promptly.
    let _ = tx.rollback().await;

    if rows.is_empty() {
        return Err(SfError::Source(
            "No rows returned from PostgreSQL query".into(),
        ));
    }

    let df = crate::normalize::pg_rows_to_dataframe(&rows)?;
    tracing::info!(
        "Loaded PostgreSQL: {} rows, {} cols",
        df.height(),
        df.width()
    );
    Ok(df.lazy())
}
