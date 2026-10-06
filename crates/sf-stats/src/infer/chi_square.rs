//! Chi-squared tests: goodness-of-fit and independence.

use polars::prelude::*;
use sf_core::{SfError, SfResult, StatsArtifact, StatsKind};
use std::collections::BTreeMap;

use super::distributions::chi2_upper_p;

/// Chi-squared goodness-of-fit test.
///
/// Tests whether observed frequencies match expected frequencies (or a uniform distribution).
pub fn compute_chisq_goodness(
    observed: &[f64],
    expected: Option<&[f64]>,
) -> SfResult<StatsArtifact> {
    let k = observed.len();
    if k < 2 {
        return Err(SfError::Validation(
            "Goodness-of-fit test requires at least 2 categories".into(),
        ));
    }

    let expected_vals: Vec<f64> = match expected {
        Some(exp) => {
            if exp.len() != k {
                return Err(SfError::Validation(format!(
                    "Observed ({}) and expected ({}) must have same length",
                    k,
                    exp.len()
                )));
            }
            exp.to_vec()
        }
        None => {
            // Uniform expected: total / k for each category
            let total: f64 = observed.iter().sum();
            vec![total / k as f64; k]
        }
    };

    // Check expected values are positive
    if expected_vals.iter().any(|&e| e <= 0.0) {
        return Err(SfError::Validation(
            "Expected frequencies must be positive".into(),
        ));
    }

    let chi2_stat: f64 = observed
        .iter()
        .zip(expected_vals.iter())
        .map(|(o, e)| (o - e).powi(2) / e)
        .sum();

    let df_val = (k - 1) as f64;
    let p_value = chi2_upper_p(chi2_stat, df_val);

    // Cramér's V as effect size (for goodness of fit, V = sqrt(chi2 / (n * (k-1))))
    let n: f64 = observed.iter().sum();
    let cramers_v = if n > 0.0 && df_val > 0.0 {
        (chi2_stat / (n * df_val)).sqrt()
    } else {
        0.0
    };

    let mut values_map = BTreeMap::new();
    values_map.insert("test".into(), serde_json::json!("chi_squared_goodness_of_fit"));
    values_map.insert("k".into(), serde_json::json!(k));
    values_map.insert("observed".into(), serde_json::json!(observed));
    values_map.insert("expected".into(), serde_json::json!(expected_vals));
    values_map.insert("chi2_statistic".into(), serde_json::json!(chi2_stat));
    values_map.insert("df".into(), serde_json::json!(df_val));
    values_map.insert("p_value".into(), serde_json::json!(p_value));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("effect_size".into(), serde_json::json!(cramers_v));
    values_map.insert("effect_size_type".into(), serde_json::json!("cramers_v"));

    Ok(StatsArtifact {
        kind: StatsKind::ChiSquareTest,
        values: values_map,
    })
}

