use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::types::{ForecastArtifact, ForecastPoint};
use sf_core::SfError;

/// Minimal ARIMA(p,d,q) forecast implementation.
///
/// - p: AR order (autoregressive)
/// - d: differencing order
/// - q: MA order (moving average) — simplified via residual approximation
///
/// Returns a ForecastArtifact with predicted points and confidence intervals.
pub fn compute_arima_forecast(
    df: &DataFrame,
    time_col: &str,
    value_col: &str,
    order: [usize; 3],
    horizon: usize,
    confidence: f64,
) -> SfResult<ForecastArtifact> {
    let _ = df
        .column(time_col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", time_col)))?;
    let value_series = df
        .column(value_col)
        .map_err(|_| SfError::Schema(format!("Column not found: '{}'", value_col)))?;

    let values = series_to_f64_vec(value_series.as_materialized_series())?;
    let [p, d, q] = order;

    if values.len() < p + d + 2 {
        return Err(SfError::Validation(format!(
            "ARIMA({},{},{}) requires at least {} observations, got {}",
            p,
            d,
            q,
            p + d + 2,
            values.len()
        )));
    }

    // Step 1: Apply differencing
    let diffed = difference(&values, d);

    if diffed.len() < p + 2 {
        return Err(SfError::Validation(
            "Not enough observations after differencing".into(),
        ));
    }

    // Step 2: Fit AR(p) model on differenced series
    let (ar_coeffs, intercept, sigma) = fit_ar(&diffed, p)?;

    // Step 3: Compute residuals for MA component
    let residuals = compute_residuals(&diffed, &ar_coeffs, intercept, p);

    // Step 4: Fit MA(q) component (simplified: use residual autocorrelation)
    let ma_coeffs = if q > 0 {
        fit_ma_approx(&residuals, q)
    } else {
        vec![]
    };

    // Step 5: Forecast on differenced scale
    let mut forecast_diff = Vec::with_capacity(horizon);
    let mut recent_values: Vec<f64> = diffed[diffed.len().saturating_sub(p)..].to_vec();
    let mut recent_residuals: Vec<f64> = residuals[residuals.len().saturating_sub(q)..].to_vec();

    for _ in 0..horizon {
        let mut pred = intercept;

        // AR component
        for (j, coeff) in ar_coeffs.iter().enumerate() {
            if j < recent_values.len() {
                pred += coeff * recent_values[recent_values.len() - 1 - j];
            }
        }

        // MA component
        for (j, coeff) in ma_coeffs.iter().enumerate() {
            if j < recent_residuals.len() {
                pred += coeff * recent_residuals[recent_residuals.len() - 1 - j];
            }
        }

        forecast_diff.push(pred);
        recent_values.push(pred);
        recent_residuals.push(0.0); // Future residuals assumed zero
    }

    // Step 6: Reverse differencing to get forecasts on original scale
    let forecasts = undifference(&values, &forecast_diff, d);

    // Step 7: Compute prediction intervals
    let z = z_critical(confidence);
    let mut cumulative_var = vec![0.0; horizon];
    cumulative_var[0] = sigma * sigma;
    // MA coefficients affect prediction interval width
    let psi = compute_psi_weights(&ar_coeffs, &ma_coeffs, horizon);
    for h in 0..horizon {
        let mut var_h = 0.0;
        for j in 0..=h {
            let psi_j = if j == 0 { 1.0 } else if j <= psi.len() { psi[j - 1] } else { 0.0 };
            var_h += psi_j * psi_j;
        }
        cumulative_var[h] = sigma * sigma * var_h;
    }

    // Build time labels for forecast points
    let points: Vec<ForecastPoint> = forecasts
        .iter()
        .enumerate()
        .map(|(i, &yhat)| {
            let se = cumulative_var[i].sqrt();
            ForecastPoint {
                index: format!("T+{}", i + 1),
                yhat,
                lower: Some(yhat - z * se),
                upper: Some(yhat + z * se),
            }
        })
        .collect();

    Ok(ForecastArtifact {
        model: format!("ARIMA({},{},{})", p, d, q),
        points,
    })
}

/// Apply d-order differencing.
fn difference(values: &[f64], d: usize) -> Vec<f64> {
    let mut current = values.to_vec();
    for _ in 0..d {
        let next: Vec<f64> = current.windows(2).map(|w| w[1] - w[0]).collect();
        current = next;
    }
    current
}

/// Reverse differencing to recover original scale.
fn undifference(original: &[f64], forecast_diff: &[f64], d: usize) -> Vec<f64> {
    if d == 0 {
        return forecast_diff.to_vec();
    }

    // For d=1: forecast[i] = last_original + cumsum(forecast_diff[0..=i])
    // For d>1: apply recursively
    let mut result = forecast_diff.to_vec();

    for diff_level in (0..d).rev() {
        let base_values = if diff_level == 0 {
            original.to_vec()
        } else {
            difference(original, diff_level)
        };

        let last = *base_values.last().unwrap_or(&0.0);
        let mut cumsum = last;
        for val in &mut result {
            cumsum += *val;
            *val = cumsum;
        }
    }

    result
}

/// Fit AR(p) model using Yule-Walker equations.
fn fit_ar(y: &[f64], p: usize) -> SfResult<(Vec<f64>, f64, f64)> {
    if p == 0 {
        let mean = y.iter().sum::<f64>() / y.len() as f64;
        let var = y.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (y.len() - 1) as f64;
        return Ok((vec![], mean, var.sqrt()));
    }

    let n = y.len();
    let mean = y.iter().sum::<f64>() / n as f64;
    let centered: Vec<f64> = y.iter().map(|v| v - mean).collect();

    // Compute autocorrelations
    let mut gamma = vec![0.0; p + 1];
    for k in 0..=p {
        let mut sum = 0.0;
        for i in k..n {
            sum += centered[i] * centered[i - k];
        }
        gamma[k] = sum / n as f64;
    }

    if gamma[0].abs() < 1e-15 {
        return Ok((vec![0.0; p], mean, 0.0));
    }

    // Solve Yule-Walker: R * φ = r
    let mut r_matrix = vec![vec![0.0; p]; p];
    let mut r_vec = vec![0.0; p];

    for i in 0..p {
        r_vec[i] = gamma[i + 1];
        for j in 0..p {
            let idx = i.abs_diff(j);
            r_matrix[i][j] = gamma[idx];
        }
    }

    let coeffs = solve_linear_system(&r_matrix, &r_vec)?;

    // Estimate residual standard deviation
    let mut sigma2 = gamma[0];
    for (i, c) in coeffs.iter().enumerate() {
        sigma2 -= c * gamma[i + 1];
    }
    let sigma = if sigma2 > 0.0 { sigma2.sqrt() } else { 0.0 };

    // Intercept: μ * (1 - Σφ_i)
    let phi_sum: f64 = coeffs.iter().sum();
    let intercept = mean * (1.0 - phi_sum);

    Ok((coeffs, intercept, sigma))
}

/// Solve linear system Ax = b using Gaussian elimination.
fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> SfResult<Vec<f64>> {
    let n = a.len();
    let mut aug: Vec<Vec<f64>> = a
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut r = row.clone();
            r.push(b[i]);
            r
        })
        .collect();

    // Forward elimination with partial pivoting
    for col in 0..n {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return Err(SfError::Execution(
                "Singular matrix in AR parameter estimation".into(),
            ));
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        for j in col..=n {
            aug[col][j] /= pivot;
        }

        for row in 0..n {
            if row != col {
                let factor = aug[row][col];
                for j in col..=n {
                    aug[row][j] -= factor * aug[col][j];
                }
            }
        }
    }

    Ok(aug.iter().map(|row| row[n]).collect())
}

