use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

mod support;

use support::{output_path, workspace_root};

fn ts_fixture_path() -> String {
    let root = workspace_root();
    let path = root.join("tests/fixtures/parquet/timeseries.parquet");
    if !path.exists() {
        setup_ts_fixture();
    }
    path.to_string_lossy().to_string()
}

fn setup_ts_fixture() {
    use polars::prelude::*;
    let root = workspace_root();
    let path = root.join("tests/fixtures/parquet/timeseries.parquet");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

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

    let mut file = std::fs::File::create(&path).unwrap();
    ParquetWriter::new(&mut file)
        .finish(&mut df)
        .unwrap();
}

#[test]
fn ts_adf_terminal_works() {
    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "adf",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Augmented Dickey-Fuller"))
        .stdout(predicate::str::contains("Test Statistic"))
        .stdout(predicate::str::contains("Critical Values"));
}

#[test]
fn ts_adf_with_max_lag() {
    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "adf",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
            "--max-lag",
            "4",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Lags Used"));
}

#[test]
fn ts_adf_json_output() {
    let output = output_path("ts-adf", "json");

    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "adf",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
            "--format",
            "json",
            "--out",
            &output.to_string_lossy(),
        ])
        .assert()
        .success();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    let artifacts = json.get("artifacts").unwrap().as_object().unwrap();
    assert!(artifacts.contains_key("step_01_ts_adf"));
}

#[test]
fn ts_adf_missing_column() {
    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "adf",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "nonexistent",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn ts_decompose_terminal_works() {
    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "decompose",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
            "--period",
            "12",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Decomposition"))
        .stdout(predicate::str::contains("Period"))
        .stdout(predicate::str::contains("Trend"));
}

#[test]
fn ts_decompose_json_output() {
    let output = output_path("ts-decompose", "json");

    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "decompose",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
            "--period",
            "12",
            "--format",
            "json",
            "--out",
            &output.to_string_lossy(),
        ])
        .assert()
        .success();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    let artifacts = json.get("artifacts").unwrap().as_object().unwrap();
    assert!(artifacts.contains_key("step_01_ts_decompose"));
}

#[test]
fn ts_forecast_terminal_works() {
    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "forecast",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
            "--order",
            "1,1,0",
            "--horizon",
            "5",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("ARIMA"))
        .stdout(predicate::str::contains("T+1"))
        .stdout(predicate::str::contains("Yhat"));
}

#[test]
fn ts_forecast_json_output() {
    let output = output_path("ts-forecast", "json");

    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "forecast",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "value",
            "--order",
            "1,1,1",
            "--horizon",
            "10",
            "--format",
            "json",
            "--out",
            &output.to_string_lossy(),
        ])
        .assert()
        .success();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    let artifacts = json.get("artifacts").unwrap().as_object().unwrap();
    assert!(artifacts.contains_key("step_01_ts_forecast"));
}

#[test]
fn ts_forecast_missing_column() {
    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "ts",
            "forecast",
            "--source",
            &ts_fixture_path(),
            "--time-col",
            "time",
            "--value-col",
            "nonexistent",
            "--order",
            "1,1,0",
            "--horizon",
            "5",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