/// Chi-squared test of independence.
///
/// Tests whether two categorical variables in a DataFrame are independent.
pub fn compute_chisq_independence(
    df: &DataFrame,
    col1: &str,
    col2: &str,
) -> SfResult<StatsArtifact> {
    let series1 = df
        .column(col1)
        .map_err(|e| SfError::Execution(format!("Column '{}' not found: {}", col1, e)))?
        .as_materialized_series();
    let series2 = df
        .column(col2)
        .map_err(|e| SfError::Execution(format!("Column '{}' not found: {}", col2, e)))?
        .as_materialized_series();

    let n = df.height();
    if n == 0 {
        return Err(SfError::Validation("DataFrame is empty".into()));
    }

    // Get unique categories
    let cats1 = series1
        .unique()
        .map_err(|e| SfError::Execution(format!("Failed to get unique values for '{}': {}", col1, e)))?;
    let cats2 = series2
        .unique()
        .map_err(|e| SfError::Execution(format!("Failed to get unique values for '{}': {}", col2, e)))?;

    let r = cats1.len();
    let c = cats2.len();
    if r < 2 || c < 2 {
        return Err(SfError::Validation(
            "Independence test requires at least 2 categories in each variable".into(),
        ));
    }

    // Build contingency table
    let mut table = vec![vec![0u64; c]; r];
    let mut row_labels = Vec::with_capacity(r);
    let mut col_labels = Vec::with_capacity(c);

    for i in 0..r {
        row_labels.push(
            cats1
                .get(i)
                .map(|v| format!("{}", v))
                .unwrap_or_else(|_| format!("cat_{}", i)),
        );
    }
    for j in 0..c {
        col_labels.push(
            cats2
                .get(j)
                .map(|v| format!("{}", v))
                .unwrap_or_else(|_| format!("cat_{}", j)),
        );
    }

    // Count frequencies
    for row_idx in 0..n {
        let v1 = series1.get(row_idx).map_err(|e| {
            SfError::Execution(format!("Failed to get value at row {}: {}", row_idx, e))
        })?;
        let v2 = series2.get(row_idx).map_err(|e| {
            SfError::Execution(format!("Failed to get value at row {}: {}", row_idx, e))
        })?;

        let v1_str = format!("{}", v1);
        let v2_str = format!("{}", v2);

        if let Some(i) = row_labels.iter().position(|l| *l == v1_str) {
            if let Some(j) = col_labels.iter().position(|l| *l == v2_str) {
                table[i][j] += 1;
            }
        }
    }

    // Compute row/col totals
    let row_totals: Vec<f64> = table.iter().map(|row| row.iter().sum::<u64>() as f64).collect();
    let col_totals: Vec<f64> = (0..c)
        .map(|j| table.iter().map(|row| row[j]).sum::<u64>() as f64)
        .collect();
    let n_f = n as f64;

    // Chi-squared statistic
    let mut chi2_stat = 0.0;
    for i in 0..r {
        for j in 0..c {
            let expected = row_totals[i] * col_totals[j] / n_f;
            if expected > 0.0 {
                let observed = table[i][j] as f64;
                chi2_stat += (observed - expected).powi(2) / expected;
            }
        }
    }

    let df_val = ((r - 1) * (c - 1)) as f64;
    let p_value = chi2_upper_p(chi2_stat, df_val);

    // Cramér's V
    let min_dim = (r - 1).min(c - 1) as f64;
    let cramers_v = if n_f > 0.0 && min_dim > 0.0 {
        (chi2_stat / (n_f * min_dim)).sqrt()
    } else {
        0.0
    };

    // Convert table to JSON
    let table_json: Vec<Vec<u64>> = table;

    let mut values_map = BTreeMap::new();
    values_map.insert("test".into(), serde_json::json!("chi_squared_independence"));
    values_map.insert("column1".into(), serde_json::json!(col1));
    values_map.insert("column2".into(), serde_json::json!(col2));
    values_map.insert("row_labels".into(), serde_json::json!(row_labels));
    values_map.insert("col_labels".into(), serde_json::json!(col_labels));
    values_map.insert("contingency_table".into(), serde_json::json!(table_json));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("rows".into(), serde_json::json!(r));
    values_map.insert("cols".into(), serde_json::json!(c));
    values_map.insert("chi2_statistic".into(), serde_json::json!(chi2_stat));
    values_map.insert("df".into(), serde_json::json!(df_val));
    values_map.insert("p_value".into(), serde_json::json!(p_value));
    values_map.insert("effect_size".into(), serde_json::json!(cramers_v));
    values_map.insert("effect_size_type".into(), serde_json::json!("cramers_v"));

    Ok(StatsArtifact {
        kind: StatsKind::ChiSquareTest,
        values: values_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Goodness of fit ──

    #[test]
    fn chisq_goodness_uniform_returns_correct_structure() {
        let observed = vec![20.0, 20.0, 20.0, 20.0, 20.0];
        let result = compute_chisq_goodness(&observed, None).unwrap();
        assert!(matches!(result.kind, StatsKind::ChiSquareTest));
        assert_eq!(
            result.values["test"].as_str(),
            Some("chi_squared_goodness_of_fit")
        );
    }

    #[test]
    fn chisq_goodness_uniform_high_p_for_equal_freqs() {
        let observed = vec![20.0, 20.0, 20.0, 20.0, 20.0];
        let result = compute_chisq_goodness(&observed, None).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!((p - 1.0).abs() < 1e-10, "p should be 1.0 for perfectly uniform, got {}", p);
    }

    #[test]
    fn chisq_goodness_low_p_for_skewed_freqs() {
        let observed = vec![90.0, 5.0, 3.0, 1.0, 1.0];
        let result = compute_chisq_goodness(&observed, None).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p < 0.001, "p should be very low for skewed data, got {}", p);
    }

    #[test]
    fn chisq_goodness_with_expected_values() {
        let observed = vec![50.0, 30.0, 20.0];
        let expected = vec![40.0, 40.0, 20.0];
        let result = compute_chisq_goodness(&observed, Some(&expected)).unwrap();
        assert!(result.values.contains_key("chi2_statistic"));
        assert!(result.values.contains_key("p_value"));
    }

    #[test]
    fn chisq_goodness_rejects_mismatched_lengths() {
        let observed = vec![50.0, 30.0, 20.0];
        let expected = vec![40.0, 40.0];
        let result = compute_chisq_goodness(&observed, Some(&expected));
        assert!(result.is_err());
    }

    #[test]
    fn chisq_goodness_df_is_k_minus_one() {
        let observed = vec![20.0, 30.0, 50.0];
        let result = compute_chisq_goodness(&observed, None).unwrap();
        let df = result.values["df"].as_f64().unwrap();
        assert!((df - 2.0).abs() < 1e-10);
    }

    // ── Independence ──

    #[test]
    fn chisq_independence_returns_correct_structure() {
        let df = DataFrame::new(vec![
            Column::new("gender".into(), &["M", "M", "F", "F", "M", "F", "M", "F"]),
            Column::new(
                "preference".into(),
                &["A", "B", "A", "A", "B", "B", "A", "B"],
            ),
        ])
        .unwrap();
        let result = compute_chisq_independence(&df, "gender", "preference").unwrap();
        assert!(matches!(result.kind, StatsKind::ChiSquareTest));
        assert_eq!(
            result.values["test"].as_str(),
            Some("chi_squared_independence")
        );
        assert!(result.values.contains_key("contingency_table"));
        assert!(result.values.contains_key("chi2_statistic"));
        assert!(result.values.contains_key("p_value"));
        assert!(result.values.contains_key("effect_size"));
    }

    #[test]
    fn chisq_independence_detects_association() {
        // Strong association: M always A, F always B
        let df = DataFrame::new(vec![
            Column::new(
                "gender".into(),
                &["M", "M", "M", "M", "M", "F", "F", "F", "F", "F"],
            ),
            Column::new(
                "choice".into(),
                &["A", "A", "A", "A", "A", "B", "B", "B", "B", "B"],
            ),
        ])
        .unwrap();
        let result = compute_chisq_independence(&df, "gender", "choice").unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p < 0.05, "p should be significant for strong association, got {}", p);
    }

    #[test]
    fn chisq_independence_df_is_r_minus1_times_c_minus1() {
        let df = DataFrame::new(vec![
            Column::new("x".into(), &["A", "A", "B", "B", "C", "C"]),
            Column::new("y".into(), &["X", "Y", "X", "Y", "X", "Y"]),
        ])
        .unwrap();
        let result = compute_chisq_independence(&df, "x", "y").unwrap();
        let df_val = result.values["df"].as_f64().unwrap();
        // 3 rows * 2 cols -> df = (3-1)(2-1) = 2
        assert!((df_val - 2.0).abs() < 1e-10);
    }
}
