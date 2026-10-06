use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Compute summary statistics for given columns.
///
/// Returns a StatsArtifact with kind=Summary and values containing
/// a "columns" array with per-column stats.
pub fn compute_summary(df: &DataFrame, cols: &[String]) -> SfResult<StatsArtifact> {
    let target_cols: Vec<String> = if cols.is_empty() {
        df.get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        cols.to_vec()
    };

    let mut col_results = Vec::new();

    for col_name in &target_cols {
        let col = df
            .column(col_name)
            .map_err(|_| SfError::Schema(format!("Column not found: '{}'", col_name)))?;

        let is_numeric = matches!(
            col.dtype(),
            DataType::Float32
                | DataType::Float64
                | DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
        );

        let count = col.len() as u64;
        let null_count = col.null_count() as u64;

        if is_numeric {
            let series = col.as_materialized_series();
            let casted = series.cast(&DataType::Float64)?;

            let mean = casted.mean();
            let std = casted.std(1);
            let min = casted.min::<f64>().ok().flatten();
            let max = casted.max::<f64>().ok().flatten();

            let sorted = casted.sort(Default::default())?;
            let q25 = quantile_at(&sorted, 0.25);
            let q50 = quantile_at(&sorted, 0.50);
            let q75 = quantile_at(&sorted, 0.75);

            col_results.push(serde_json::json!({
                "name": col_name,
                "count": count,
                "null_count": null_count,
                "mean": mean,
                "std": std,
                "min": min,
                "max": max,
                "q25": q25,
                "q50": q50,
                "q75": q75,
            }));
        } else {
            col_results.push(serde_json::json!({
                "name": col_name,
                "count": count,
                "null_count": null_count,
                "mean": null,
                "std": null,
                "min": null,
                "max": null,
                "q25": null,
                "q50": null,
                "q75": null,
            }));
        }
    }

    let mut values = BTreeMap::new();
    values.insert("columns".to_string(), serde_json::to_value(col_results)?);

    Ok(StatsArtifact {
        kind: StatsKind::Summary,
        values,
    })
}

fn quantile_at(sorted: &Series, q: f64) -> Option<f64> {
    let non_null = sorted.drop_nulls();
    let len = non_null.len();
    if len == 0 {
        return None;
    }
    let idx = ((len - 1) as f64 * q).round() as usize;
    non_null.get(idx).ok().and_then(|v| match v {
        AnyValue::Float64(f) => Some(f),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_basic() {
        let df = df! {
            "a" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0],
            "b" => &[10.0_f64, 20.0, 30.0, 40.0, 50.0],
        }
        .unwrap();

        let result = compute_summary(&df, &["a".into(), "b".into()]).unwrap();
        assert!(matches!(result.kind, StatsKind::Summary));

        let columns = result.values.get("columns").unwrap().as_array().unwrap();
        assert_eq!(columns.len(), 2);

        let col_a = &columns[0];
        assert_eq!(col_a["name"], "a");
        assert_eq!(col_a["count"], 5);
        assert!((col_a["mean"].as_f64().unwrap() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_summary_all_cols() {
        let df = df! {
            "x" => &[1.0_f64, 2.0],
        }
        .unwrap();

        let result = compute_summary(&df, &[]).unwrap();
        let columns = result.values.get("columns").unwrap().as_array().unwrap();
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0]["name"], "x");
    }

    #[test]
    fn test_summary_missing_col() {
        let df = df! { "a" => &[1.0_f64] }.unwrap();
        let result = compute_summary(&df, &["missing".into()]);
        assert!(result.is_err());
    }
}
