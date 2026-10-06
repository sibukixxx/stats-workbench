use polars::prelude::*;
use sf_core::{Artifact, ArtifactStore, SfError, SfResult, StatsArtifact, StatsKind};
use std::fs;
use std::path::Path;

/// Write dataset artifacts to Parquet.
///
/// Dataset, Table, and supported Stats artifacts can be written.
pub fn write_parquet(artifacts: &ArtifactStore, keys: &[&str], path: Option<&str>) -> SfResult<()> {
    let path = path.ok_or_else(|| {
        SfError::Validation("Parquet output requires a 'path' in output config".into())
    })?;

    // Find the first dataset artifact
    let mut df = None;
    for key in keys {
        if let Some(artifact) = artifacts.get(key) {
            match artifact {
                Artifact::Dataset(ds) => {
                    df = Some(ds.collect().map_err(|e| {
                        SfError::Internal(format!("Failed to collect dataset: {}", e))
                    })?);
                    break;
                }
                Artifact::Table(table_df) => {
                    df = Some(table_df.clone());
                    break;
                }
                Artifact::Stats(stats) => {
                    df = Some(stats_to_dataframe(stats)?);
                    break;
                }
                _ => continue,
            }
        }
    }

    let mut df = df.ok_or_else(|| {
        SfError::Validation("No parquet-compatible artifact found for Parquet output".into())
    })?;

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    ParquetWriter::new(&mut file)
        .finish(&mut df)
        .map_err(|e| SfError::Io(format!("Parquet write failed: {}", e)))?;

    tracing::info!("Parquet output written to {}", path);
    Ok(())
}

fn stats_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    match stats.kind {
        StatsKind::Summary => summary_to_dataframe(stats),
        StatsKind::Correlation => correlation_to_dataframe(stats),
        StatsKind::Distribution => distribution_to_dataframe(stats),
        StatsKind::OutlierDetection => outlier_to_dataframe(stats),
        StatsKind::TTest => ttest_to_dataframe(stats),
        StatsKind::ConfidenceInterval => ci_to_dataframe(stats),
        StatsKind::ProportionTest => hypothesis_test_to_dataframe(stats),
        StatsKind::ChiSquareTest => hypothesis_test_to_dataframe(stats),
        StatsKind::VarianceTest => hypothesis_test_to_dataframe(stats),
        StatsKind::Regression => regression_to_dataframe(stats),
        StatsKind::RegressionDiagnostics => diagnostics_to_dataframe(stats),
        _ => Err(SfError::Validation(format!(
            "Stats kind '{:?}' cannot be written to Parquet",
            stats.kind
        ))),
    }
}

fn summary_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let columns = stats
        .values
        .get("columns")
        .and_then(|value| value.as_array())
        .ok_or_else(|| SfError::Internal("Summary stats missing 'columns' array".into()))?;

    let names: SfResult<Vec<String>> = columns
        .iter()
        .map(|column| required_str(column, "name").map(ToOwned::to_owned))
        .collect();
    let counts: SfResult<Vec<u64>> = columns
        .iter()
        .map(|column| required_u64(column, "count"))
        .collect();
    let null_counts: SfResult<Vec<u64>> = columns
        .iter()
        .map(|column| required_u64(column, "null_count"))
        .collect();
    let means = columns
        .iter()
        .map(|column| optional_f64(column, "mean"))
        .collect::<Vec<_>>();
    let stds = columns
        .iter()
        .map(|column| optional_f64(column, "std"))
        .collect::<Vec<_>>();
    let mins = columns
        .iter()
        .map(|column| optional_f64(column, "min"))
        .collect::<Vec<_>>();
    let maxs = columns
        .iter()
        .map(|column| optional_f64(column, "max"))
        .collect::<Vec<_>>();
    let q25s = columns
        .iter()
        .map(|column| optional_f64(column, "q25"))
        .collect::<Vec<_>>();
    let q50s = columns
        .iter()
        .map(|column| optional_f64(column, "q50"))
        .collect::<Vec<_>>();
    let q75s = columns
        .iter()
        .map(|column| optional_f64(column, "q75"))
        .collect::<Vec<_>>();

    DataFrame::new(vec![
        Column::new("name".into(), &names?),
        Column::new("count".into(), &counts?),
        Column::new("null_count".into(), &null_counts?),
        Column::new("mean".into(), &means),
        Column::new("std".into(), &stds),
        Column::new("min".into(), &mins),
        Column::new("max".into(), &maxs),
        Column::new("q25".into(), &q25s),
        Column::new("q50".into(), &q50s),
        Column::new("q75".into(), &q75s),
    ])
    .map_err(|error| SfError::Internal(format!("Summary Parquet frame build failed: {}", error)))
}

