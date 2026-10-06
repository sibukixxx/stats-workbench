//! T-test implementations: 1-sample and 2-sample (independent).

use polars::prelude::*;
use sf_core::{SfError, SfResult, StatsArtifact, StatsKind};
use std::collections::BTreeMap;

use super::distributions::{t_quantile, t_two_tailed_p};

/// Alternative hypothesis for t-tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alternative {
    TwoSided,
    Less,
    Greater,
}

/// Compute a one-sample t-test.
///
/// Tests whether the population mean of `col` equals `mu0`.
/// Returns test statistic, p-value, df, CI, and effect size (Cohen's d).
pub fn compute_ttest_one_sample(
    df: &DataFrame,
    col: &str,
    mu0: f64,
    confidence: f64,
    alternative: Alternative,
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
            "One-sample t-test requires at least 2 observations".into(),
        ));
    }

    let n_f = n as f64;
    let mean = values.iter().sum::<f64>() / n_f;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let std_dev = variance.sqrt();
    let se = std_dev / n_f.sqrt();
    let df_val = n_f - 1.0;

    let t_stat = (mean - mu0) / se;

    let p_value = match alternative {
        Alternative::TwoSided => t_two_tailed_p(t_stat, df_val),
        Alternative::Less => super::distributions::t_cdf(t_stat, df_val),
        Alternative::Greater => 1.0 - super::distributions::t_cdf(t_stat, df_val),
    };

    // Confidence interval for the mean
    let alpha = 1.0 - confidence;
    let t_crit = t_quantile(1.0 - alpha / 2.0, df_val);
    let ci_lower = mean - t_crit * se;
    let ci_upper = mean + t_crit * se;

    // Cohen's d
    let cohens_d = (mean - mu0) / std_dev;

    let mut values_map = BTreeMap::new();
    values_map.insert("test".into(), serde_json::json!("one_sample_t_test"));
    values_map.insert("column".into(), serde_json::json!(col));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("mean".into(), serde_json::json!(mean));
    values_map.insert("std_dev".into(), serde_json::json!(std_dev));
    values_map.insert("mu0".into(), serde_json::json!(mu0));
    values_map.insert("t_statistic".into(), serde_json::json!(t_stat));
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
    values_map.insert("effect_size".into(), serde_json::json!(cohens_d));
    values_map.insert("effect_size_type".into(), serde_json::json!("cohens_d"));
    values_map.insert("se".into(), serde_json::json!(se));

    Ok(StatsArtifact {
        kind: StatsKind::TTest,
        values: values_map,
    })
}

