//! Thin wrappers around statrs distribution functions for p-values and quantiles.

use statrs::distribution::{ChiSquared, ContinuousCDF, FisherSnedecor, Normal, StudentsT};

/// CDF of the t-distribution: P(T <= t) for given degrees of freedom.
pub fn t_cdf(t: f64, df: f64) -> f64 {
    let dist = StudentsT::new(0.0, 1.0, df).expect("valid t-distribution params");
    dist.cdf(t)
}

/// Two-tailed p-value from t-distribution.
pub fn t_two_tailed_p(t_stat: f64, df: f64) -> f64 {
    let p = 2.0 * (1.0 - t_cdf(t_stat.abs(), df));
    p.clamp(0.0, 1.0)
}

/// Quantile (inverse CDF) of the t-distribution.
pub fn t_quantile(p: f64, df: f64) -> f64 {
    let dist = StudentsT::new(0.0, 1.0, df).expect("valid t-distribution params");
    dist.inverse_cdf(p)
}

/// CDF of the chi-squared distribution: P(X <= x) for given degrees of freedom.
pub fn chi2_cdf(x: f64, df: f64) -> f64 {
    let dist = ChiSquared::new(df).expect("valid chi-squared params");
    dist.cdf(x)
}

/// Upper-tail p-value from chi-squared distribution: P(X >= x).
pub fn chi2_upper_p(x: f64, df: f64) -> f64 {
    let p = 1.0 - chi2_cdf(x, df);
    p.clamp(0.0, 1.0)
}

/// Two-tailed p-value from chi-squared distribution (for variance tests).
pub fn chi2_two_tailed_p(x: f64, df: f64) -> f64 {
    let lower = chi2_cdf(x, df);
    let upper = 1.0 - lower;
    let p = 2.0 * lower.min(upper);
    p.clamp(0.0, 1.0)
}

/// Quantile (inverse CDF) of the chi-squared distribution.
pub fn chi2_quantile(p: f64, df: f64) -> f64 {
    let dist = ChiSquared::new(df).expect("valid chi-squared params");
    dist.inverse_cdf(p)
}

/// CDF of the F-distribution.
pub fn f_cdf(x: f64, df1: f64, df2: f64) -> f64 {
    let dist = FisherSnedecor::new(df1, df2).expect("valid F-distribution params");
    dist.cdf(x)
}

/// Upper-tail p-value from F-distribution.
pub fn f_upper_p(x: f64, df1: f64, df2: f64) -> f64 {
    let p = 1.0 - f_cdf(x, df1, df2);
    p.clamp(0.0, 1.0)
}

/// CDF of the standard normal distribution.
pub fn normal_cdf(z: f64) -> f64 {
    let dist = Normal::new(0.0, 1.0).expect("valid normal params");
    dist.cdf(z)
}

/// Two-tailed p-value from standard normal distribution.
pub fn normal_two_tailed_p(z: f64) -> f64 {
    let p = 2.0 * (1.0 - normal_cdf(z.abs()));
    p.clamp(0.0, 1.0)
}

/// Quantile (inverse CDF) of the standard normal distribution.
pub fn normal_quantile(p: f64) -> f64 {
    let dist = Normal::new(0.0, 1.0).expect("valid normal params");
    dist.inverse_cdf(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-6;

    #[test]
    fn t_cdf_returns_half_at_zero() {
        let p = t_cdf(0.0, 10.0);
        assert!((p - 0.5).abs() < TOL);
    }

    #[test]
    fn t_two_tailed_p_returns_one_at_zero() {
        let p = t_two_tailed_p(0.0, 10.0);
        assert!((p - 1.0).abs() < TOL);
    }

    #[test]
    fn t_two_tailed_p_returns_small_for_large_t() {
        let p = t_two_tailed_p(10.0, 30.0);
        assert!(p < 0.001);
    }

    #[test]
    fn t_quantile_returns_zero_at_half() {
        let q = t_quantile(0.5, 10.0);
        assert!(q.abs() < TOL);
    }

    #[test]
    fn t_quantile_is_positive_above_half() {
        let q = t_quantile(0.975, 10.0);
        assert!(q > 2.0);
    }

    #[test]
    fn chi2_cdf_at_df_is_roughly_half() {
        // For chi-squared, CDF at x=df is approximately 0.5 for large df
        let p = chi2_cdf(30.0, 30.0);
        assert!((p - 0.5).abs() < 0.1);
    }

    #[test]
    fn chi2_upper_p_at_zero_is_one() {
        let p = chi2_upper_p(0.0, 5.0);
        assert!((p - 1.0).abs() < TOL);
    }

    #[test]
    fn chi2_quantile_roundtrips() {
        let df = 10.0;
        let p = 0.95;
        let q = chi2_quantile(p, df);
        let p2 = chi2_cdf(q, df);
        assert!((p - p2).abs() < TOL);
    }

    #[test]
    fn normal_cdf_at_zero_is_half() {
        let p = normal_cdf(0.0);
        assert!((p - 0.5).abs() < TOL);
    }

    #[test]
    fn normal_two_tailed_p_at_196_is_about_005() {
        let p = normal_two_tailed_p(1.96);
        assert!((p - 0.05).abs() < 0.001);
    }

    #[test]
    fn normal_quantile_at_0975_is_about_196() {
        let q = normal_quantile(0.975);
        assert!((q - 1.96).abs() < 0.01);
    }

    #[test]
    fn f_upper_p_at_zero_is_one() {
        let p = f_upper_p(0.0, 5.0, 10.0);
        assert!((p - 1.0).abs() < TOL);
    }

    #[test]
    fn f_upper_p_decreases_with_larger_x() {
        let p1 = f_upper_p(1.0, 5.0, 10.0);
        let p2 = f_upper_p(5.0, 5.0, 10.0);
        assert!(p2 < p1);
    }
}
