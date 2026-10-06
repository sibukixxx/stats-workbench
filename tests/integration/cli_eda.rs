use assert_cmd::Command;
use predicates::prelude::*;

fn setup_fixture() {
    // Generate test parquet if it doesn't exist
    let path = "tests/fixtures/parquet/sales.parquet";
    if std::path::Path::new(path).exists() {
        return;
    }

    use polars::prelude::*;
    std::fs::create_dir_all("tests/fixtures/parquet").unwrap();

    let df = df! {
        "price" => &[10.0_f64, 20.0, 30.0, 40.0, 50.0, 15.0, 25.0, 35.0, 45.0, 55.0],
        "quantity" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 1.5, 2.5, 3.5, 4.5, 5.5],
        "revenue" => &[10.0_f64, 40.0, 90.0, 160.0, 250.0, 22.5, 62.5, 122.5, 202.5, 302.5],
        "category" => &["A", "B", "A", "B", "A", "B", "A", "B", "A", "B"],
    }
    .unwrap();

    let mut file = std::fs::File::create(path).unwrap();
    ParquetWriter::new(&mut file)
        .finish(&mut df.clone())
        .unwrap();
}

#[test]
fn eda_summary_parquet_works() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "summary",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--cols",
        "price,quantity",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("quantity"));
}

#[test]
fn eda_summary_all_cols() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "summary",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("revenue"));
}

#[test]
fn eda_corr_works() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "corr",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--cols",
        "price,quantity,revenue",
        "--method",
        "pearson",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("pearson"));
}

#[test]
fn eda_corr_spearman() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "corr",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--cols",
        "price,quantity",
        "--method",
        "spearman",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("spearman"));
}

#[test]
fn eda_dist_works() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "dist",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--col",
        "price",
        "--bins",
        "5",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("price"))
        .stdout(predicate::str::contains("Lower"));
}

#[test]
fn eda_summary_invalid_column() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "summary",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--cols",
        "nonexistent_column",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn eda_summary_missing_file() {
    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "summary",
        "--source",
        "nonexistent_file.parquet",
        "--cols",
        "a",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn eda_corr_too_few_cols() {
    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args([
        "eda",
        "corr",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--cols",
        "price",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("at least 2"));
}