fn correlation_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let columns = stats
        .values
        .get("columns")
        .and_then(|value| value.as_array())
        .ok_or_else(|| SfError::Internal("Correlation stats missing 'columns' array".into()))?;
    let matrix = stats
        .values
        .get("matrix")
        .and_then(|value| value.as_array())
        .ok_or_else(|| SfError::Internal("Correlation stats missing 'matrix' array".into()))?;
    let method = stats
        .values
        .get("method")
        .and_then(|value| value.as_str())
        .ok_or_else(|| SfError::Internal("Correlation stats missing 'method'".into()))?;

    let column_names: SfResult<Vec<String>> = columns
        .iter()
        .map(|column| {
            column
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| SfError::Internal("Correlation column name must be a string".into()))
        })
        .collect();
    let column_names = column_names?;

    if matrix.len() != column_names.len() {
        return Err(SfError::Internal(
            "Correlation matrix size does not match column count".into(),
        ));
    }

    let mut frame_columns = vec![
        Column::new("method".into(), vec![method; column_names.len()]),
        Column::new("row".into(), &column_names),
    ];

    for (column_index, column_name) in column_names.iter().enumerate() {
        let mut values = Vec::with_capacity(matrix.len());
        for row in matrix {
            let row_values = row.as_array().ok_or_else(|| {
                SfError::Internal("Correlation matrix row must be an array".into())
            })?;
            let value = row_values
                .get(column_index)
                .and_then(|item| item.as_f64())
                .ok_or_else(|| {
                    SfError::Internal("Correlation matrix value must be numeric".into())
                })?;
            values.push(value);
        }
        frame_columns.push(Column::new(column_name.as_str().into(), &values));
    }

    DataFrame::new(frame_columns).map_err(|error| {
        SfError::Internal(format!("Correlation Parquet frame build failed: {}", error))
    })
}

fn distribution_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let column_name = stats
        .values
        .get("column")
        .and_then(|value| value.as_str())
        .ok_or_else(|| SfError::Internal("Distribution stats missing 'column'".into()))?;
    let bins = stats
        .values
        .get("bins")
        .and_then(|value| value.as_array())
        .ok_or_else(|| SfError::Internal("Distribution stats missing 'bins' array".into()))?;

    let lowers: SfResult<Vec<f64>> = bins.iter().map(|bin| required_f64(bin, "lower")).collect();
    let uppers: SfResult<Vec<f64>> = bins.iter().map(|bin| required_f64(bin, "upper")).collect();
    let counts: SfResult<Vec<u64>> = bins.iter().map(|bin| required_u64(bin, "count")).collect();

    DataFrame::new(vec![
        Column::new("column".into(), vec![column_name; bins.len()]),
        Column::new("lower".into(), &lowers?),
        Column::new("upper".into(), &uppers?),
        Column::new("count".into(), &counts?),
    ])
    .map_err(|error| {
        SfError::Internal(format!(
            "Distribution Parquet frame build failed: {}",
            error
        ))
    })
}

