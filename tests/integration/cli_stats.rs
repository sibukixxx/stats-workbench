use assert_cmd::Command;
use predicates::prelude::*;

fn setup_fixture() {
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
fn stats_outlier_works() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.args([
        "stats",
        "outlier",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
        "--cols",
        "revenue",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("revenue"))
        .stdout(predicate::str::contains("Outliers"));
}

#[test]
fn stats_outlier_missing_cols() {
    let mut cmd = Command::cargo_bin("sf").unwrap();
    cmd.args([
        "stats",
        "outlier",
        "--source",
        "tests/fixtures/parquet/sales.parquet",
    ]);

    // clap should fail because --cols is missing
    cmd.assert().failure();
}