/// Compute an independent two-sample t-test (Welch's t-test).
///
/// Tests whether the means of two groups differ.
/// Uses Welch's approximation for unequal variances.
pub fn compute_ttest_two_sample(
    df: &DataFrame,
    col: &str,
    group_col: &str,
    confidence: f64,
    alternative: Alternative,
) -> SfResult<StatsArtifact> {
    let value_series = df
        .column(col)
        .map_err(|e| SfError::Execution(format!("Column '{}' not found: {}", col, e)))?
        .as_materialized_series();
    let group_series = df
        .column(group_col)
        .map_err(|e| SfError::Execution(format!("Group column '{}' not found: {}", group_col, e)))?
        .as_materialized_series();

    // Get unique groups
    let groups = group_series.unique().map_err(|e| {
        SfError::Execution(format!("Failed to get unique groups: {}", e))
    })?;
    if groups.len() != 2 {
        return Err(SfError::Validation(format!(
            "Two-sample t-test requires exactly 2 groups, found {}",
            groups.len()
        )));
    }

    let group_names: Vec<String> = (0..2)
        .map(|i| {
            groups
                .get(i)
                .map(|v| format!("{}", v))
                .unwrap_or_else(|_| format!("group_{}", i))
        })
        .collect();

    // Split values by group using string comparison on formatted values
    let n = df.height();
    let mut values1: Vec<f64> = Vec::new();
    let mut values2: Vec<f64> = Vec::new();
    for i in 0..n {
        let group_val = format!(
            "{}",
            group_series
                .get(i)
                .map_err(|e| SfError::Execution(format!("Failed to get group value: {}", e)))?
        );
        let val = value_series
            .get(i)
            .map_err(|e| SfError::Execution(format!("Failed to get value: {}", e)))?;
        let val_f64 = match val {
            AnyValue::Float64(v) => v,
            AnyValue::Float32(v) => v as f64,
            AnyValue::Int32(v) => v as f64,
            AnyValue::Int64(v) => v as f64,
            _ => {
                return Err(SfError::Execution(format!(
                    "Column '{}' is not numeric",
                    col
                )));
            }
        };
        if group_val == group_names[0] {
            values1.push(val_f64);
        } else if group_val == group_names[1] {
            values2.push(val_f64);
        }
    }

    let n1 = values1.len();
    let n2 = values2.len();
    if n1 < 2 || n2 < 2 {
        return Err(SfError::Validation(
            "Each group in two-sample t-test requires at least 2 observations".into(),
        ));
    }

    let n1_f = n1 as f64;
    let n2_f = n2 as f64;
    let mean1 = values1.iter().sum::<f64>() / n1_f;
    let mean2 = values2.iter().sum::<f64>() / n2_f;
    let var1 = values1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1_f - 1.0);
    let var2 = values2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2_f - 1.0);

    // Welch's t-test
    let se = (var1 / n1_f + var2 / n2_f).sqrt();
    let t_stat = (mean1 - mean2) / se;

    // Welch-Satterthwaite degrees of freedom
    let num = (var1 / n1_f + var2 / n2_f).powi(2);
    let denom =
        (var1 / n1_f).powi(2) / (n1_f - 1.0) + (var2 / n2_f).powi(2) / (n2_f - 1.0);
    let df_val = num / denom;

    let p_value = match alternative {
        Alternative::TwoSided => t_two_tailed_p(t_stat, df_val),
        Alternative::Less => super::distributions::t_cdf(t_stat, df_val),
        Alternative::Greater => 1.0 - super::distributions::t_cdf(t_stat, df_val),
    };

    // CI for difference in means
    let alpha = 1.0 - confidence;
    let t_crit = t_quantile(1.0 - alpha / 2.0, df_val);
    let diff = mean1 - mean2;
    let ci_lower = diff - t_crit * se;
    let ci_upper = diff + t_crit * se;

    // Cohen's d (pooled std dev)
    let pooled_var = ((n1_f - 1.0) * var1 + (n2_f - 1.0) * var2) / (n1_f + n2_f - 2.0);
    let cohens_d = (mean1 - mean2) / pooled_var.sqrt();

    let mut values_map = BTreeMap::new();
    values_map.insert("test".into(), serde_json::json!("two_sample_t_test"));
    values_map.insert("column".into(), serde_json::json!(col));
    values_map.insert("group_column".into(), serde_json::json!(group_col));
    values_map.insert("groups".into(), serde_json::json!(group_names));
    values_map.insert("n1".into(), serde_json::json!(n1));
    values_map.insert("n2".into(), serde_json::json!(n2));
    values_map.insert("mean1".into(), serde_json::json!(mean1));
    values_map.insert("mean2".into(), serde_json::json!(mean2));
    values_map.insert("std_dev1".into(), serde_json::json!(var1.sqrt()));
    values_map.insert("std_dev2".into(), serde_json::json!(var2.sqrt()));
    values_map.insert("mean_diff".into(), serde_json::json!(diff));
    values_map.insert("t_statistic".into(), serde_json::json!(t_stat));
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
    values_map.insert("effect_size".into(), serde_json::json!(cohens_d));
    values_map.insert("effect_size_type".into(), serde_json::json!("cohens_d"));
    values_map.insert("se".into(), serde_json::json!(se));

    Ok(StatsArtifact {
        kind: StatsKind::TTest,
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

    fn two_sample_df() -> DataFrame {
        DataFrame::new(vec![
            Column::new(
                "score".into(),
                &[10.0, 12.0, 14.0, 11.0, 13.0, 20.0, 22.0, 18.0, 21.0, 19.0],
            ),
            Column::new(
                "group".into(),
                &["A", "A", "A", "A", "A", "B", "B", "B", "B", "B"],
            ),
        ])
        .unwrap()
    }

    // ── One-sample t-test ──

    #[test]
    fn one_sample_ttest_returns_correct_structure() {
        let df = sample_df();
        let result = compute_ttest_one_sample(&df, "value", 12.0, 0.95, Alternative::TwoSided).unwrap();
        assert!(matches!(result.kind, StatsKind::TTest));
        assert_eq!(
            result.values.get("test").and_then(|v| v.as_str()),
            Some("one_sample_t_test")
        );
        assert!(result.values.contains_key("t_statistic"));
        assert!(result.values.contains_key("p_value"));
        assert!(result.values.contains_key("df"));
        assert!(result.values.contains_key("ci_lower"));
        assert!(result.values.contains_key("ci_upper"));
        assert!(result.values.contains_key("effect_size"));
    }

    #[test]
    fn one_sample_ttest_mu_equals_mean_gives_high_p() {
        let df = sample_df();
        // mean is 12.1
        let result =
            compute_ttest_one_sample(&df, "value", 12.1, 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p > 0.5, "p-value should be high when mu0 == mean, got {}", p);
    }

    #[test]
    fn one_sample_ttest_mu_far_from_mean_gives_low_p() {
        let df = sample_df();
        let result =
            compute_ttest_one_sample(&df, "value", 100.0, 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p < 0.001, "p-value should be very low, got {}", p);
    }

    #[test]
    fn one_sample_ttest_df_is_n_minus_one() {
        let df = sample_df();
        let result =
            compute_ttest_one_sample(&df, "value", 12.0, 0.95, Alternative::TwoSided).unwrap();
        let degrees = result.values["df"].as_f64().unwrap();
        assert!((degrees - 9.0).abs() < 1e-10); // n=10, df=9
    }

    #[test]
    fn one_sample_ttest_ci_contains_mean() {
        let df = sample_df();
        let result =
            compute_ttest_one_sample(&df, "value", 12.0, 0.95, Alternative::TwoSided).unwrap();
        let mean = result.values["mean"].as_f64().unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(ci_lower < mean && mean < ci_upper);
    }

    #[test]
    fn one_sample_ttest_rejects_insufficient_data() {
        let df = DataFrame::new(vec![Column::new("value".into(), &[10.0])]).unwrap();
        let result = compute_ttest_one_sample(&df, "value", 10.0, 0.95, Alternative::TwoSided);
        assert!(result.is_err());
    }

    #[test]
    fn one_sample_ttest_rejects_missing_column() {
        let df = sample_df();
        let result = compute_ttest_one_sample(&df, "nonexistent", 0.0, 0.95, Alternative::TwoSided);
        assert!(result.is_err());
    }

    #[test]
    fn one_sample_ttest_effect_size_is_cohens_d() {
        let df = sample_df();
        let result =
            compute_ttest_one_sample(&df, "value", 12.0, 0.95, Alternative::TwoSided).unwrap();
        assert_eq!(
            result.values["effect_size_type"].as_str(),
            Some("cohens_d")
        );
        let d = result.values["effect_size"].as_f64().unwrap();
        // mean=12.1, mu0=12.0, so d should be small positive
        assert!(d > 0.0 && d < 1.0);
    }

    // ── Two-sample t-test ──

    #[test]
    fn two_sample_ttest_returns_correct_structure() {
        let df = two_sample_df();
        let result =
            compute_ttest_two_sample(&df, "score", "group", 0.95, Alternative::TwoSided).unwrap();
        assert!(matches!(result.kind, StatsKind::TTest));
        assert_eq!(
            result.values.get("test").and_then(|v| v.as_str()),
            Some("two_sample_t_test")
        );
        assert!(result.values.contains_key("t_statistic"));
        assert!(result.values.contains_key("p_value"));
        assert!(result.values.contains_key("df"));
        assert!(result.values.contains_key("mean1"));
        assert!(result.values.contains_key("mean2"));
        assert!(result.values.contains_key("ci_lower"));
        assert!(result.values.contains_key("ci_upper"));
    }

    #[test]
    fn two_sample_ttest_detects_difference() {
        let df = two_sample_df();
        let result =
            compute_ttest_two_sample(&df, "score", "group", 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        // Groups A (mean ~12) and B (mean ~20) should differ significantly
        assert!(p < 0.05, "p-value should be significant, got {}", p);
    }

    #[test]
    fn two_sample_ttest_ci_for_difference() {
        let df = two_sample_df();
        let result =
            compute_ttest_two_sample(&df, "score", "group", 0.95, Alternative::TwoSided).unwrap();
        let diff = result.values["mean_diff"].as_f64().unwrap();
        let ci_lower = result.values["ci_lower"].as_f64().unwrap();
        let ci_upper = result.values["ci_upper"].as_f64().unwrap();
        assert!(ci_lower < diff && diff < ci_upper);
    }

    #[test]
    fn two_sample_ttest_rejects_non_two_groups() {
        let df = DataFrame::new(vec![
            Column::new("score".into(), &[1.0, 2.0, 3.0]),
            Column::new("group".into(), &["A", "B", "C"]),
        ])
        .unwrap();
        let result = compute_ttest_two_sample(&df, "score", "group", 0.95, Alternative::TwoSided);
        assert!(result.is_err());
    }
}
