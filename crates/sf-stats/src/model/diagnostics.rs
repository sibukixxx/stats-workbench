use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

use super::regression::{extract_f64_column, mat_invert};

/// Compute regression diagnostics: residual stats, VIF, Durbin-Watson, leverage, Cook's distance.
pub fn compute_regression_diagnostics(
    df: &DataFrame,
    target: &str,
    predictors: &[String],
) -> SfResult<StatsArtifact> {
    if predictors.is_empty() {
        return Err(SfError::Validation(
            "Diagnostics requires at least one predictor column".into(),
        ));
    }

    let y = extract_f64_column(df, target)?;
    let n = y.len();
    let p = predictors.len() + 1; // +1 for intercept

    if n <= p {
        return Err(SfError::Validation(format!(
            "Not enough observations ({}) for {} parameters",
            n, p
        )));
    }

    // Build design matrix
    let mut x_mat = vec![vec![0.0; p]; n];
    for row in &mut x_mat {
        row[0] = 1.0;
    }
    for (j, pred_name) in predictors.iter().enumerate() {
        let col_vals = extract_f64_column(df, pred_name)?;
        for i in 0..n {
            x_mat[i][j + 1] = col_vals[i];
        }
    }

    // X'X and its inverse
    let xtx = mat_mul_ata(&x_mat, n, p);
    let xtx_inv = mat_invert(&xtx, p)?;

    // X'y
    let xty: Vec<f64> = (0..p)
        .map(|j| (0..n).map(|i| x_mat[i][j] * y[i]).sum())
        .collect();

    // Coefficients
    let beta: Vec<f64> = (0..p)
        .map(|i| (0..p).map(|j| xtx_inv[i][j] * xty[j]).sum())
        .collect();

    // Predictions and residuals
    let y_hat: Vec<f64> = (0..n)
        .map(|i| (0..p).map(|j| x_mat[i][j] * beta[j]).sum())
        .collect();
    let residuals: Vec<f64> = y
        .iter()
        .zip(y_hat.iter())
        .map(|(yi, yhi)| yi - yhi)
        .collect();

    let sse: f64 = residuals.iter().map(|e| e * e).sum();
    let mse = sse / (n - p) as f64;

    // ── Hat matrix diagonal (leverage) ──
    // h_ii = x_i' (X'X)^{-1} x_i
    let leverage: Vec<f64> = (0..n)
        .map(|i| {
            let mut h = 0.0;
            for a in 0..p {
                for b in 0..p {
                    h += x_mat[i][a] * xtx_inv[a][b] * x_mat[i][b];
                }
            }
            h
        })
        .collect();

    // ── Cook's distance ──
    // D_i = (e_i² / (p * MSE)) * (h_ii / (1 - h_ii)²)
    let cooks_distance: Vec<f64> = (0..n)
        .map(|i| {
            let h = leverage[i];
            if (1.0 - h).abs() < 1e-14 {
                f64::INFINITY
            } else {
                (residuals[i].powi(2) / (p as f64 * mse)) * (h / (1.0 - h).powi(2))
            }
        })
        .collect();

    // ── Standardized residuals ──
    let std_residuals: Vec<f64> = (0..n)
        .map(|i| {
            let denom = (mse * (1.0 - leverage[i])).sqrt();
            if denom > 1e-14 {
                residuals[i] / denom
            } else {
                0.0
            }
        })
        .collect();

    // ── Durbin-Watson statistic ──
    let dw = durbin_watson(&residuals);

    // ── VIF (Variance Inflation Factor) ──
    let vif = compute_vif(df, predictors)?;

    // ── Residual statistics ──
    let resid_mean: f64 = residuals.iter().sum::<f64>() / n as f64;
    let resid_std: f64 = (residuals
        .iter()
        .map(|e| (e - resid_mean).powi(2))
        .sum::<f64>()
        / (n as f64 - 1.0))
        .sqrt();
    let resid_min = residuals.iter().copied().fold(f64::INFINITY, f64::min);
    let resid_max = residuals.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    // ── Shapiro-Wilk approximation: skewness & kurtosis of residuals ──
    let skewness = if resid_std > 0.0 {
        let m3: f64 = residuals
            .iter()
            .map(|e| ((e - resid_mean) / resid_std).powi(3))
            .sum::<f64>()
            / n as f64;
        m3
    } else {
        0.0
    };
    let kurtosis = if resid_std > 0.0 {
        let m4: f64 = residuals
            .iter()
            .map(|e| ((e - resid_mean) / resid_std).powi(4))
            .sum::<f64>()
            / n as f64;
        m4 - 3.0 // excess kurtosis
    } else {
        0.0
    };

    // ── Q-Q data (theoretical vs actual quantiles for standardized residuals) ──
    let qq_data = compute_qq_data(&std_residuals);

    // ── Build output ──
    let mut values = BTreeMap::new();
    values.insert("target".to_string(), serde_json::json!(target));
    values.insert("predictors".to_string(), serde_json::to_value(predictors)?);
    values.insert("n".to_string(), serde_json::json!(n));
    values.insert("durbin_watson".to_string(), serde_json::json!(dw));

    // VIF per predictor
    let vif_entries: Vec<serde_json::Value> = predictors
        .iter()
        .zip(vif.iter())
        .map(|(name, &v)| serde_json::json!({"name": name, "vif": v}))
        .collect();
    values.insert("vif".to_string(), serde_json::to_value(&vif_entries)?);

    // Residual stats
    values.insert(
        "residual_stats".to_string(),
        serde_json::json!({
            "mean": resid_mean,
            "std": resid_std,
            "min": resid_min,
            "max": resid_max,
            "skewness": skewness,
            "kurtosis": kurtosis,
        }),
    );

    // Leverage summary
    let lev_mean = leverage.iter().sum::<f64>() / n as f64;
    let lev_max = leverage.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let high_leverage_count = leverage
        .iter()
        .filter(|&&h| h > 2.0 * p as f64 / n as f64)
        .count();
    values.insert(
        "leverage".to_string(),
        serde_json::json!({
            "mean": lev_mean,
            "max": lev_max,
            "threshold": 2.0 * p as f64 / n as f64,
            "high_leverage_count": high_leverage_count,
        }),
    );

    // Cook's distance summary
    let cooks_max = cooks_distance
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0_f64, f64::max);
    let influential_count = cooks_distance
        .iter()
        .filter(|&&d| d.is_finite() && d > 4.0 / n as f64)
        .count();
    values.insert(
        "cooks_distance".to_string(),
        serde_json::json!({
            "max": cooks_max,
            "threshold": 4.0 / n as f64,
            "influential_count": influential_count,
        }),
    );

    // Q-Q data
    values.insert("qq_data".to_string(), serde_json::to_value(&qq_data)?);

    Ok(StatsArtifact {
        kind: StatsKind::RegressionDiagnostics,
        values,
    })
}

