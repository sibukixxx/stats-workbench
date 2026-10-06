use polars::prelude::*;
use sf_core::{SfError, SfResult};
use sqlx::{Column as SqlxColumn, Row};

/// Convert SQLite rows to a Polars DataFrame.
pub fn rows_to_dataframe(rows: &[sqlx::sqlite::SqliteRow]) -> SfResult<DataFrame> {
    if rows.is_empty() {
        return Err(SfError::Source("No rows to convert".into()));
    }

    let columns = rows[0].columns();
    let mut series_vec: Vec<polars::prelude::Column> = Vec::new();

    for col_info in columns {
        let col_name = col_info.name();
        let type_name = col_info.type_info().to_string();

        match type_name.to_uppercase().as_str() {
            "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" => {
                let values: Vec<Option<f64>> = rows
                    .iter()
                    .map(|row| row.try_get::<f64, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
            "INTEGER" | "INT" | "BIGINT" | "SMALLINT" => {
                let values: Vec<Option<i64>> = rows
                    .iter()
                    .map(|row| row.try_get::<i64, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
            _ => {
                let values: Vec<Option<String>> = rows
                    .iter()
                    .map(|row| row.try_get::<String, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
        }
    }

    DataFrame::new(series_vec).map_err(|e| SfError::Internal(format!("DataFrame build: {}", e)))
}

/// Convert PostgreSQL rows to a Polars DataFrame.
pub fn pg_rows_to_dataframe(rows: &[sqlx::postgres::PgRow]) -> SfResult<DataFrame> {
    if rows.is_empty() {
        return Err(SfError::Source("No rows to convert".into()));
    }

    let columns = rows[0].columns();
    let mut series_vec: Vec<polars::prelude::Column> = Vec::new();

    for col_info in columns {
        let col_name = col_info.name();
        let type_name = col_info.type_info().to_string();

        match type_name.to_uppercase().as_str() {
            "FLOAT4" | "FLOAT8" | "NUMERIC" | "REAL" | "DOUBLE PRECISION" => {
                let values: Vec<Option<f64>> = rows
                    .iter()
                    .map(|row| row.try_get::<f64, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
            "INT2" | "INT4" | "INT8" | "SERIAL" | "BIGSERIAL" => {
                let values: Vec<Option<i64>> = rows
                    .iter()
                    .map(|row| row.try_get::<i64, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
            "BOOL" | "BOOLEAN" => {
                let values: Vec<Option<bool>> = rows
                    .iter()
                    .map(|row| row.try_get::<bool, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
            _ => {
                let values: Vec<Option<String>> = rows
                    .iter()
                    .map(|row| row.try_get::<String, _>(col_name).ok())
                    .collect();
                series_vec.push(polars::prelude::Column::new(col_name.into(), &values));
            }
        }
    }

    DataFrame::new(series_vec).map_err(|e| SfError::Internal(format!("DataFrame build: {}", e)))
}