fn outlier_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let rows: Vec<(&String, &serde_json::Value)> = stats.values.iter().collect();

    let column_names = rows
        .iter()
        .map(|(name, _)| (*name).clone())
        .collect::<Vec<_>>();
    let methods: SfResult<Vec<String>> = rows
        .iter()
        .map(|(_, value)| required_str(value, "method").map(ToOwned::to_owned))
        .collect();
    let total_rows: SfResult<Vec<u64>> = rows
        .iter()
        .map(|(_, value)| required_u64(value, "total_rows"))
        .collect();
    let outlier_counts: SfResult<Vec<u64>> = rows
        .iter()
        .map(|(_, value)| required_u64(value, "outlier_count"))
        .collect();
    let lower_bounds: SfResult<Vec<f64>> = rows
        .iter()
        .map(|(_, value)| required_f64(value, "lower_bound"))
        .collect();
    let upper_bounds: SfResult<Vec<f64>> = rows
        .iter()
        .map(|(_, value)| required_f64(value, "upper_bound"))
        .collect();
    let q25s: SfResult<Vec<f64>> = rows
        .iter()
        .map(|(_, value)| required_f64(value, "q25"))
        .collect();
    let q75s: SfResult<Vec<f64>> = rows
        .iter()
        .map(|(_, value)| required_f64(value, "q75"))
        .collect();
    let iqrs: SfResult<Vec<f64>> = rows
        .iter()
        .map(|(_, value)| required_f64(value, "iqr"))
        .collect();

    DataFrame::new(vec![
        Column::new("column".into(), &column_names),
        Column::new("method".into(), &methods?),
        Column::new("total_rows".into(), &total_rows?),
        Column::new("outlier_count".into(), &outlier_counts?),
        Column::new("lower_bound".into(), &lower_bounds?),
        Column::new("upper_bound".into(), &upper_bounds?),
        Column::new("q25".into(), &q25s?),
        Column::new("q75".into(), &q75s?),
        Column::new("iqr".into(), &iqrs?),
    ])
    .map_err(|error| SfError::Internal(format!("Outlier Parquet frame build failed: {}", error)))
}

/// Convert a t-test result to a single-row DataFrame with key statistics.
fn ttest_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let test = stats.values.get("test").and_then(|v| v.as_str()).unwrap_or("t_test");
    let col = stats.values.get("column").and_then(|v| v.as_str()).unwrap_or("");
    let t_stat = stats.values.get("t_statistic").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let df_val = stats.values.get("df").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let p_value = stats.values.get("p_value").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let ci_lower = stats.values.get("ci_lower").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let ci_upper = stats.values.get("ci_upper").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let effect_size = stats.values.get("effect_size").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let confidence = stats.values.get("confidence_level").and_then(|v| v.as_f64()).unwrap_or(0.95);
    let alternative = stats.values.get("alternative").and_then(|v| v.as_str()).unwrap_or("two_sided");

    DataFrame::new(vec![
        Column::new("test".into(), vec![test]),
        Column::new("column".into(), vec![col]),
        Column::new("t_statistic".into(), vec![t_stat]),
        Column::new("df".into(), vec![df_val]),
        Column::new("p_value".into(), vec![p_value]),
        Column::new("ci_lower".into(), vec![ci_lower]),
        Column::new("ci_upper".into(), vec![ci_upper]),
        Column::new("effect_size".into(), vec![effect_size]),
        Column::new("confidence_level".into(), vec![confidence]),
        Column::new("alternative".into(), vec![alternative]),
    ])
    .map_err(|e| SfError::Internal(format!("TTest Parquet frame build failed: {}", e)))
}

