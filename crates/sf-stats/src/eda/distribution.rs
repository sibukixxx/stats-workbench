use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Compute histogram bin aggregation for a single column.
pub fn compute_distribution(df: &DataFrame, col: &str, bins: u32) -> SfResult<StatsArtifact> {
    if bins == 0 {
        return Err(SfError::Validation("bins must be > 0".into()));
    }

    let column = df
        .column(col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", col)))?;
    let series = column.as_materialized_series();
    let casted = series.cast(&DataType::Float64)?;
    let values: Vec<f64> = casted.f64()?.into_no_null_iter().collect();

    if values.is_empty() {
        return Err(SfError::Execution(format!(
            "Column '{}' has no non-null values",
            col
        )));
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = (max - min) / bins as f64;

    let mut bin_results = Vec::new();
    for i in 0..bins {
        let lower = min + bin_width * i as f64;
        let upper = if i == bins - 1 {
            max
        } else {
            min + bin_width * (i + 1) as f64
        };
        let count = values
            .iter()
            .filter(|&&v| {
                if i == bins - 1 {
                    v >= lower && v <= upper
                } else {
                    v >= lower && v < upper
                }
            })
            .count() as u64;
        bin_results.push(serde_json::json!({
            "lower": lower,
            "upper": upper,
            "count": count,
        }));
    }

    let mut bmap = BTreeMap::new();
    bmap.insert(
        "column".to_string(),
        serde_json::Value::String(col.to_string()),
    );
    bmap.insert("bins".to_string(), serde_json::to_value(bin_results)?);

    Ok(StatsArtifact {
        kind: StatsKind::Distribution,
        values: bmap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribution() {
        let df = polars::prelude::df! {
            "val" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        }
        .unwrap();

        let result = compute_distribution(&df, "val", 5).unwrap();
        let bins = result.values.get("bins").unwrap().as_array().unwrap();
        assert_eq!(bins.len(), 5);
        let total: u64 = bins.iter().map(|b| b["count"].as_u64().unwrap()).sum();
        assert_eq!(total, 10);
    }

    #[test]
    fn test_distribution_zero_bins() {
        let df = polars::prelude::df! { "v" => &[1.0_f64] }.unwrap();
        assert!(compute_distribution(&df, "v", 0).is_err());
    }
}