/// Compute Durbin-Watson statistic from residuals.
fn durbin_watson(residuals: &[f64]) -> f64 {
    if residuals.len() < 2 {
        return 0.0;
    }
    let num: f64 = residuals.windows(2).map(|w| (w[1] - w[0]).powi(2)).sum();
    let den: f64 = residuals.iter().map(|e| e * e).sum();
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

/// Compute VIF for each predictor by regressing each on the others.
fn compute_vif(df: &DataFrame, predictors: &[String]) -> SfResult<Vec<f64>> {
    if predictors.len() == 1 {
        return Ok(vec![1.0]);
    }

    let k = predictors.len();
    let mut vifs = Vec::with_capacity(k);

    for j in 0..k {
        let target_col = &predictors[j];
        let other_cols: Vec<String> = predictors
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != j)
            .map(|(_, s)| s.clone())
            .collect();

        let y = extract_f64_column(df, target_col)?;
        let n = y.len();
        let p = other_cols.len() + 1;

        // Build X with intercept + other predictors
        let mut x_mat = vec![vec![0.0; p]; n];
        for row in &mut x_mat {
            row[0] = 1.0;
        }
        for (col_j, col_name) in other_cols.iter().enumerate() {
            let vals = extract_f64_column(df, col_name)?;
            for i in 0..n {
                x_mat[i][col_j + 1] = vals[i];
            }
        }

        // Compute R² for this auxiliary regression
        let xtx = mat_mul_ata(&x_mat, n, p);
        let xtx_inv = match mat_invert(&xtx, p) {
            Ok(inv) => inv,
            Err(_) => {
                vifs.push(f64::INFINITY);
                continue;
            }
        };

        let xty: Vec<f64> = (0..p)
            .map(|i| (0..n).map(|k| x_mat[k][i] * y[k]).sum())
            .collect();

        let beta: Vec<f64> = (0..p)
            .map(|i| (0..p).map(|k| xtx_inv[i][k] * xty[k]).sum())
            .collect();

        let y_hat: Vec<f64> = (0..n)
            .map(|i| (0..p).map(|k| x_mat[i][k] * beta[k]).sum())
            .collect();

        let y_mean: f64 = y.iter().sum::<f64>() / n as f64;
        let sse: f64 = y
            .iter()
            .zip(y_hat.iter())
            .map(|(yi, yhi)| (yi - yhi).powi(2))
            .sum();
        let sst: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();

        let r2 = if sst > 0.0 { 1.0 - sse / sst } else { 0.0 };
        let vif = if (1.0 - r2).abs() < 1e-14 {
            f64::INFINITY
        } else {
            1.0 / (1.0 - r2)
        };
        vifs.push(vif);
    }

    Ok(vifs)
}

