use assert_cmd::Command;
use predicates::prelude::*;
mod support;

use support::{fixture_path, setup_fixture};

#[test]
fn stats_outlier_works() {
    setup_fixture();

    Command::cargo_bin("sf")
        .unwrap()
        .args([
            "stats",
            "outlier",
            "--source",
            &fixture_path(),
            "--cols",
            "revenue",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("revenue"))
        .stdout(predicate::str::contains("Outliers"));
}

#[test]
fn stats_outlier_missing_cols() {
    Command::cargo_bin("sf")
        .unwrap()
        .args(["stats", "outlier", "--source", &fixture_path()])
        .assert()
        .failure();
}
