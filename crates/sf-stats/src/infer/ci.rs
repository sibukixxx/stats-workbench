//! Confidence interval computations for population parameters.

use polars::prelude::*;
use sf_core::{SfError, SfResult, StatsArtifact, StatsKind};
use std::collections::BTreeMap;

use super::distributions::{chi2_quantile, normal_quantile, t_quantile};

/// Compute a confidence interval for the population mean.
///
/// Uses the t-distribution when population variance is unknown (which is always
/// the case in practice).
pub fn compute_mean_ci(
    df: &DataFrame,
    col: &str,
    confidence: f64,
) -> SfResult<StatsArtifact> {
    let series = df
        .column(col)
        .map_err(|e| SfError::Execution(format!("Column '{}' not found: {}", col, e)))?
        .as_materialized_series();

    let values: Vec<f64> = series
        .f64()
        .map_err(|e| SfError::Execution(format!("Column '{}' is not numeric: {}", col, e)))?
        .into_no_null_iter()
        .collect();

    let n = values.len();
    if n < 2 {
        return Err(SfError::Validation(
            "Mean CI requires at least 2 observations".into(),
        ));
    }

    let n_f = n as f64;
    let mean = values.iter().sum::<f64>() / n_f;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let std_dev = variance.sqrt();
    let se = std_dev / n_f.sqrt();
    let df_val = n_f - 1.0;

    let alpha = 1.0 - confidence;
    let t_crit = t_quantile(1.0 - alpha / 2.0, df_val);
    let ci_lower = mean - t_crit * se;
    let ci_upper = mean + t_crit * se;
    let margin_of_error = t_crit * se;

    let mut values_map = BTreeMap::new();
    values_map.insert("estimate".into(), serde_json::json!("mean"));
    values_map.insert("column".into(), serde_json::json!(col));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("mean".into(), serde_json::json!(mean));
    values_map.insert("std_dev".into(), serde_json::json!(std_dev));
    values_map.insert("se".into(), serde_json::json!(se));
    values_map.insert("df".into(), serde_json::json!(df_val));
    values_map.insert("confidence_level".into(), serde_json::json!(confidence));
    values_map.insert("ci_lower".into(), serde_json::json!(ci_lower));
    values_map.insert("ci_upper".into(), serde_json::json!(ci_upper));
    values_map.insert("margin_of_error".into(), serde_json::json!(margin_of_error));
    values_map.insert("method".into(), serde_json::json!("t_distribution"));

    Ok(StatsArtifact {
        kind: StatsKind::ConfidenceInterval,
        values: values_map,
    })
}

/// Compute a confidence interval for the population proportion.
///
/// Uses the normal approximation (Wald interval). Requires np >= 5 and n(1-p) >= 5.
pub fn compute_proportion_ci(
    n: u64,
    successes: u64,
    confidence: f64,
) -> SfResult<StatsArtifact> {
    if n == 0 {
        return Err(SfError::Validation("Sample size must be > 0".into()));
    }
    if successes > n {
        return Err(SfError::Validation(
            "Number of successes cannot exceed sample size".into(),
        ));
    }

    let n_f = n as f64;
    let p_hat = successes as f64 / n_f;
    let se = (p_hat * (1.0 - p_hat) / n_f).sqrt();

    let alpha = 1.0 - confidence;
    let z_crit = normal_quantile(1.0 - alpha / 2.0);
    let ci_lower = (p_hat - z_crit * se).max(0.0);
    let ci_upper = (p_hat + z_crit * se).min(1.0);
    let margin_of_error = z_crit * se;

    let mut values_map = BTreeMap::new();
    values_map.insert("estimate".into(), serde_json::json!("proportion"));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("successes".into(), serde_json::json!(successes));
    values_map.insert("p_hat".into(), serde_json::json!(p_hat));
    values_map.insert("se".into(), serde_json::json!(se));
    values_map.insert("confidence_level".into(), serde_json::json!(confidence));
    values_map.insert("ci_lower".into(), serde_json::json!(ci_lower));
    values_map.insert("ci_upper".into(), serde_json::json!(ci_upper));
    values_map.insert("margin_of_error".into(), serde_json::json!(margin_of_error));
    values_map.insert("method".into(), serde_json::json!("wald_normal_approximation"));

    Ok(StatsArtifact {
        kind: StatsKind::ConfidenceInterval,
        values: values_map,
    })
}

