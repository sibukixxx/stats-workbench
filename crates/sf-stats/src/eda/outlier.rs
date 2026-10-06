use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Detect outliers using IQR method for one or more columns.
pub fn compute_outlier_detection(df: &DataFrame, cols: &[String]) -> SfResult<StatsArtifact> {
    let target_cols = if cols.is_empty() {
        return Err(SfError::Validation(
            "outlier_detect requires at least one column".into(),
        ));
    } else {
        cols.to_vec()
    };

    let mut values = BTreeMap::new();

    for col_name in &target_cols {
        let col = df
            .column(col_name)
            .map_err(|_| SfError::Schema(format!("Column not found: '{}'", col_name)))?;
        let series = col.as_materialized_series();
        let casted = series.cast(&DataType::Float64)?;
        let sorted = casted.sort(Default::default())?;
        let sorted_vals: Vec<f64> = sorted.f64()?.into_no_null_iter().collect();

        if sorted_vals.is_empty() {
            return Err(SfError::Execution(format!(
                "Column '{}' has no non-null values",
                col_name
            )));
        }

        let n = sorted_vals.len();
        let q25 = sorted_vals[((n - 1) as f64 * 0.25).round() as usize];
        let q75 = sorted_vals[((n - 1) as f64 * 0.75).round() as usize];
        let iqr = q75 - q25;
        let lower_bound = q25 - 1.5 * iqr;
        let upper_bound = q75 + 1.5 * iqr;

        let all_values: Vec<f64> = casted.f64()?.into_no_null_iter().collect();
        let outlier_count = all_values
            .iter()
            .filter(|&&v| v < lower_bound || v > upper_bound)
            .count() as u64;

        values.insert(
            col_name.clone(),
            serde_json::json!({
                "method": "iqr",
                "total_rows": all_values.len() as u64,
                "outlier_count": outlier_count,
                "lower_bound": lower_bound,
                "upper_bound": upper_bound,
                "q25": q25,
                "q75": q75,
                "iqr": iqr,
            }),
        );
    }

    Ok(StatsArtifact {
        kind: StatsKind::OutlierDetection,
        values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_detect() {
        let df = polars::prelude::df! {
            "val" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 100.0],
        }
        .unwrap();

        let result = compute_outlier_detection(&df, &["val".into()]).unwrap();
        let val_info = result.values.get("val").unwrap();
        assert_eq!(val_info["total_rows"], 6);
        assert!(val_info["outlier_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn test_outlier_no_cols() {
        let df = polars::prelude::df! { "v" => &[1.0_f64] }.unwrap();
        assert!(compute_outlier_detection(&df, &[]).is_err());
    }
}