/// A'A for n×p matrix A, returns p×p matrix.
fn mat_mul_ata(a: &[Vec<f64>], n: usize, p: usize) -> Vec<Vec<f64>> {
    let mut result = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in i..p {
            let mut sum = 0.0;
            for row in a.iter().take(n) {
                sum += row[i] * row[j];
            }
            result[i][j] = sum;
            result[j][i] = sum;
        }
    }
    result
}

/// Compute Q-Q data: sorted standardized residuals vs theoretical normal quantiles.
fn compute_qq_data(std_residuals: &[f64]) -> Vec<serde_json::Value> {
    let n = std_residuals.len();
    let mut sorted = std_residuals.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    sorted
        .iter()
        .enumerate()
        .map(|(i, &actual)| {
            let p = (i as f64 + 0.5) / n as f64;
            let theoretical = normal_quantile(p);
            serde_json::json!({
                "theoretical": theoretical,
                "actual": actual,
            })
        })
        .collect()
}

/// Approximate the standard normal quantile using the rational approximation
/// from Peter Acklam.
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    // Coefficients for rational approximation
    let a = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383_577_518_672_69e2,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    let b = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    let c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    let d = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_df() -> DataFrame {
        df! {
            "x1" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "x2" => &[2.0_f64, 1.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0, 10.0, 9.0],
            "y" => &[9.1_f64, 8.2, 19.8, 18.5, 29.2, 27.9, 39.1, 38.2, 49.8, 48.3],
        }
        .unwrap()
    }

    #[test]
    fn diagnostics_returns_durbin_watson() {
        let df = test_df();
        let result = compute_regression_diagnostics(&df, "y", &["x1".into(), "x2".into()]).unwrap();

        let dw = result
            .values
            .get("durbin_watson")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!(dw > 0.0 && dw < 4.0, "DW={}", dw);
    }

    #[test]
    fn diagnostics_returns_vif_for_each_predictor() {
        let df = test_df();
        let result = compute_regression_diagnostics(&df, "y", &["x1".into(), "x2".into()]).unwrap();

        let vif = result.values.get("vif").unwrap().as_array().unwrap();
        assert_eq!(vif.len(), 2);
        for entry in vif {
            let v = entry["vif"].as_f64().unwrap();
            assert!(v >= 1.0, "VIF must be >= 1, got {}", v);
        }
    }

    #[test]
    fn diagnostics_returns_leverage_info() {
        let df = test_df();
        let result = compute_regression_diagnostics(&df, "y", &["x1".into(), "x2".into()]).unwrap();

        let lev = result.values.get("leverage").unwrap();
        assert!(lev.get("mean").is_some());
        assert!(lev.get("max").is_some());
        assert!(lev.get("threshold").is_some());
        assert!(lev.get("high_leverage_count").is_some());
    }

    #[test]
    fn diagnostics_returns_cooks_distance() {
        let df = test_df();
        let result = compute_regression_diagnostics(&df, "y", &["x1".into(), "x2".into()]).unwrap();

        let cooks = result.values.get("cooks_distance").unwrap();
        assert!(cooks.get("max").is_some());
        assert!(cooks.get("threshold").is_some());
        assert!(cooks.get("influential_count").is_some());
    }

    #[test]
    fn diagnostics_returns_residual_stats() {
        let df = test_df();
        let result = compute_regression_diagnostics(&df, "y", &["x1".into(), "x2".into()]).unwrap();

        let stats = result.values.get("residual_stats").unwrap();
        assert!(stats.get("mean").is_some());
        assert!(stats.get("std").is_some());
        assert!(stats.get("min").is_some());
        assert!(stats.get("max").is_some());
        assert!(stats.get("skewness").is_some());
        assert!(stats.get("kurtosis").is_some());
    }

    #[test]
    fn diagnostics_returns_qq_data() {
        let df = test_df();
        let result = compute_regression_diagnostics(&df, "y", &["x1".into(), "x2".into()]).unwrap();

        let qq = result.values.get("qq_data").unwrap().as_array().unwrap();
        assert_eq!(qq.len(), 10); // same as n
        for pt in qq {
            assert!(pt.get("theoretical").is_some());
            assert!(pt.get("actual").is_some());
        }
    }

    #[test]
    fn single_predictor_vif_is_one() {
        let df = df! {
            "x" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
            "y" => &[5.0_f64, 8.0, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0, 29.0, 32.0],
        }
        .unwrap();
        let result = compute_regression_diagnostics(&df, "y", &["x".into()]).unwrap();

        let vif = result.values.get("vif").unwrap().as_array().unwrap();
        assert_eq!(vif.len(), 1);
        assert!((vif[0]["vif"].as_f64().unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn durbin_watson_near_two_for_random_residuals() {
        // DW ≈ 2 means no autocorrelation. For a perfect fit with no noise,
        // residuals are all zero, so DW is 0/0 → handled as 0.
        let residuals = vec![0.5, -0.3, 0.1, -0.4, 0.2, -0.1, 0.3, -0.2, 0.4, -0.5];
        let dw = durbin_watson(&residuals);
        assert!(dw > 1.0 && dw < 3.0, "DW={}", dw);
    }

    #[test]
    fn diagnostics_rejects_empty_predictors() {
        let df = df! { "y" => &[1.0_f64, 2.0] }.unwrap();
        let result = compute_regression_diagnostics(&df, "y", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn normal_quantile_returns_zero_at_half() {
        let q = normal_quantile(0.5);
        assert!(q.abs() < 1e-10, "normal_quantile(0.5) = {}", q);
    }

    #[test]
    fn normal_quantile_is_monotonic() {
        let probs = [0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 0.99];
        let quantiles: Vec<f64> = probs.iter().map(|&p| normal_quantile(p)).collect();
        for i in 1..quantiles.len() {
            assert!(
                quantiles[i] > quantiles[i - 1],
                "quantiles not monotonic at index {}: {} vs {}",
                i,
                quantiles[i - 1],
                quantiles[i]
            );
        }
    }
}