/// Compute a confidence interval for the population variance.
///
/// Uses the chi-squared distribution.
pub fn compute_variance_ci(
    df: &DataFrame,
    col: &str,
    confidence: f64,
) -> SfResult<StatsArtifact> {
    let series = df
        .column(col)
        .map_err(|e| SfError::Execution(format!("Column '{}' not found: {}", col, e)))?
        .as_materialized_series();

    let values: Vec<f64> = series
        .f64()
        .map_err(|e| SfError::Execution(format!("Column '{}' is not numeric: {}", col, e)))?
        .into_no_null_iter()
        .collect();

    let n = values.len();
    if n < 2 {
        return Err(SfError::Validation(
            "Variance CI requires at least 2 observations".into(),
        ));
    }

    let n_f = n as f64;
    let mean = values.iter().sum::<f64>() / n_f;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let df_val = n_f - 1.0;

    let alpha = 1.0 - confidence;
    let chi2_lower = chi2_quantile(alpha / 2.0, df_val);
    let chi2_upper = chi2_quantile(1.0 - alpha / 2.0, df_val);

    // CI for variance: ((n-1)*s^2 / chi2_upper, (n-1)*s^2 / chi2_lower)
    let ci_lower = df_val * variance / chi2_upper;
    let ci_upper = df_val * variance / chi2_lower;

    let mut values_map = BTreeMap::new();
    values_map.insert("estimate".into(), serde_json::json!("variance"));
    values_map.insert("column".into(), serde_json::json!(col));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("variance".into(), serde_json::json!(variance));
    values_map.insert("std_dev".into(), serde_json::json!(variance.sqrt()));
    values_map.insert("df".into(), serde_json::json!(df_val));
    values_map.insert("confidence_level".into(), serde_json::json!(confidence));
    values_map.insert("ci_lower".into(), serde_json::json!(ci_lower));
    values_map.insert("ci_upper".into(), serde_json::json!(ci_upper));
    values_map.insert("method".into(), serde_json::json!("chi_squared_distribution"));

    Ok(StatsArtifact {
        kind: StatsKind::ConfidenceInterval,
        values: values_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_df() -> DataFrame {
        DataFrame::new(vec![Column::new(
            "value".into(),
            &[10.0, 12.0, 14.0, 11.0, 13.0, 15.0, 9.0, 11.0, 12.0, 14.0],
        )])
        .unwrap()
    }

    // ── Mean CI ──

    #[test]
    fn mean_ci_returns_correct_structure() {
        let df = sample_df();
        let result = compute_mean_ci(&df, "value", 0.95).unwrap();
        assert!(matches!(result.kind, StatsKind::ConfidenceInterval));
        assert_eq!(result.values["estimate"].as_str(), Some("mean"));
        assert!(result.values.contains_key("ci_lower"));
        assert!(result.values.contains_key("ci_upper"));
        assert!(result.values.contains_key("margin_of_error"));
    }

    #[test]
    fn mean_ci_contains_sample_mean() {
        let df = sample_df();
        let result = compute_mean_ci(&df, "value", 0.95).unwrap();
        let mean = result.values["mean"].as_f64().unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(ci_lower < mean && mean < ci_upper);
    }

    #[test]
    fn mean_ci_wider_at_higher_confidence() {
        let df = sample_df();
        let result_95 = compute_mean_ci(&df, "value", 0.95).unwrap();
        let result_99 = compute_mean_ci(&df, "value", 0.99).unwrap();

        let width_95 = result_95.values["ci_upper"].as_f64().unwrap()
            - result_95.values["ci_lower"].as_f64().unwrap();
        let width_99 = result_99.values["ci_upper"].as_f64().unwrap()
            - result_99.values["ci_lower"].as_f64().unwrap();

        assert!(width_99 > width_95, "99% CI should be wider than 95% CI");
    }

    #[test]
    fn mean_ci_rejects_insufficient_data() {
        let df = DataFrame::new(vec![Column::new("value".into(), &[10.0])]).unwrap();
        let result = compute_mean_ci(&df, "value", 0.95);
        assert!(result.is_err());
    }

    // ── Proportion CI ──

    #[test]
    fn proportion_ci_returns_correct_structure() {
        let result = compute_proportion_ci(100, 60, 0.95).unwrap();
        assert!(matches!(result.kind, StatsKind::ConfidenceInterval));
        assert_eq!(result.values["estimate"].as_str(), Some("proportion"));
        let p_hat = result.values["p_hat"].as_f64().unwrap();
        assert!((p_hat - 0.6).abs() < 1e-10);
    }

    #[test]
    fn proportion_ci_bounds_between_zero_and_one() {
        let result = compute_proportion_ci(100, 60, 0.95).unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!((0.0..=1.0).contains(&ci_lower));
        assert!((0.0..=1.0).contains(&ci_upper));
        assert!(ci_lower < ci_upper);
    }

    #[test]
    fn proportion_ci_contains_p_hat() {
        let result = compute_proportion_ci(100, 60, 0.95).unwrap();
        let p_hat = result.values["p_hat"].as_f64().unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(ci_lower < p_hat && p_hat < ci_upper);
    }

    #[test]
    fn proportion_ci_rejects_successes_exceeding_n() {
        let result = compute_proportion_ci(100, 101, 0.95);
        assert!(result.is_err());
    }

    // ── Variance CI ──

    #[test]
    fn variance_ci_returns_correct_structure() {
        let df = sample_df();
        let result = compute_variance_ci(&df, "value", 0.95).unwrap();
        assert!(matches!(result.kind, StatsKind::ConfidenceInterval));
        assert_eq!(result.values["estimate"].as_str(), Some("variance"));
        assert!(result.values.contains_key("variance"));
        assert!(result.values.contains_key("ci_lower"));
        assert!(result.values.contains_key("ci_upper"));
    }

    #[test]
    fn variance_ci_contains_sample_variance() {
        let df = sample_df();
        let result = compute_variance_ci(&df, "value", 0.95).unwrap();
        let variance = result.values["variance"].as_f64().unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(
            ci_lower < variance && variance < ci_upper,
            "CI [{}, {}] should contain variance {}",
            ci_lower,
            ci_upper,
            variance
        );
    }

    #[test]
    fn variance_ci_positive_bounds() {
        let df = sample_df();
        let result = compute_variance_ci(&df, "value", 0.95).unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(ci_lower > 0.0);
        assert!(ci_upper > ci_lower);
    }
}
