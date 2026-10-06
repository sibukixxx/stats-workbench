use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Augmented Dickey-Fuller test for stationarity.
///
/// Tests H0: unit root exists (non-stationary) vs H1: stationary.
/// Returns test statistic, critical values, and lag used.
pub fn compute_adf(
    df: &DataFrame,
    time_col: &str,
    value_col: &str,
    max_lag: usize,
) -> SfResult<StatsArtifact> {
    // Validate columns exist
    let _ = df
        .column(time_col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", time_col)))?;
    let value_series = df
        .column(value_col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", value_col)))?;

    // Extract values as f64
    let values = series_to_f64_vec(value_series.as_materialized_series())?;
    let n = values.len();

    if n < 4 {
        return Err(SfError::Validation(
            "ADF test requires at least 4 observations".into(),
        ));
    }

    // Determine optimal lag (use min of max_lag and n/3)
    let effective_max_lag = max_lag.min(n / 3);
    let lag = select_lag_aic(&values, effective_max_lag);

    // Compute ADF test statistic
    let (t_stat, n_obs) = adf_test_statistic(&values, lag);

    // Critical values for ADF test (with constant, no trend)
    // Approximation from MacKinnon (1994)
    let critical_values = mackinnon_critical_values(n_obs);

    // Determine stationarity
    let is_stationary = t_stat < critical_values[1]; // 5% level

    let mut values_map = BTreeMap::new();
    values_map.insert(
        "test_statistic".to_string(),
        serde_json::json!(t_stat),
    );
    values_map.insert("lag_used".to_string(), serde_json::json!(lag));
    values_map.insert("n_obs".to_string(), serde_json::json!(n_obs));
    values_map.insert(
        "critical_values".to_string(),
        serde_json::json!({
            "1%": critical_values[0],
            "5%": critical_values[1],
            "10%": critical_values[2],
        }),
    );
    values_map.insert(
        "is_stationary_5pct".to_string(),
        serde_json::json!(is_stationary),
    );
    values_map.insert(
        "null_hypothesis".to_string(),
        serde_json::json!("Unit root exists (non-stationary)"),
    );

    Ok(StatsArtifact {
        kind: StatsKind::Adf,
        values: values_map,
    })
}

/// Compute ADF test statistic using OLS regression:
/// Δy_t = α + γ * y_{t-1} + Σ(β_i * Δy_{t-i}) + ε_t
/// Test statistic = γ / SE(γ)
fn adf_test_statistic(y: &[f64], lag: usize) -> (f64, usize) {
    // Compute first differences
    let dy: Vec<f64> = y.windows(2).map(|w| w[1] - w[0]).collect();

    // Build regression: Δy_t = α + γ * y_{t-1} + Σ(β_i * Δy_{t-i}) + ε
    // Starting index: lag + 1 (to have enough lagged differences)
    let start = lag;
    let n_obs = dy.len() - start;

    if n_obs < lag + 3 {
        return (0.0, n_obs);
    }

    // Number of regressors: constant (1) + y_{t-1} (1) + lag differences
    let k = 2 + lag;

    // Build X matrix and y vector
    let mut x_matrix: Vec<Vec<f64>> = Vec::with_capacity(n_obs);
    let mut y_vec: Vec<f64> = Vec::with_capacity(n_obs);

    for t in start..dy.len() {
        let mut row = Vec::with_capacity(k);
        row.push(1.0); // constant
        row.push(y[t]); // y_{t-1} (since dy[t] = y[t+1] - y[t], y_{t-1} for dy[t] is y[t])
        for j in 1..=lag {
            row.push(dy[t - j]); // Δy_{t-j}
        }
        x_matrix.push(row);
        y_vec.push(dy[t]);
    }

    // OLS: β = (X'X)^{-1} X'y
    let (coeffs, se) = ols_with_se(&x_matrix, &y_vec, k);

    if coeffs.is_empty() || se.is_empty() || se[1].abs() < 1e-15 {
        return (0.0, n_obs);
    }

    // t-statistic for γ (coefficient index 1)
    let t_stat = coeffs[1] / se[1];

    (t_stat, n_obs)
}

/// OLS regression returning coefficients and standard errors.
fn ols_with_se(x: &[Vec<f64>], y: &[f64], k: usize) -> (Vec<f64>, Vec<f64>) {
    let n = x.len();
    if n <= k {
        return (vec![], vec![]);
    }

    // X'X
    let mut xtx = vec![vec![0.0; k]; k];
    for row in x {
        for i in 0..k {
            for j in 0..k {
                xtx[i][j] += row[i] * row[j];
            }
        }
    }

    // X'y
    let mut xty = vec![0.0; k];
    for (i, row) in x.iter().enumerate() {
        for j in 0..k {
            xty[j] += row[j] * y[i];
        }
    }

    // Solve via Cholesky-like approach (simple Gaussian elimination for small k)
    let Some(xtx_inv) = invert_matrix(&xtx) else {
        return (vec![], vec![]);
    };

    // β = (X'X)^{-1} X'y
    let mut coeffs = vec![0.0; k];
    for i in 0..k {
        for j in 0..k {
            coeffs[i] += xtx_inv[i][j] * xty[j];
        }
    }

    // Residuals and σ²
    let mut rss = 0.0;
    for (i, row) in x.iter().enumerate() {
        let predicted: f64 = row.iter().zip(&coeffs).map(|(xi, bi)| xi * bi).sum();
        let resid = y[i] - predicted;
        rss += resid * resid;
    }
    let sigma2 = rss / (n - k) as f64;

    // SE = sqrt(diag(σ² * (X'X)^{-1}))
    let se: Vec<f64> = (0..k).map(|i| (sigma2 * xtx_inv[i][i]).sqrt()).collect();

    (coeffs, se)
}

/// Invert a small square matrix using Gauss-Jordan elimination.
fn invert_matrix(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = m.len();
    let mut aug: Vec<Vec<f64>> = m
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut r = row.clone();
            r.extend(std::iter::repeat(0.0).take(n));
            r[n + i] = 1.0;
            r
        })
        .collect();

    for col in 0..n {
        // Partial pivoting
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None; // Singular
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        for j in 0..(2 * n) {
            aug[col][j] /= pivot;
        }

        for row in 0..n {
            if row != col {
                let factor = aug[row][col];
                for j in 0..(2 * n) {
                    aug[row][j] -= factor * aug[col][j];
                }
            }
        }
    }

    Some(aug.iter().map(|row| row[n..].to_vec()).collect())
}