/// Compute residuals from AR model.
fn compute_residuals(y: &[f64], ar_coeffs: &[f64], intercept: f64, p: usize) -> Vec<f64> {
    let mut residuals = vec![0.0; y.len()];
    for t in p..y.len() {
        let mut pred = intercept;
        for (j, coeff) in ar_coeffs.iter().enumerate() {
            pred += coeff * y[t - 1 - j];
        }
        residuals[t] = y[t] - pred;
    }
    residuals
}

/// Approximate MA coefficients from residual autocorrelation.
fn fit_ma_approx(residuals: &[f64], q: usize) -> Vec<f64> {
    let n = residuals.len();
    if n < q + 1 {
        return vec![0.0; q];
    }

    let mean = residuals.iter().sum::<f64>() / n as f64;
    let centered: Vec<f64> = residuals.iter().map(|r| r - mean).collect();

    let var: f64 = centered.iter().map(|v| v * v).sum::<f64>() / n as f64;
    if var.abs() < 1e-15 {
        return vec![0.0; q];
    }

    let mut ma_coeffs = vec![0.0; q];
    for k in 1..=q {
        let mut sum = 0.0;
        for i in k..n {
            sum += centered[i] * centered[i - k];
        }
        let acf_k = sum / (n as f64 * var);
        // Approximate MA coefficient from autocorrelation
        ma_coeffs[k - 1] = acf_k.clamp(-0.99, 0.99);
    }

    ma_coeffs
}

/// Compute psi weights (impulse response) for prediction intervals.
fn compute_psi_weights(ar: &[f64], ma: &[f64], n: usize) -> Vec<f64> {
    let mut psi = vec![0.0; n];
    for i in 0..n {
        let mut val = if i < ma.len() { ma[i] } else { 0.0 };
        for (j, ar_j) in ar.iter().enumerate() {
            if i > j {
                val += ar_j * psi[i - j - 1];
            } else if i == j {
                val += ar_j;
            }
        }
        psi[i] = val;
    }
    psi
}

