use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Compute OLS regression: y ~ X with intercept.
///
/// Returns coefficients, standard errors, t-values, p-values, confidence intervals,
/// R², adjusted R², F-statistic, residuals, and predicted values.
pub fn compute_regression(
    df: &DataFrame,
    target: &str,
    predictors: &[String],
    confidence_level: f64,
) -> SfResult<StatsArtifact> {
    if predictors.is_empty() {
        return Err(SfError::Validation(
            "Regression requires at least one predictor column".into(),
        ));
    }

    // Extract target vector
    let y = extract_f64_column(df, target)?;
    let n = y.len();
    let p = predictors.len() + 1; // +1 for intercept

    if n <= p {
        return Err(SfError::Validation(format!(
            "Not enough observations ({}) for {} parameters",
            n, p
        )));
    }

    // Build design matrix X (n × p) with intercept column
    let mut x_mat = vec![vec![0.0; p]; n];
    for i in 0..n {
        x_mat[i][0] = 1.0; // intercept
    }
    for (j, pred_name) in predictors.iter().enumerate() {
        let col_vals = extract_f64_column(df, pred_name)?;
        if col_vals.len() != n {
            return Err(SfError::Schema(format!(
                "Column '{}' has {} rows, expected {}",
                pred_name,
                col_vals.len(),
                n
            )));
        }
        for i in 0..n {
            x_mat[i][j + 1] = col_vals[i];
        }
    }

    // Compute X'X (p × p)
    let xtx = mat_mul_ata(&x_mat, n, p);

    // Compute X'y (p × 1)
    let xty = mat_mul_atb_vec(&x_mat, &y, n, p);

    // Solve (X'X)^{-1}
    let xtx_inv = mat_invert(&xtx, p)?;

    // Coefficients: β = (X'X)^{-1} X'y
    let beta = mat_mul_mv(&xtx_inv, &xty, p);

    // Predictions: ŷ = Xβ
    let y_hat: Vec<f64> = (0..n)
        .map(|i| (0..p).map(|j| x_mat[i][j] * beta[j]).sum())
        .collect();

    // Residuals: e = y - ŷ
    let residuals: Vec<f64> = y.iter().zip(y_hat.iter()).map(|(yi, yhi)| yi - yhi).collect();

    // SSE, SSR, SST
    let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
    let sse: f64 = residuals.iter().map(|e| e * e).sum();
    let sst: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
    let ssr = sst - sse;

    // R² and adjusted R²
    let r_squared = if sst > 0.0 { 1.0 - sse / sst } else { 0.0 };
    let df_resid = (n - p) as f64;
    let adj_r_squared = if sst > 0.0 && n > p {
        1.0 - (sse / df_resid) / (sst / (n as f64 - 1.0))
    } else {
        0.0
    };

    // MSE (mean squared error of residuals)
    let mse = sse / df_resid;

    // F-statistic: (SSR / (p-1)) / (SSE / (n-p))
    let df_model = (p - 1) as f64;
    let f_statistic = if df_model > 0.0 && mse > 0.0 {
        (ssr / df_model) / mse
    } else {
        0.0
    };

    // Standard errors, t-values, p-values, confidence intervals
    let alpha = 1.0 - confidence_level;
    let t_crit = t_quantile(1.0 - alpha / 2.0, (n - p) as f64);

    let mut coefficients = Vec::new();
    let names: Vec<&str> = std::iter::once("(Intercept)")
        .chain(predictors.iter().map(|s| s.as_str()))
        .collect();

    for j in 0..p {
        let se = (mse * xtx_inv[j][j]).sqrt();
        let t_val = if se > 0.0 { beta[j] / se } else { 0.0 };
        let p_val = 2.0 * (1.0 - t_cdf(t_val.abs(), (n - p) as f64));
        let ci_lower = beta[j] - t_crit * se;
        let ci_upper = beta[j] + t_crit * se;

        coefficients.push(serde_json::json!({
            "name": names[j],
            "estimate": beta[j],
            "std_error": se,
            "t_value": t_val,
            "p_value": p_val,
            "ci_lower": ci_lower,
            "ci_upper": ci_upper,
        }));
    }

    let mut values = BTreeMap::new();
    values.insert(
        "coefficients".to_string(),
        serde_json::to_value(&coefficients)?,
    );
    values.insert("r_squared".to_string(), serde_json::json!(r_squared));
    values.insert(
        "adj_r_squared".to_string(),
        serde_json::json!(adj_r_squared),
    );
    values.insert("f_statistic".to_string(), serde_json::json!(f_statistic));
    values.insert("df_model".to_string(), serde_json::json!(df_model as u64));
    values.insert("df_resid".to_string(), serde_json::json!(df_resid as u64));
    values.insert("mse".to_string(), serde_json::json!(mse));
    values.insert("n".to_string(), serde_json::json!(n));
    values.insert(
        "confidence_level".to_string(),
        serde_json::json!(confidence_level),
    );
    values.insert("target".to_string(), serde_json::json!(target));
    values.insert(
        "predictors".to_string(),
        serde_json::to_value(predictors)?,
    );
    values.insert("predictions".to_string(), serde_json::to_value(&y_hat)?);
    values.insert("residuals".to_string(), serde_json::to_value(&residuals)?);

    Ok(StatsArtifact {
        kind: StatsKind::Regression,
        values,
    })
}

