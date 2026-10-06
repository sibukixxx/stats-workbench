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
fn recipe_eda_summary() {
    setup_fixture();

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
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

    // Clean up previous output
    let _ = std::fs::remove_file("tests/fixtures/output/eda_full_result.json");

    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/eda_full.toml"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("summary"))
        .stdout(predicate::str::contains("corr"))
        .stdout(predicate::str::contains("dist"))
        .stdout(predicate::str::contains("outliers"));

    // Verify JSON output was written
    assert!(
        std::path::Path::new("tests/fixtures/output/eda_full_result.json").exists(),
        "JSON output file should be created"
    );

    // Verify JSON structure
    let content =
        std::fs::read_to_string("tests/fixtures/output/eda_full_result.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("execution").is_some());
    assert!(json.get("artifacts").is_some());

    // Clean up
    let _ = std::fs::remove_file("tests/fixtures/output/eda_full_result.json");
    let _ = std::fs::remove_dir("tests/fixtures/output");
}

#[test]
fn recipe_broken_toml() {
    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args(["run", "--recipe", "tests/fixtures/recipes/broken.toml"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("TOML parse error").or(predicate::str::contains("recipe")));
}

#[test]
fn recipe_nonexistent_file() {
    let mut cmd = Command::cargo_bin("stats-workbench").unwrap();
    cmd.args(["run", "--recipe", "nonexistent_recipe.toml"]);

    cmd.assert().failure();
}