/// Z critical value for a given confidence level.
fn z_critical(confidence: f64) -> f64 {
    // Common critical values
    let alpha = 1.0 - confidence;
    if (alpha - 0.01).abs() < 0.001 {
        2.576
    } else if (alpha - 0.05).abs() < 0.001 {
        1.960
    } else if (alpha - 0.10).abs() < 0.001 {
        1.645
    } else {
        // Approximation using rational approximation of inverse normal
        rational_approx_inv_normal(1.0 - alpha / 2.0)
    }
}

/// Rational approximation for inverse standard normal (Abramowitz & Stegun).
fn rational_approx_inv_normal(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        return 1.96; // fallback
    }
    let t = (-2.0 * (1.0 - p).ln()).sqrt();
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;
    t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t)
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
    fn arima_forecast_returns_correct_horizon() {
        let values: Vec<f64> = (0..50).map(|i| 10.0 + 0.5 * i as f64).collect();
        let df = make_ts_df(&values);

        let result = compute_arima_forecast(&df, "time", "value", [1, 1, 0], 5, 0.95).unwrap();
        assert_eq!(result.points.len(), 5);
        assert_eq!(result.model, "ARIMA(1,1,0)");
    }

    #[test]
    fn arima_forecast_has_confidence_intervals() {
        let values: Vec<f64> = (0..50).map(|i| 10.0 + 0.5 * i as f64).collect();
        let df = make_ts_df(&values);

        let result = compute_arima_forecast(&df, "time", "value", [1, 1, 0], 5, 0.95).unwrap();

        for pt in &result.points {
            assert!(pt.lower.is_some());
            assert!(pt.upper.is_some());
            let lower = pt.lower.unwrap();
            let upper = pt.upper.unwrap();
            assert!(lower <= pt.yhat, "lower {} > yhat {}", lower, pt.yhat);
            assert!(upper >= pt.yhat, "upper {} < yhat {}", upper, pt.yhat);
        }
    }

    #[test]
    fn arima_forecast_intervals_widen_with_horizon() {
        let values: Vec<f64> = (0..50).map(|i| 10.0 + 0.5 * i as f64 + (i as f64 * 0.3).sin()).collect();
        let df = make_ts_df(&values);

        let result = compute_arima_forecast(&df, "time", "value", [1, 1, 0], 10, 0.95).unwrap();

        let first_width = result.points[0].upper.unwrap() - result.points[0].lower.unwrap();
        let last_width = result.points[9].upper.unwrap() - result.points[9].lower.unwrap();
        assert!(
            last_width >= first_width,
            "Intervals should widen: first={}, last={}",
            first_width,
            last_width
        );
    }

    #[test]
    fn arima_forecast_linear_trend_continues() {
        // Linear trend: y = 10 + 0.5*t
        let values: Vec<f64> = (0..50).map(|i| 10.0 + 0.5 * i as f64).collect();
        let df = make_ts_df(&values);

        let result = compute_arima_forecast(&df, "time", "value", [0, 1, 0], 5, 0.95).unwrap();

        // Forecast should roughly continue the trend
        let last_value = values.last().unwrap();
        assert!(
            result.points[0].yhat > *last_value,
            "First forecast {} should exceed last value {}",
            result.points[0].yhat,
            last_value
        );
    }

    #[test]
    fn arima_rejects_too_few_observations() {
        let df = make_ts_df(&[1.0, 2.0]);
        let result = compute_arima_forecast(&df, "time", "value", [1, 1, 0], 5, 0.95);
        assert!(result.is_err());
    }

    #[test]
    fn arima_rejects_missing_column() {
        let df = make_ts_df(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = compute_arima_forecast(&df, "time", "missing", [1, 0, 0], 5, 0.95);
        assert!(result.is_err());
    }

    #[test]
    fn arima_with_ma_component() {
        let values: Vec<f64> = (0..60).map(|i| 10.0 + 0.3 * i as f64 + (i as f64 * 0.5).sin()).collect();
        let df = make_ts_df(&values);

        let result = compute_arima_forecast(&df, "time", "value", [1, 1, 1], 5, 0.95).unwrap();
        assert_eq!(result.model, "ARIMA(1,1,1)");
        assert_eq!(result.points.len(), 5);
    }

    #[test]
    fn difference_and_undifference_roundtrip() {
        let original = vec![1.0, 3.0, 6.0, 10.0, 15.0];
        let diffed = difference(&original, 1);
        assert_eq!(diffed, vec![2.0, 3.0, 4.0, 5.0]);

        // Forecast: continue with 6.0, 7.0
        let forecast_diff = vec![6.0, 7.0];
        let forecasts = undifference(&original, &forecast_diff, 1);
        assert!((forecasts[0] - 21.0).abs() < 1e-10); // 15 + 6
        assert!((forecasts[1] - 28.0).abs() < 1e-10); // 21 + 7
    }

    #[test]
    fn difference_order_2() {
        let original = vec![1.0, 4.0, 9.0, 16.0, 25.0]; // squares
        let d1 = difference(&original, 1);
        assert_eq!(d1, vec![3.0, 5.0, 7.0, 9.0]);

        let d2 = difference(&original, 2);
        assert_eq!(d2, vec![2.0, 2.0, 2.0]);
    }
}
