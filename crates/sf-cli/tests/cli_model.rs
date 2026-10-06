use assert_cmd::Command;
use predicates::prelude::*;
mod support;

use support::{fixture_path, regression_fixture_path, setup_fixture, setup_regression_fixture};

#[test]
fn model_regress_works() {
    setup_regression_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "regress",
            "--source",
            &regression_fixture_path(),
            "--target",
            "revenue",
            "--predictors",
            "price,quantity",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("OLS Regression"))
        .stdout(predicate::str::contains("R²"))
        .stdout(predicate::str::contains("(Intercept)"))
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("quantity"));
}

#[test]
fn model_regress_simple_regression() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "regress",
            "--source",
            &fixture_path(),
            "--target",
            "revenue",
            "--predictors",
            "price",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("OLS Regression"))
        .stdout(predicate::str::contains("(Intercept)"))
        .stdout(predicate::str::contains("price"));
}

#[test]
fn model_regress_json_output() {
    setup_regression_fixture();

    let output_file = support::output_path("regress", "json");
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "regress",
            "--source",
            &regression_fixture_path(),
            "--target",
            "revenue",
            "--predictors",
            "price,quantity",
            "--format",
            "json",
            "--out",
            &output_file.to_string_lossy(),
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("artifacts").is_some());

    let artifacts = json.get("artifacts").unwrap().as_object().unwrap();
    assert!(artifacts.contains_key("step_01_model_regress"));

    let _ = std::fs::remove_file(&output_file);
}

#[test]
fn model_regress_missing_target() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "regress",
            "--source",
            &fixture_path(),
            "--predictors",
            "price",
        ])
        .assert()
        .failure();
}

#[test]
fn model_regress_missing_predictors() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "regress",
            "--source",
            &fixture_path(),
            "--target",
            "revenue",
        ])
        .assert()
        .failure();
}

#[test]
fn model_diagnose_works() {
    setup_regression_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "diagnose",
            "--source",
            &regression_fixture_path(),
            "--target",
            "revenue",
            "--predictors",
            "price,quantity",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Regression Diagnostics"))
        .stdout(predicate::str::contains("Durbin-Watson"))
        .stdout(predicate::str::contains("VIF"))
        .stdout(predicate::str::contains("Leverage"))
        .stdout(predicate::str::contains("Cook's Distance"));
}

#[test]
fn model_diagnose_single_predictor() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "model",
            "diagnose",
            "--source",
            &fixture_path(),
            "--target",
            "revenue",
            "--predictors",
            "price",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Durbin-Watson"))
        .stdout(predicate::str::contains("VIF"));
}
