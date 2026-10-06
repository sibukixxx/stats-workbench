//! Variance hypothesis tests (chi-squared test for single population variance).

use polars::prelude::*;
use sf_core::{SfError, SfResult, StatsArtifact, StatsKind};
use std::collections::BTreeMap;

use super::distributions::{chi2_quantile, chi2_two_tailed_p};
use super::ttest::Alternative;

/// Chi-squared test for population variance.
///
/// Tests whether the population variance equals `sigma2_0`.
pub fn compute_variance_test(
    df: &DataFrame,
    col: &str,
    sigma2_0: f64,
    confidence: f64,
    alternative: Alternative,
) -> SfResult<StatsArtifact> {
    if sigma2_0 <= 0.0 {
        return Err(SfError::Validation(
            "Null hypothesis variance sigma2_0 must be > 0".into(),
        ));
    }

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
            "Variance test requires at least 2 observations".into(),
        ));
    }

    let n_f = n as f64;
    let mean = values.iter().sum::<f64>() / n_f;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let df_val = n_f - 1.0;

    let chi2_stat = df_val * variance / sigma2_0;

    let p_value = match alternative {
        Alternative::TwoSided => chi2_two_tailed_p(chi2_stat, df_val),
        Alternative::Less => super::distributions::chi2_cdf(chi2_stat, df_val),
        Alternative::Greater => 1.0 - super::distributions::chi2_cdf(chi2_stat, df_val),
    };

    // CI for variance
    let alpha = 1.0 - confidence;
    let chi2_lower = chi2_quantile(alpha / 2.0, df_val);
    let chi2_upper = chi2_quantile(1.0 - alpha / 2.0, df_val);
    let ci_lower = df_val * variance / chi2_upper;
    let ci_upper = df_val * variance / chi2_lower;

    let mut values_map = BTreeMap::new();
    values_map.insert("test".into(), serde_json::json!("variance_chi2_test"));
    values_map.insert("column".into(), serde_json::json!(col));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("variance".into(), serde_json::json!(variance));
    values_map.insert("std_dev".into(), serde_json::json!(variance.sqrt()));
    values_map.insert("sigma2_0".into(), serde_json::json!(sigma2_0));
    values_map.insert("chi2_statistic".into(), serde_json::json!(chi2_stat));
    values_map.insert("df".into(), serde_json::json!(df_val));
    values_map.insert("p_value".into(), serde_json::json!(p_value));
    values_map.insert(
        "alternative".into(),
        serde_json::json!(match alternative {
            Alternative::TwoSided => "two_sided",
            Alternative::Less => "less",
            Alternative::Greater => "greater",
        }),
    );
    values_map.insert("confidence_level".into(), serde_json::json!(confidence));
    values_map.insert("ci_lower".into(), serde_json::json!(ci_lower));
    values_map.insert("ci_upper".into(), serde_json::json!(ci_upper));

    Ok(StatsArtifact {
        kind: StatsKind::VarianceTest,
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

    #[test]
    fn variance_test_returns_correct_structure() {
        let df = sample_df();
        let result = compute_variance_test(&df, "value", 4.0, 0.95, Alternative::TwoSided).unwrap();
        assert!(matches!(result.kind, StatsKind::VarianceTest));
        assert_eq!(
            result.values["test"].as_str(),
            Some("variance_chi2_test")
        );
        assert!(result.values.contains_key("chi2_statistic"));
        assert!(result.values.contains_key("p_value"));
        assert!(result.values.contains_key("df"));
        assert!(result.values.contains_key("ci_lower"));
        assert!(result.values.contains_key("ci_upper"));
    }

    #[test]
    fn variance_test_high_p_when_sigma2_matches() {
        let df = sample_df();
        // Calculate sample variance first: it's about 3.66
        let result = compute_variance_test(&df, "value", 3.66, 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p > 0.5, "p-value should be high when sigma2_0 is near sample variance, got {}", p);
    }

    #[test]
    fn variance_test_low_p_when_sigma2_far() {
        let df = sample_df();
        let result =
            compute_variance_test(&df, "value", 100.0, 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p < 0.05, "p-value should be low, got {}", p);
    }

    #[test]
    fn variance_test_rejects_non_positive_sigma2() {
        let df = sample_df();
        let result = compute_variance_test(&df, "value", -1.0, 0.95, Alternative::TwoSided);
        assert!(result.is_err());
    }

    #[test]
    fn variance_test_ci_contains_sample_variance() {
        let df = sample_df();
        let result = compute_variance_test(&df, "value", 4.0, 0.95, Alternative::TwoSided).unwrap();
        let variance = result.values["variance"].as_f64().unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(ci_lower < variance && variance < ci_upper);
    }
}
