use assert_cmd::Command;
use predicates::prelude::*;
mod support;

use support::setup_sqlite_fixture;

#[test]
fn sqlite_summary_with_table_works() {
    let source = setup_sqlite_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            &source,
            "--table",
            "orders",
            "--cols",
            "price,revenue",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("revenue"));
}

#[test]
fn sqlite_outlier_with_query_works() {
    let source = setup_sqlite_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "stats",
            "outlier",
            "--source",
            &source,
            "--query",
            "SELECT revenue FROM orders",
            "--cols",
            "revenue",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("revenue"))
        .stdout(predicate::str::contains("Outliers"));
}

#[test]
fn sqlite_source_requires_table_or_query() {
    let source = setup_sqlite_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args(["eda", "summary", "--source", &source, "--cols", "price"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Database source requires either 'table' or 'query'",
        ));
}

#[test]
fn postgres_source_requires_table_or_query() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "eda",
            "summary",
            "--source",
            "pg://localhost:5432/app",
            "--cols",
            "price",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Database source requires either 'table' or 'query'",
        ));
}