/// Convert a CI result to a single-row DataFrame.
fn ci_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let estimate = stats.values.get("estimate").and_then(|v| v.as_str()).unwrap_or("");
    let ci_lower = stats.values.get("ci_lower").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let ci_upper = stats.values.get("ci_upper").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let confidence = stats.values.get("confidence_level").and_then(|v| v.as_f64()).unwrap_or(0.95);
    let margin = stats.values.get("margin_of_error").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let method = stats.values.get("method").and_then(|v| v.as_str()).unwrap_or("");

    let mut cols = vec![
        Column::new("estimate".into(), vec![estimate]),
        Column::new("ci_lower".into(), vec![ci_lower]),
        Column::new("ci_upper".into(), vec![ci_upper]),
        Column::new("confidence_level".into(), vec![confidence]),
        Column::new("margin_of_error".into(), vec![margin]),
        Column::new("method".into(), vec![method]),
    ];

    // Add the point estimate value
    if let Some(mean) = stats.values.get("mean").and_then(|v| v.as_f64()) {
        cols.push(Column::new("point_estimate".into(), vec![mean]));
    } else if let Some(p_hat) = stats.values.get("p_hat").and_then(|v| v.as_f64()) {
        cols.push(Column::new("point_estimate".into(), vec![p_hat]));
    } else if let Some(variance) = stats.values.get("variance").and_then(|v| v.as_f64()) {
        cols.push(Column::new("point_estimate".into(), vec![variance]));
    }

    DataFrame::new(cols)
        .map_err(|e| SfError::Internal(format!("CI Parquet frame build failed: {}", e)))
}

/// Generic hypothesis test to DataFrame (proportion, chi-square, variance tests).
fn hypothesis_test_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let test = stats.values.get("test").and_then(|v| v.as_str()).unwrap_or("");
    let df_val = stats.values.get("df").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let p_value = stats.values.get("p_value").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
    let effect_size = stats.values.get("effect_size").and_then(|v| v.as_f64()).unwrap_or(f64::NAN);

    let mut cols = vec![
        Column::new("test".into(), vec![test]),
        Column::new("df".into(), vec![df_val]),
        Column::new("p_value".into(), vec![p_value]),
        Column::new("effect_size".into(), vec![effect_size]),
    ];

    // Add test-specific statistic
    if let Some(z) = stats.values.get("z_statistic").and_then(|v| v.as_f64()) {
        cols.push(Column::new("test_statistic".into(), vec![z]));
    } else if let Some(chi2) = stats.values.get("chi2_statistic").and_then(|v| v.as_f64()) {
        cols.push(Column::new("test_statistic".into(), vec![chi2]));
    }

    // CI if present
    if let Some(ci_lo) = stats.values.get("ci_lower").and_then(|v| v.as_f64()) {
        cols.push(Column::new("ci_lower".into(), vec![ci_lo]));
    }
    if let Some(ci_hi) = stats.values.get("ci_upper").and_then(|v| v.as_f64()) {
        cols.push(Column::new("ci_upper".into(), vec![ci_hi]));
    }

    DataFrame::new(cols)
        .map_err(|e| SfError::Internal(format!("Hypothesis test Parquet frame build failed: {}", e)))
}

fn regression_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let coefficients = stats
        .values
        .get("coefficients")
        .and_then(|v| v.as_array())
        .ok_or_else(|| SfError::Internal("Regression missing 'coefficients' array".into()))?;

    let names: SfResult<Vec<String>> = coefficients
        .iter()
        .map(|c| required_str(c, "name").map(ToOwned::to_owned))
        .collect();
    let estimates: SfResult<Vec<f64>> = coefficients
        .iter()
        .map(|c| required_f64(c, "estimate"))
        .collect();
    let std_errors: SfResult<Vec<f64>> = coefficients
        .iter()
        .map(|c| required_f64(c, "std_error"))
        .collect();
    let t_values: SfResult<Vec<f64>> = coefficients
        .iter()
        .map(|c| required_f64(c, "t_value"))
        .collect();
    let p_values: SfResult<Vec<f64>> = coefficients
        .iter()
        .map(|c| required_f64(c, "p_value"))
        .collect();
    let ci_lowers: SfResult<Vec<f64>> = coefficients
        .iter()
        .map(|c| required_f64(c, "ci_lower"))
        .collect();
    let ci_uppers: SfResult<Vec<f64>> = coefficients
        .iter()
        .map(|c| required_f64(c, "ci_upper"))
        .collect();

    DataFrame::new(vec![
        Column::new("name".into(), &names?),
        Column::new("estimate".into(), &estimates?),
        Column::new("std_error".into(), &std_errors?),
        Column::new("t_value".into(), &t_values?),
        Column::new("p_value".into(), &p_values?),
        Column::new("ci_lower".into(), &ci_lowers?),
        Column::new("ci_upper".into(), &ci_uppers?),
    ])
    .map_err(|e| SfError::Internal(format!("Regression Parquet frame build failed: {}", e)))
}

