use polars::prelude::*;
use sf_core::error::SfResult;
use sf_core::recipe::CorrMethod;
use sf_core::types::{StatsArtifact, StatsKind};
use sf_core::SfError;
use std::collections::BTreeMap;

/// Compute a correlation matrix for the given columns.
pub fn compute_correlation(
    df: &DataFrame,
    cols: &[String],
    method: &CorrMethod,
) -> SfResult<StatsArtifact> {
    if cols.len() < 2 {
        return Err(SfError::Validation(
            "Correlation requires at least 2 columns".into(),
        ));
    }

    let mut col_data: Vec<(String, Vec<f64>)> = Vec::new();
    for col_name in cols {
        let col = df
            .column(col_name)
            .map_err(|_| SfError::Schema(format!("Column not found: '{}'", col_name)))?;
        let series = col.as_materialized_series();
        let casted = series.cast(&DataType::Float64)?;
        let values: Vec<f64> = casted.f64()?.into_no_null_iter().collect();
        col_data.push((col_name.clone(), values));
    }

    let n = col_data.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        matrix[i][i] = 1.0;
        for j in (i + 1)..n {
            let r = match method {
                CorrMethod::Pearson => pearson(&col_data[i].1, &col_data[j].1),
                CorrMethod::Spearman => spearman(&col_data[i].1, &col_data[j].1),
            };
            matrix[i][j] = r;
            matrix[j][i] = r;
        }
    }

    let method_str = match method {
        CorrMethod::Pearson => "pearson",
        CorrMethod::Spearman => "spearman",
    };

    let mut values = BTreeMap::new();
    values.insert(
        "method".to_string(),
        serde_json::Value::String(method_str.to_string()),
    );
    values.insert("columns".to_string(), serde_json::to_value(cols)?);
    values.insert("matrix".to_string(), serde_json::to_value(&matrix)?);

    Ok(StatsArtifact {
        kind: StatsKind::Correlation,
        values,
    })
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }
    cov / (var_x.sqrt() * var_y.sqrt())
}

fn spearman(x: &[f64], y: &[f64]) -> f64 {
    let rank_x = rank(x);
    let rank_y = rank(y);
    pearson(&rank_x, &rank_y)
}

fn rank(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && indexed[j].1 == indexed[i].1 {
            j += 1;
        }
        let avg_rank = (i + j + 1) as f64 / 2.0;
        for k in i..j {
            ranks[indexed[k].0] = avg_rank;
        }
        i = j;
    }
    ranks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pearson_perfect() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = pearson(&x, &y);
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_step() {
        let df = polars::prelude::df! {
            "x" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0],
            "y" => &[2.0_f64, 4.0, 6.0, 8.0, 10.0],
        }
        .unwrap();

        let result =
            compute_correlation(&df, &["x".into(), "y".into()], &CorrMethod::Pearson).unwrap();

        let matrix = result.values.get("matrix").unwrap().as_array().unwrap();
        let r = matrix[0].as_array().unwrap()[1].as_f64().unwrap();
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_too_few_cols() {
        let df = polars::prelude::df! { "x" => &[1.0_f64] }.unwrap();
        let result = compute_correlation(&df, &["x".into()], &CorrMethod::Pearson);
        assert!(result.is_err());
    }
}