/// Select optimal lag using AIC (simplified).
fn select_lag_aic(y: &[f64], max_lag: usize) -> usize {
    if max_lag == 0 {
        return 0;
    }

    let mut best_lag = 0;
    let mut best_aic = f64::INFINITY;

    for lag in 0..=max_lag {
        let dy: Vec<f64> = y.windows(2).map(|w| w[1] - w[0]).collect();
        let start = lag;
        let n_obs = dy.len().saturating_sub(start);
        let k = 2 + lag;

        if n_obs <= k + 1 {
            continue;
        }

        let mut x_matrix: Vec<Vec<f64>> = Vec::with_capacity(n_obs);
        let mut y_vec: Vec<f64> = Vec::with_capacity(n_obs);

        for t in start..dy.len() {
            let mut row = Vec::with_capacity(k);
            row.push(1.0);
            row.push(y[t]);
            for j in 1..=lag {
                row.push(dy[t - j]);
            }
            x_matrix.push(row);
            y_vec.push(dy[t]);
        }

        let (coeffs, _) = ols_with_se(&x_matrix, &y_vec, k);
        if coeffs.is_empty() {
            continue;
        }

        let mut rss = 0.0;
        for (i, row) in x_matrix.iter().enumerate() {
            let predicted: f64 = row.iter().zip(&coeffs).map(|(xi, bi)| xi * bi).sum();
            let resid = y_vec[i] - predicted;
            rss += resid * resid;
        }

        let n_f = n_obs as f64;
        let k_f = k as f64;
        let aic = n_f * (rss / n_f).ln() + 2.0 * k_f;

        if aic < best_aic {
            best_aic = aic;
            best_lag = lag;
        }
    }

    best_lag
}