// ── Helper: extract column as f64 vec ──────────────

pub(crate) fn extract_f64_column(df: &DataFrame, name: &str) -> SfResult<Vec<f64>> {
    let col = df
        .column(name)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", name)))?;
    let series = col.as_materialized_series();
    let casted = series.cast(&DataType::Float64)?;
    Ok(casted.f64()?.into_no_null_iter().collect())
}

// ── Linear algebra helpers ─────────────────────────

/// Compute A'A where A is n×p, result is p×p.
fn mat_mul_ata(a: &[Vec<f64>], n: usize, p: usize) -> Vec<Vec<f64>> {
    let mut result = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in i..p {
            let mut sum = 0.0;
            for k in 0..n {
                sum += a[k][i] * a[k][j];
            }
            result[i][j] = sum;
            result[j][i] = sum;
        }
    }
    result
}

/// Compute A'b where A is n×p and b is n×1, result is p×1.
fn mat_mul_atb_vec(a: &[Vec<f64>], b: &[f64], n: usize, p: usize) -> Vec<f64> {
    let mut result = vec![0.0; p];
    for j in 0..p {
        let mut sum = 0.0;
        for i in 0..n {
            sum += a[i][j] * b[i];
        }
        result[j] = sum;
    }
    result
}

/// Multiply p×p matrix by p×1 vector.
fn mat_mul_mv(m: &[Vec<f64>], v: &[f64], p: usize) -> Vec<f64> {
    let mut result = vec![0.0; p];
    for i in 0..p {
        let mut sum = 0.0;
        for j in 0..p {
            sum += m[i][j] * v[j];
        }
        result[i] = sum;
    }
    result
}

