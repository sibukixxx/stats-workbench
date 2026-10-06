use assert_cmd::Command;
use polars::prelude::*;
use predicates::prelude::*;
use std::fs;
mod support;

use support::{fixture_path, output_path, setup_fixture};

#[test]
fn eda_summary_terminal_works() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            &fixture_path(),
            "--cols",
            "price,quantity",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("quantity"));
}

#[test]
fn eda_summary_all_cols() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args(["eda", "summary", "--source", &fixture_path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("revenue"));
}

#[test]
fn eda_corr_works() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "corr",
            "--source",
            &fixture_path(),
            "--cols",
            "price,quantity,revenue",
            "--method",
            "pearson",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("pearson"));
}

#[test]
fn eda_corr_spearman() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "corr",
            "--source",
            &fixture_path(),
            "--cols",
            "price,quantity",
            "--method",
            "spearman",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("spearman"));
}

#[test]
fn eda_dist_works() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "dist",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--bins",
            "5",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("Lower"));
}

#[test]
fn eda_summary_invalid_column() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            &fixture_path(),
            "--cols",
            "nonexistent_column",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn eda_summary_missing_file() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            "nonexistent_file.parquet",
            "--cols",
            "a",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn eda_corr_too_few_cols() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "corr",
            "--source",
            &fixture_path(),
            "--cols",
            "price",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("at least 2"));
}

#[test]
fn eda_summary_json_output_excludes_source_artifact() {
    setup_fixture();
    let output_path = output_path("eda-summary", "json");

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            &fixture_path(),
            "--cols",
            "price,quantity",
            "--format",
            "json",
            "--out",
            &output_path.to_string_lossy(),
        ])
        .assert()
        .success();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();
    let artifacts = json
        .get("artifacts")
        .and_then(|value| value.as_object())
        .unwrap();

    assert_eq!(artifacts.len(), 1);
    assert!(artifacts.contains_key("step_01_eda_summary"));
    assert!(!artifacts.contains_key("source"));
}

#[test]
fn eda_summary_parquet_output_writes_summary_artifact() {
    setup_fixture();
    let output_path = output_path("eda-summary", "parquet");

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            &fixture_path(),
            "--cols",
            "price,quantity",
            "--format",
            "parquet",
            "--out",
            &output_path.to_string_lossy(),
        ])
        .assert()
        .success();

    let file = std::fs::File::open(&output_path).unwrap();
    let df = ParquetReader::new(file).finish().unwrap();
    let column_names = df.get_column_names();

    assert_eq!(df.height(), 2);
    assert!(column_names.iter().any(|name| name.as_str() == "name"));
    assert!(column_names.iter().any(|name| name.as_str() == "mean"));
    assert!(!column_names.iter().any(|name| name.as_str() == "price"));
    assert!(!column_names.iter().any(|name| name.as_str() == "quantity"));
}
