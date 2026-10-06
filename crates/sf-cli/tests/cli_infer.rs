use assert_cmd::Command;
use predicates::prelude::*;
mod support;

use support::{fixture_path, output_path, setup_fixture};

// ── One-sample t-test ──

#[test]
fn infer_ttest1_terminal_output() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "ttest1",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--mu0",
            "30.0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("t-statistic"))
        .stdout(predicate::str::contains("p-value"))
        .stdout(predicate::str::contains("df"))
        .stdout(predicate::str::contains("CI"));
}

#[test]
fn infer_ttest1_json_output() {
    setup_fixture();
    let out = output_path("ttest1", "json");

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "ttest1",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--mu0",
            "30.0",
            "--format",
            "json",
            "--out",
            &out.to_string_lossy(),
        ])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let artifacts = &json["artifacts"];
    let first_artifact = artifacts.as_object().unwrap().values().next().unwrap();
    assert_eq!(first_artifact["kind"], "TTest");
    assert!(first_artifact["values"]["p_value"].is_number());
    assert!(first_artifact["values"]["t_statistic"].is_number());
    assert!(first_artifact["values"]["ci_lower"].is_number());
    assert!(first_artifact["values"]["ci_upper"].is_number());
    assert!(first_artifact["values"]["effect_size"].is_number());
}

#[test]
fn infer_ttest1_alternative_less() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "ttest1",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--mu0",
            "30.0",
            "--alternative",
            "less",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("less"));
}

// ── Two-sample t-test ──

#[test]
fn infer_ttest2_terminal_output() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "ttest2",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--group-col",
            "category",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("t-statistic"))
        .stdout(predicate::str::contains("p-value"))
        .stdout(predicate::str::contains("Mean"));
}

// ── Mean CI ──

#[test]
fn infer_mean_ci_terminal_output() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "mean-ci",
            "--source",
            &fixture_path(),
            "--col",
            "price",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Confidence Interval"))
        .stdout(predicate::str::contains("CI"));
}

#[test]
fn infer_mean_ci_custom_confidence() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "mean-ci",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--confidence",
            "0.99",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("99% CI"));
}

// ── Proportion test ──

#[test]
fn infer_prop_test_terminal_output() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "prop-test",
            "--n",
            "100",
            "--successes",
            "60",
            "--p0",
            "0.5",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("z-statistic"))
        .stdout(predicate::str::contains("p-value"));
}

// ── Proportion CI ──

#[test]
fn infer_prop_ci_terminal_output() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args(["infer", "prop-ci", "--n", "100", "--successes", "60"])
        .assert()
        .success()
        .stdout(predicate::str::contains("proportion"))
        .stdout(predicate::str::contains("CI"));
}

// ── Variance test ──

#[test]
fn infer_var_test_terminal_output() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "var-test",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--sigma2-0",
            "200.0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("chi-squared"))
        .stdout(predicate::str::contains("p-value"));
}

// ── Variance CI ──

#[test]
fn infer_var_ci_terminal_output() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "var-ci",
            "--source",
            &fixture_path(),
            "--col",
            "price",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("variance"))
        .stdout(predicate::str::contains("CI"));
}

// ── Chi-squared goodness of fit ──

#[test]
fn infer_chisq_goodness_terminal_output() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "chisq-goodness",
            "--observed",
            "20,30,50",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("chi_squared_goodness_of_fit"))
        .stdout(predicate::str::contains("p-value"));
}

#[test]
fn infer_chisq_goodness_with_expected() {
    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "chisq-goodness",
            "--observed",
            "20,30,50",
            "--expected",
            "33.33,33.33,33.34",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("p-value"));
}

// ── Chi-squared independence ──

#[test]
fn infer_chisq_independence_terminal_output() {
    setup_fixture();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "chisq-independence",
            "--source",
            &fixture_path(),
            "--col1",
            "category",
            "--col2",
            "category",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("chi_squared_independence"));
}

// ── Recipe integration ──

#[test]
fn infer_recipe_ttest_one_sample() {
    setup_fixture();

    let recipe_path = output_path("recipe_ttest", "toml");
    let recipe = format!(
        r#"
[source]
type = "parquet"
path = "{}"

[[steps]]
type = "infer.ttest_one_sample"
col = "price"
mu0 = 30.0
confidence = 0.95
alternative = "two_sided"

[output]
terminal = true
"#,
        fixture_path()
    );

    std::fs::create_dir_all(recipe_path.parent().unwrap()).unwrap();
    std::fs::write(&recipe_path, recipe).unwrap();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args(["run", "--recipe", &recipe_path.to_string_lossy()])
        .assert()
        .success()
        .stdout(predicate::str::contains("t-statistic"))
        .stdout(predicate::str::contains("p-value"));
}

#[test]
fn infer_recipe_mean_ci() {
    setup_fixture();

    let recipe_path = output_path("recipe_mean_ci", "toml");
    let recipe = format!(
        r#"
[source]
type = "parquet"
path = "{}"

[[steps]]
type = "infer.mean_ci"
col = "price"
confidence = 0.99

[output]
terminal = true
"#,
        fixture_path()
    );

    std::fs::create_dir_all(recipe_path.parent().unwrap()).unwrap();
    std::fs::write(&recipe_path, recipe).unwrap();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args(["run", "--recipe", &recipe_path.to_string_lossy()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Confidence Interval"))
        .stdout(predicate::str::contains("99% CI"));
}

#[test]
fn infer_recipe_mixed_pipeline() {
    setup_fixture();

    let recipe_path = output_path("recipe_mixed", "toml");
    let json_out = output_path("recipe_mixed_out", "json");
    let recipe = format!(
        r#"
[source]
type = "parquet"
path = "{}"

[[steps]]
type = "eda.summary"
cols = ["price"]

[[steps]]
type = "infer.ttest_one_sample"
col = "price"
mu0 = 25.0

[[steps]]
type = "infer.mean_ci"
col = "price"

[output]
format = "json"
path = "{}"
"#,
        fixture_path(),
        json_out.to_string_lossy()
    );

    std::fs::create_dir_all(recipe_path.parent().unwrap()).unwrap();
    std::fs::write(&recipe_path, recipe).unwrap();

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args(["run", "--recipe", &recipe_path.to_string_lossy()])
        .assert()
        .success();

    let content = std::fs::read_to_string(&json_out).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    let artifacts = json["artifacts"].as_object().unwrap();
    // 4 artifacts: source + 3 steps (summary, ttest, mean_ci)
    assert_eq!(artifacts.len(), 4);
}

// ── Parquet output ──

#[test]
fn infer_ttest1_parquet_output() {
    setup_fixture();
    let out = output_path("ttest1_out", "parquet");

    Command::cargo_bin("stats-workbench")
        .unwrap()
        .args([
            "infer",
            "ttest1",
            "--source",
            &fixture_path(),
            "--col",
            "price",
            "--mu0",
            "30.0",
            "--format",
            "parquet",
            "--out",
            &out.to_string_lossy(),
        ])
        .assert()
        .success();

    assert!(out.exists(), "Parquet file should be created");
}
