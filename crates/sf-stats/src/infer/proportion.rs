//! Proportion hypothesis tests.

use sf_core::{SfError, SfResult, StatsArtifact, StatsKind};
use std::collections::BTreeMap;

use super::distributions::normal_two_tailed_p;
use super::ttest::Alternative;

/// One-sample proportion test (z-test for proportions).
///
/// Tests whether the population proportion equals `p0`.
pub fn compute_proportion_test(
    n: u64,
    successes: u64,
    p0: f64,
    confidence: f64,
    alternative: Alternative,
) -> SfResult<StatsArtifact> {
    if n == 0 {
        return Err(SfError::Validation("Sample size must be > 0".into()));
    }
    if successes > n {
        return Err(SfError::Validation(
            "Number of successes cannot exceed sample size".into(),
        ));
    }
    if !(0.0..=1.0).contains(&p0) {
        return Err(SfError::Validation(
            "Null hypothesis proportion p0 must be between 0 and 1".into(),
        ));
    }

    let n_f = n as f64;
    let p_hat = successes as f64 / n_f;
    let se = (p0 * (1.0 - p0) / n_f).sqrt();

    if se == 0.0 {
        return Err(SfError::Validation(
            "Cannot test proportion when p0 is 0 or 1".into(),
        ));
    }

    let z_stat = (p_hat - p0) / se;

    let p_value = match alternative {
        Alternative::TwoSided => normal_two_tailed_p(z_stat),
        Alternative::Less => super::distributions::normal_cdf(z_stat),
        Alternative::Greater => 1.0 - super::distributions::normal_cdf(z_stat),
    };

    // CI using Wald method (based on p_hat, not p0)
    let alpha = 1.0 - confidence;
    let z_crit = super::distributions::normal_quantile(1.0 - alpha / 2.0);
    let se_ci = (p_hat * (1.0 - p_hat) / n_f).sqrt();
    let ci_lower = (p_hat - z_crit * se_ci).max(0.0);
    let ci_upper = (p_hat + z_crit * se_ci).min(1.0);

    // Effect size: Cohen's h
    let cohens_h = 2.0 * (p_hat.sqrt().asin() - p0.sqrt().asin());

    let mut values_map = BTreeMap::new();
    values_map.insert("test".into(), serde_json::json!("proportion_z_test"));
    values_map.insert("n".into(), serde_json::json!(n));
    values_map.insert("successes".into(), serde_json::json!(successes));
    values_map.insert("p_hat".into(), serde_json::json!(p_hat));
    values_map.insert("p0".into(), serde_json::json!(p0));
    values_map.insert("z_statistic".into(), serde_json::json!(z_stat));
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
    values_map.insert("effect_size".into(), serde_json::json!(cohens_h));
    values_map.insert("effect_size_type".into(), serde_json::json!("cohens_h"));
    values_map.insert("se".into(), serde_json::json!(se));

    Ok(StatsArtifact {
        kind: StatsKind::ProportionTest,
        values: values_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proportion_test_returns_correct_structure() {
        let result = compute_proportion_test(100, 60, 0.5, 0.95, Alternative::TwoSided).unwrap();
        assert!(matches!(result.kind, StatsKind::ProportionTest));
        assert_eq!(result.values["test"].as_str(), Some("proportion_z_test"));
        assert!(result.values.contains_key("z_statistic"));
        assert!(result.values.contains_key("p_value"));
        assert!(result.values.contains_key("ci_lower"));
        assert!(result.values.contains_key("ci_upper"));
        assert!(result.values.contains_key("effect_size"));
    }

    #[test]
    fn proportion_test_p_hat_equals_p0_gives_high_p() {
        // 50 out of 100 when testing p0=0.5
        let result = compute_proportion_test(100, 50, 0.5, 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(
            p > 0.9,
            "p-value should be very high when p_hat == p0, got {}",
            p
        );
    }

    #[test]
    fn proportion_test_detects_departure_from_half() {
        // 80 out of 100 when testing p0=0.5
        let result = compute_proportion_test(100, 80, 0.5, 0.95, Alternative::TwoSided).unwrap();
        let p = result.values["p_value"].as_f64().unwrap();
        assert!(p < 0.001, "p-value should be very small, got {}", p);
    }

    #[test]
    fn proportion_test_rejects_invalid_p0() {
        let result = compute_proportion_test(100, 50, 1.5, 0.95, Alternative::TwoSided);
        assert!(result.is_err());
    }

    #[test]
    fn proportion_test_rejects_successes_exceeding_n() {
        let result = compute_proportion_test(100, 101, 0.5, 0.95, Alternative::TwoSided);
        assert!(result.is_err());
    }

    #[test]
    fn proportion_test_effect_size_is_cohens_h() {
        let result = compute_proportion_test(100, 60, 0.5, 0.95, Alternative::TwoSided).unwrap();
        assert_eq!(result.values["effect_size_type"].as_str(), Some("cohens_h"));
    }
}