/// Invert a p×p matrix using Gauss-Jordan elimination.
pub(crate) fn mat_invert(m: &[Vec<f64>], p: usize) -> SfResult<Vec<Vec<f64>>> {
    // Augmented matrix [M | I]
    let mut aug = vec![vec![0.0; 2 * p]; p];
    for i in 0..p {
        for j in 0..p {
            aug[i][j] = m[i][j];
        }
        aug[i][p + i] = 1.0;
    }

    for col in 0..p {
        // Partial pivoting
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..p {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-14 {
            return Err(SfError::Execution(
                "Matrix is singular or near-singular; cannot compute regression".into(),
            ));
        }
        aug.swap(col, max_row);

        // Scale pivot row
        let pivot = aug[col][col];
        for j in 0..(2 * p) {
            aug[col][j] /= pivot;
        }

        // Eliminate column
        for row in 0..p {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            for j in 0..(2 * p) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Extract inverse from right half
    let mut inv = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in 0..p {
            inv[i][j] = aug[i][p + j];
        }
    }
    Ok(inv)
}

// ── t-distribution CDF ────────────────────────────

/// Approximate CDF of the t-distribution using the regularized incomplete beta function.
/// P(T ≤ t) for T ~ t(df).
pub(crate) fn t_cdf(t: f64, df: f64) -> f64 {
    if df <= 0.0 {
        return 0.5;
    }
    let x = df / (df + t * t);
    let ib = regularized_incomplete_beta(df / 2.0, 0.5, x);
    if t >= 0.0 {
        1.0 - 0.5 * ib
    } else {
        0.5 * ib
    }
}

/// Approximate the quantile of the t-distribution (inverse CDF).
/// Uses bisection for simplicity.
pub(crate) fn t_quantile(prob: f64, df: f64) -> f64 {
    if prob <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if prob >= 1.0 {
        return f64::INFINITY;
    }
    if (prob - 0.5).abs() < 1e-15 {
        return 0.0;
    }

    let mut lo = -100.0_f64;
    let mut hi = 100.0_f64;
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if t_cdf(mid, df) < prob {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Regularized incomplete beta function I_x(a, b) using continued fraction expansion.
fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use symmetry relation when x > (a+1)/(a+b+2) for better convergence
    if x > (a + 1.0) / (a + b + 2.0) {
        return 1.0 - regularized_incomplete_beta(b, a, 1.0 - x);
    }

    let ln_prefix = ln_beta_prefix(a, b, x);
    let cf = beta_continued_fraction(a, b, x);

    (ln_prefix + cf.ln()).exp() / a
}

/// Compute ln(x^a * (1-x)^b / B(a,b))
fn ln_beta_prefix(a: f64, b: f64, x: f64) -> f64 {
    a * x.ln() + b * (1.0 - x).ln() - ln_beta(a, b)
}

/// Compute ln(B(a,b)) = ln(Γ(a)) + ln(Γ(b)) - ln(Γ(a+b))
fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
}

/// Stirling's approximation for ln(Γ(x)) — Lanczos approximation.
fn ln_gamma(x: f64) -> f64 {
    // Lanczos approximation with g=7
    let coefficients = [
        0.999_999_999_999_809_9,
        676.5203681218851,
        -1259.1392167224028,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507343278686905,
        -0.13857109526572012,
        9.984_369_578_019_572e-6,
        1.5056327351493116e-7,
    ];

    if x < 0.5 {
        let s = coefficients[1..]
            .iter()
            .enumerate()
            .fold(coefficients[0], |acc, (i, &c)| acc + c / (i as f64 + 1.0 - x));
        std::f64::consts::PI.ln()
            - (std::f64::consts::PI * x).sin().ln()
            - s.ln()
            - (2.0 * std::f64::consts::PI).ln() / 2.0
            - (0.5 - x + 7.5) * (0.5 - x + 7.5).ln()
            + (0.5 - x + 7.5)
    } else {
        let x = x - 1.0;
        let s = coefficients[1..]
            .iter()
            .enumerate()
            .fold(coefficients[0], |acc, (i, &c)| acc + c / (x + i as f64 + 1.0));
        (2.0 * std::f64::consts::PI).ln() / 2.0 + s.ln() + (x + 7.5).ln() * (x + 0.5)
            - (x + 7.5)
    }
}

/// Continued fraction for the regularized incomplete beta function (Lentz's method).
fn beta_continued_fraction(a: f64, b: f64, x: f64) -> f64 {
    let max_iter = 200;
    let eps = 1e-14;
    let tiny = 1e-30;

    let mut c = 1.0;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1.0 / d;
    let mut f = d;

    for m in 1..=max_iter {
        let m_f64 = m as f64;

        // Even step: d_{2m}
        let num = m_f64 * (b - m_f64) * x / ((a + 2.0 * m_f64 - 1.0) * (a + 2.0 * m_f64));
        d = 1.0 + num * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + num / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        f *= c * d;

        // Odd step: d_{2m+1}
        let num = -(a + m_f64) * (a + b + m_f64) * x
            / ((a + 2.0 * m_f64) * (a + 2.0 * m_f64 + 1.0));
        d = 1.0 + num * d;
        if d.abs() < tiny {
            d = tiny;
        }
        c = 1.0 + num / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < eps {
            break;
        }
    }

    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_df() -> DataFrame {
        // y = 2 + 3*x (perfect linear relationship)
        df! {
            "x" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "y" => &[5.0_f64, 8.0, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0, 29.0, 32.0],
        }
        .unwrap()
    }

    fn noisy_df() -> DataFrame {
        // y ≈ 1 + 2*x1 + 3*x2 with some noise
        df! {
            "x1" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "x2" => &[2.0_f64, 1.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0, 10.0, 9.0],
            "y" => &[9.1_f64, 8.2, 19.8, 18.5, 29.2, 27.9, 39.1, 38.2, 49.8, 48.3],
        }
        .unwrap()
    }

    #[test]
    fn simple_regression_returns_correct_coefficients() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();

        let coeffs = result
            .values
            .get("coefficients")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(coeffs.len(), 2); // intercept + 1 predictor

        let intercept = coeffs[0]["estimate"].as_f64().unwrap();
        let slope = coeffs[1]["estimate"].as_f64().unwrap();
        assert!((intercept - 2.0).abs() < 1e-8, "intercept={}", intercept);
        assert!((slope - 3.0).abs() < 1e-8, "slope={}", slope);
    }

    #[test]
    fn perfect_fit_returns_r_squared_one() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();
        let r2 = result.values.get("r_squared").unwrap().as_f64().unwrap();
        assert!((r2 - 1.0).abs() < 1e-10, "r²={}", r2);
    }

    #[test]
    fn multiple_regression_returns_correct_parameter_count() {
        let df = noisy_df();
        let result =
            compute_regression(&df, "y", &["x1".into(), "x2".into()], 0.95).unwrap();

        let coeffs = result
            .values
            .get("coefficients")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(coeffs.len(), 3); // intercept + 2 predictors
    }

    #[test]
    fn coefficients_have_all_required_fields() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();

        let coeffs = result
            .values
            .get("coefficients")
            .unwrap()
            .as_array()
            .unwrap();
        for coeff in coeffs {
            assert!(coeff.get("name").is_some());
            assert!(coeff.get("estimate").is_some());
            assert!(coeff.get("std_error").is_some());
            assert!(coeff.get("t_value").is_some());
            assert!(coeff.get("p_value").is_some());
            assert!(coeff.get("ci_lower").is_some());
            assert!(coeff.get("ci_upper").is_some());
        }
    }

    #[test]
    fn perfect_fit_has_near_zero_p_values() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();

        let coeffs = result
            .values
            .get("coefficients")
            .unwrap()
            .as_array()
            .unwrap();
        let slope_p = coeffs[1]["p_value"].as_f64().unwrap();
        assert!(slope_p < 1e-6, "slope p-value={}", slope_p);
    }

    #[test]
    fn predictions_match_observed_for_perfect_fit() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();

        let preds = result
            .values
            .get("predictions")
            .unwrap()
            .as_array()
            .unwrap();
        let expected = vec![5.0, 8.0, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0, 29.0, 32.0];
        for (pred, exp) in preds.iter().zip(expected.iter()) {
            assert!(
                (pred.as_f64().unwrap() - exp).abs() < 1e-8,
                "pred={}, expected={}",
                pred,
                exp
            );
        }
    }

    #[test]
    fn residuals_near_zero_for_perfect_fit() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();

        let resids = result
            .values
            .get("residuals")
            .unwrap()
            .as_array()
            .unwrap();
        for r in resids {
            assert!(r.as_f64().unwrap().abs() < 1e-8);
        }
    }

    #[test]
    fn regression_rejects_empty_predictors() {
        let df = test_df();
        let result = compute_regression(&df, "y", &[], 0.95);
        assert!(result.is_err());
    }

    #[test]
    fn regression_rejects_insufficient_observations() {
        let df = df! {
            "x" => &[1.0_f64],
            "y" => &[2.0_f64],
        }
        .unwrap();
        let result = compute_regression(&df, "y", &["x".into()], 0.95);
        assert!(result.is_err());
    }

    #[test]
    fn t_cdf_returns_half_at_zero() {
        let p = t_cdf(0.0, 10.0);
        assert!((p - 0.5).abs() < 1e-10, "t_cdf(0, 10) = {}", p);
    }

    #[test]
    fn t_cdf_returns_values_in_valid_range() {
        for &df in &[1.0, 5.0, 10.0, 30.0, 100.0] {
            for &t in &[-3.0, -1.0, 0.0, 1.0, 3.0] {
                let p = t_cdf(t, df);
                assert!((0.0..=1.0).contains(&p), "t_cdf({}, {}) = {}", t, df, p);
            }
        }
    }

    #[test]
    fn t_quantile_is_inverse_of_t_cdf() {
        let df = 10.0;
        for &prob in &[0.025, 0.05, 0.5, 0.95, 0.975] {
            let t = t_quantile(prob, df);
            let p = t_cdf(t, df);
            assert!((p - prob).abs() < 1e-6, "t_quantile({}, {}) = {}, cdf = {}", prob, df, t, p);
        }
    }

    #[test]
    fn confidence_interval_contains_true_value_for_perfect_fit() {
        let df = test_df();
        let result = compute_regression(&df, "y", &["x".into()], 0.95).unwrap();

        let coeffs = result
            .values
            .get("coefficients")
            .unwrap()
            .as_array()
            .unwrap();

        // Intercept = 2.0
        let ci_lo = coeffs[0]["ci_lower"].as_f64().unwrap();
        let ci_hi = coeffs[0]["ci_upper"].as_f64().unwrap();
        assert!(ci_lo <= 2.0 && ci_hi >= 2.0);

        // Slope = 3.0
        let ci_lo = coeffs[1]["ci_lower"].as_f64().unwrap();
        let ci_hi = coeffs[1]["ci_upper"].as_f64().unwrap();
        assert!(ci_lo <= 3.0 && ci_hi >= 3.0);
    }

    #[test]
    fn mat_invert_returns_identity_product() {
        let m = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let inv = mat_invert(&m, 2).unwrap();

        // M * M^{-1} ≈ I
        for i in 0..2 {
            for j in 0..2 {
                let val: f64 = (0..2).map(|k| m[i][k] * inv[k][j]).sum();
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((val - expected).abs() < 1e-10);
            }
        }
    }
}
