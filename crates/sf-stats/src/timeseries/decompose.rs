use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Classical additive decomposition: Y = Trend + Seasonal + Residual.
///
/// Uses centered moving average for trend extraction.
pub fn compute_decompose(
    df: &DataFrame,
    time_col: &str,
    value_col: &str,
    period: usize,
) -> SfResult<StatsArtifact> {
    let _ = df
        .column(time_col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", time_col)))?;
    let value_series = df
        .column(value_col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", value_col)))?;

    let values = series_to_f64_vec(value_series.as_materialized_series())?;
    let n = values.len();

    if n < period * 2 {
        return Err(SfError::Validation(format!(
            "Decomposition requires at least {} observations (2 * period={})",
            period * 2,
            period
        )));
    }

    // Step 1: Compute trend using centered moving average
    let trend = centered_moving_average(&values, period);

    // Step 2: Detrend
    let detrended: Vec<Option<f64>> = values
        .iter()
        .zip(&trend)
        .map(|(y, t)| t.map(|tv| y - tv))
        .collect();

    // Step 3: Compute seasonal component (average of detrended values at each period position)
    let mut seasonal_means = vec![0.0; period];
    let mut seasonal_counts = vec![0usize; period];
    for (i, dt) in detrended.iter().enumerate() {
        if let Some(v) = dt {
            let pos = i % period;
            seasonal_means[pos] += v;
            seasonal_counts[pos] += 1;
        }
    }
    for (mean, count) in seasonal_means.iter_mut().zip(&seasonal_counts) {
        if *count > 0 {
            *mean /= *count as f64;
        }
    }

    // Center seasonal component (subtract mean)
    let seasonal_avg: f64 = seasonal_means.iter().sum::<f64>() / period as f64;
    for s in &mut seasonal_means {
        *s -= seasonal_avg;
    }

    // Expand seasonal to full length
    let seasonal: Vec<f64> = (0..n).map(|i| seasonal_means[i % period]).collect();

    // Step 4: Compute residual
    let residual: Vec<Option<f64>> = values
        .iter()
        .zip(&trend)
        .zip(&seasonal)
        .map(|((y, t), s)| t.map(|tv| y - tv - s))
        .collect();

    // Build output
    let trend_json: Vec<serde_json::Value> = trend
        .iter()
        .map(|v| match v {
            Some(f) => serde_json::json!(f),
            None => serde_json::Value::Null,
        })
        .collect();

    let residual_json: Vec<serde_json::Value> = residual
        .iter()
        .map(|v| match v {
            Some(f) => serde_json::json!(f),
            None => serde_json::Value::Null,
        })
        .collect();

    let mut values_map = BTreeMap::new();
    values_map.insert("period".to_string(), serde_json::json!(period));
    values_map.insert("method".to_string(), serde_json::json!("additive"));
    values_map.insert("n_obs".to_string(), serde_json::json!(n));
    values_map.insert("trend".to_string(), serde_json::json!(trend_json));
    values_map.insert("seasonal".to_string(), serde_json::json!(seasonal));
    values_map.insert("residual".to_string(), serde_json::json!(residual_json));

    // Summary stats for each component
    let trend_vals: Vec<f64> = trend.iter().filter_map(|v| *v).collect();
    let resid_vals: Vec<f64> = residual.iter().filter_map(|v| *v).collect();

    values_map.insert(
        "summary".to_string(),
        serde_json::json!({
            "trend_mean": mean_of(&trend_vals),
            "trend_std": std_of(&trend_vals),
            "seasonal_range": seasonal_range(&seasonal_means),
            "residual_mean": mean_of(&resid_vals),
            "residual_std": std_of(&resid_vals),
        }),
    );

    Ok(StatsArtifact {
        kind: StatsKind::Decompose,
        values: values_map,
    })
}

/// Centered moving average of window size `period`.
fn centered_moving_average(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut result = vec![None; n];

    if period == 0 || n < period {
        return result;
    }

    if period % 2 == 1 {
        // Odd period: simple centered MA
        let half = period / 2;
        for i in half..(n - half) {
            let sum: f64 = values[i - half..=i + half].iter().sum();
            result[i] = Some(sum / period as f64);
        }
    } else {
        // Even period: 2x moving average (centered)
        // First pass: period-MA
        let mut ma1 = vec![None; n];
        let half = period / 2;
        for i in 0..=(n - period) {
            let sum: f64 = values[i..i + period].iter().sum();
            ma1[i + half - 1] = Some(sum / period as f64);
        }
        // Second pass: 2-MA of the first pass to center
        for i in 0..(n - 1) {
            if let (Some(a), Some(b)) = (ma1[i], ma1[i + 1]) {
                result[i + 1] = Some((a + b) / 2.0);
            }
        }
    }

    result
}

fn mean_of(vals: &[f64]) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.iter().sum::<f64>() / vals.len() as f64
}

fn std_of(vals: &[f64]) -> f64 {
    if vals.len() < 2 {
        return 0.0;
    }
    let m = mean_of(vals);
    let var = vals.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (vals.len() - 1) as f64;
    var.sqrt()
}

fn seasonal_range(seasonal: &[f64]) -> f64 {
    if seasonal.is_empty() {
        return 0.0;
    }
    let min = seasonal.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = seasonal.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    max - min
}

fn series_to_f64_vec(s: &Series) -> SfResult<Vec<f64>> {
    let casted = s
        .cast(&DataType::Float64)
        .map_err(|e| SfError::Execution(format!("Cannot cast '{}' to f64: {}", s.name(), e)))?;
    let ca = casted.f64().map_err(|e| {
        SfError::Execution(format!("Cannot extract f64 from '{}': {}", s.name(), e))
    })?;
    Ok(ca.into_no_null_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_seasonal_df(n: usize, period: usize) -> DataFrame {
        let time: Vec<i64> = (0..n as i64).collect();
        let values: Vec<f64> = (0..n)
            .map(|i| {
                let trend = 10.0 + 0.5 * i as f64;
                let seasonal = 3.0 * ((2.0 * std::f64::consts::PI * i as f64) / period as f64).sin();
                trend + seasonal
            })
            .collect();
        df! {
            "time" => &time,
            "value" => &values,
        }
        .unwrap()
    }

    #[test]
    fn decompose_returns_correct_structure() {
        let df = make_seasonal_df(48, 12);
        let result = compute_decompose(&df, "time", "value", 12).unwrap();

        assert!(matches!(result.kind, StatsKind::Decompose));
        assert!(result.values.contains_key("trend"));
        assert!(result.values.contains_key("seasonal"));
        assert!(result.values.contains_key("residual"));
        assert!(result.values.contains_key("period"));
        assert!(result.values.contains_key("summary"));

        assert_eq!(result.values["period"].as_u64().unwrap(), 12);
    }

    #[test]
    fn decompose_trend_is_correct_length() {
        let df = make_seasonal_df(48, 12);
        let result = compute_decompose(&df, "time", "value", 12).unwrap();

        let trend = result.values["trend"].as_array().unwrap();
        assert_eq!(trend.len(), 48);
    }

    #[test]
    fn decompose_seasonal_repeats_with_period() {
        let df = make_seasonal_df(48, 12);
        let result = compute_decompose(&df, "time", "value", 12).unwrap();

        let seasonal = result.values["seasonal"].as_array().unwrap();
        // Seasonal should repeat with period
        for i in 0..12 {
            let s0 = seasonal[i].as_f64().unwrap();
            let s1 = seasonal[i + 12].as_f64().unwrap();
            assert!(
                (s0 - s1).abs() < 1e-10,
                "Seasonal should repeat: {} vs {}",
                s0,
                s1
            );
        }
    }

    #[test]
    fn decompose_rejects_too_few_observations() {
        let df = make_seasonal_df(10, 12);
        let result = compute_decompose(&df, "time", "value", 12);
        assert!(result.is_err());
    }

    #[test]
    fn decompose_rejects_missing_column() {
        let df = make_seasonal_df(48, 12);
        let result = compute_decompose(&df, "time", "missing", 12);
        assert!(result.is_err());
    }

    #[test]
    fn decompose_residual_is_small_for_clean_data() {
        let df = make_seasonal_df(48, 12);
        let result = compute_decompose(&df, "time", "value", 12).unwrap();

        let summary = result.values.get("summary").unwrap();
        let resid_std = summary["residual_std"].as_f64().unwrap();
        // For clean trend+seasonal data, residual should be small
        assert!(
            resid_std < 1.0,
            "Residual std should be small for clean data, got {}",
            resid_std
        );
    }
}