fn diagnostics_to_dataframe(stats: &StatsArtifact) -> SfResult<DataFrame> {
    let vif_entries = stats
        .values
        .get("vif")
        .and_then(|v| v.as_array())
        .ok_or_else(|| SfError::Internal("Diagnostics missing 'vif' array".into()))?;

    let names: SfResult<Vec<String>> = vif_entries
        .iter()
        .map(|e| required_str(e, "name").map(ToOwned::to_owned))
        .collect();
    let vifs: SfResult<Vec<f64>> = vif_entries
        .iter()
        .map(|e| required_f64(e, "vif"))
        .collect();

    let dw = stats
        .values
        .get("durbin_watson")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let n = vif_entries.len();

    DataFrame::new(vec![
        Column::new("predictor".into(), &names?),
        Column::new("vif".into(), &vifs?),
        Column::new("durbin_watson".into(), vec![dw; n]),
    ])
    .map_err(|e| SfError::Internal(format!("Diagnostics Parquet frame build failed: {}", e)))
}


fn required_str<'a>(value: &'a serde_json::Value, field: &str) -> SfResult<&'a str> {
    value
        .get(field)
        .and_then(|item| item.as_str())
        .ok_or_else(|| SfError::Internal(format!("Missing string field '{}'", field)))
}

fn required_u64(value: &serde_json::Value, field: &str) -> SfResult<u64> {
    value
        .get(field)
        .and_then(|item| item.as_u64())
        .ok_or_else(|| SfError::Internal(format!("Missing integer field '{}'", field)))
}

fn required_f64(value: &serde_json::Value, field: &str) -> SfResult<f64> {
    value
        .get(field)
        .and_then(|item| item.as_f64())
        .ok_or_else(|| SfError::Internal(format!("Missing numeric field '{}'", field)))
}

fn optional_f64(value: &serde_json::Value, field: &str) -> Option<f64> {
    value.get(field).and_then(|item| item.as_f64())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn summary_stats_convert_to_dataframe() {
        let stats = StatsArtifact {
            kind: StatsKind::Summary,
            values: BTreeMap::from([(
                "columns".into(),
                serde_json::json!([
                    {
                        "name": "price",
                        "count": 3,
                        "null_count": 0,
                        "mean": 20.0,
                        "std": 10.0,
                        "min": 10.0,
                        "max": 30.0,
                        "q25": 10.0,
                        "q50": 20.0,
                        "q75": 30.0
                    }
                ]),
            )]),
        };

        let df = stats_to_dataframe(&stats).unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(
            df.get_column_names(),
            &[
                "name",
                "count",
                "null_count",
                "mean",
                "std",
                "min",
                "max",
                "q25",
                "q50",
                "q75"
            ]
        );
    }

    #[test]
    fn correlation_stats_convert_to_dataframe() {
        let stats = StatsArtifact {
            kind: StatsKind::Correlation,
            values: BTreeMap::from([
                ("method".into(), serde_json::json!("pearson")),
                ("columns".into(), serde_json::json!(["price", "quantity"])),
                ("matrix".into(), serde_json::json!([[1.0, 0.5], [0.5, 1.0]])),
            ]),
        };

        let df = stats_to_dataframe(&stats).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(
            df.get_column_names(),
            &["method", "row", "price", "quantity"]
        );
    }
}
