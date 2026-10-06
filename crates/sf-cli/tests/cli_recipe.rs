use assert_cmd::Command;
use predicates::prelude::*;
mod support;

use support::{setup_fixture, setup_regression_fixture, workspace_root};

#[test]
fn recipe_eda_summary() {
    setup_fixture();
    let root = workspace_root();

    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.current_dir(&root);
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/eda_summary.toml"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("quantity"))
        .stdout(predicate::str::contains("revenue"));
}

#[test]
fn recipe_eda_full_pipeline() {
    setup_fixture();
    let root = workspace_root();

    let output_path = root.join("tests/fixtures/output/eda_full_result.json");
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.current_dir(&root);
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/eda_full.toml"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("summary"))
        .stdout(predicate::str::contains("corr"))
        .stdout(predicate::str::contains("dist"))
        .stdout(predicate::str::contains("outliers"));

    assert!(output_path.exists(), "JSON output file should be created");

    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("execution").is_some());
    assert!(json.get("artifacts").is_some());

    let _ = std::fs::remove_file(&output_path);
    let _ = std::fs::remove_dir(output_path.parent().unwrap());
}

#[test]
fn recipe_regression_pipeline() {
    setup_regression_fixture();
    let root = workspace_root();

    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.current_dir(&root);
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/regression.toml"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("OLS Regression"))
        .stdout(predicate::str::contains("R²"))
        .stdout(predicate::str::contains("Durbin-Watson"))
        .stdout(predicate::str::contains("VIF"));
}

#[test]
fn recipe_ts_pipeline() {
    // Ensure timeseries fixture exists
    let root = workspace_root();
    let ts_path = root.join("tests/fixtures/parquet/timeseries.parquet");
    if !ts_path.exists() {
        use polars::prelude::*;
        std::fs::create_dir_all(ts_path.parent().unwrap()).unwrap();
        let n = 120;
        let time: Vec<i64> = (0..n as i64).collect();
        let values: Vec<f64> = (0..n)
            .map(|i| {
                let trend = 10.0 + 0.3 * i as f64;
                let seasonal =
                    5.0 * (2.0 * std::f64::consts::PI * i as f64 / 12.0).sin();
                let noise = ((i * 7 + 3) % 11) as f64 * 0.1 - 0.5;
                trend + seasonal + noise
            })
            .collect();
        let mut df = df! {
            "time" => &time,
            "value" => &values,
        }
        .unwrap();
        let mut file = std::fs::File::create(&ts_path).unwrap();
        ParquetWriter::new(&mut file).finish(&mut df).unwrap();
    }

    let output_path = root.join("tests/fixtures/output/ts_pipeline_result.json");
    let _ = std::fs::remove_file(&output_path);

    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.current_dir(&root);
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/ts_pipeline.toml"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Augmented Dickey-Fuller"))
        .stdout(predicate::str::contains("Decomposition"))
        .stdout(predicate::str::contains("ARIMA"));

    assert!(output_path.exists(), "JSON output file should be created");

    let content = std::fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let artifacts = json.get("artifacts").unwrap().as_object().unwrap();
    assert!(artifacts.contains_key("adf_test"));
    assert!(artifacts.contains_key("decomposition"));
    assert!(artifacts.contains_key("forecast"));

    let _ = std::fs::remove_file(&output_path);
}

#[test]
fn recipe_broken_toml() {
    let root = workspace_root();

    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.current_dir(&root);
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/broken.toml"]);

    cmd.assert().failure().stderr(
        predicate::str::contains("TOML parse error").or(predicate::str::contains("recipe")),
    );
}

#[test]
fn recipe_nonexistent_file() {
    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.args(["run", "--recipe", "nonexistent_recipe.toml"]);

    cmd.assert().failure();
}