/// MacKinnon (1994) approximate critical values for ADF test with constant.
fn mackinnon_critical_values(n: usize) -> [f64; 3] {
    let n_f = n as f64;
    // Approximations for constant-only case
    [
        -3.43035 - 6.5393 / n_f - 16.786 / (n_f * n_f),   // 1%
        -2.86154 - 2.7882 / n_f - 3.5529 / (n_f * n_f),   // 5%
        -2.56677 - 1.5384 / n_f - 2.8099 / (n_f * n_f),   // 10%
    ]
}

fn series_to_f64_vec(s: &Series) -> SfResult<Vec<f64>> {
    let casted = s
        .cast(&DataType::Float64)
        .map_err(|e| SfError::Execution(format!("Cannot cast '{}' to f64: {}", s.name(), e)))?;
    let ca = casted.f64().map_err(|e| {
        SfError::Execution(format!("Cannot extract f64 from '{}': {}", s.name(), e))
    })?;

    let values: Vec<f64> = ca
        .into_no_null_iter()
        .collect();

    if values.len() != s.len() - s.null_count() {
        // Filter nulls: use only non-null values
        Ok(values)
    } else {
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ts_df(values: &[f64]) -> DataFrame {
        let n = values.len();
        let time: Vec<i64> = (0..n as i64).collect();
        df! {
            "time" => &time,
            "value" => values,
        }
        .unwrap()
    }

    #[test]
    fn adf_returns_stationary_for_white_noise() {
        // White noise (mean-zero, stationary)
        let values: Vec<f64> = vec![
            0.5, -0.3, 0.8, -0.1, 0.4, -0.6, 0.2, -0.4, 0.7, -0.2,
            0.3, -0.5, 0.6, -0.3, 0.1, -0.7, 0.4, -0.2, 0.5, -0.1,
            0.2, -0.4, 0.3, -0.6, 0.1, -0.3, 0.5, -0.2, 0.4, -0.5,
        ];
        let df = make_ts_df(&values);
        let result = compute_adf(&df, "time", "value", 4).unwrap();

        assert!(matches!(result.kind, StatsKind::Adf));
        assert!(result.values.contains_key("test_statistic"));
        assert!(result.values.contains_key("critical_values"));
        assert!(result.values.contains_key("is_stationary_5pct"));

        let is_stationary = result.values["is_stationary_5pct"].as_bool().unwrap();
        assert!(is_stationary, "White noise should be stationary");
    }

    #[test]
    fn adf_returns_non_stationary_for_random_walk() {
        // Random walk: y_t = y_{t-1} + noise (non-stationary)
        let mut values = vec![0.0_f64];
        let increments = [
            0.5, 0.3, 0.8, 0.1, 0.4, 0.6, 0.2, 0.4, 0.7, 0.2,
            0.3, 0.5, 0.6, 0.3, 0.1, 0.7, 0.4, 0.2, 0.5, 0.1,
            0.2, 0.4, 0.3, 0.6, 0.1, 0.3, 0.5, 0.2, 0.4, 0.5,
        ];
        for inc in &increments {
            let last = *values.last().unwrap();
            values.push(last + inc);
        }
        let df = make_ts_df(&values);
        let result = compute_adf(&df, "time", "value", 4).unwrap();

        let is_stationary = result.values["is_stationary_5pct"].as_bool().unwrap();
        assert!(!is_stationary, "Random walk should be non-stationary");
    }

    #[test]
    fn adf_rejects_missing_column() {
        let df = make_ts_df(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = compute_adf(&df, "time", "missing", 2);
        assert!(result.is_err());
    }

    #[test]
    fn adf_rejects_too_few_observations() {
        let df = make_ts_df(&[1.0, 2.0, 3.0]);
        let result = compute_adf(&df, "time", "value", 1);
        assert!(result.is_err());
    }

    #[test]
    fn adf_has_correct_structure() {
        let values: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin()).collect();
        let df = make_ts_df(&values);
        let result = compute_adf(&df, "time", "value", 4).unwrap();

        let cv = result.values.get("critical_values").unwrap();
        assert!(cv.get("1%").is_some());
        assert!(cv.get("5%").is_some());
        assert!(cv.get("10%").is_some());

        let lag = result.values["lag_used"].as_u64().unwrap();
        assert!(lag <= 4);
    }
}
