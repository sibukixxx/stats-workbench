use polars::prelude::*;
use statforge::{output, source, step, StatForge};
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fixture_path() -> PathBuf {
    workspace_root().join("tests/fixtures/parquet/sales.parquet")
}

fn output_path(name: &str, extension: &str) -> PathBuf {
    workspace_root()
        .join("target/statforge-sdk-tests")
        .join(format!("{name}.{extension}"))
}

fn setup_fixture() {
    let path = fixture_path();
    if path.exists() {
        return;
    }

    fs::create_dir_all(path.parent().unwrap()).unwrap();

    let df = df! {
        "price" => &[10.0_f64, 20.0, 30.0, 40.0, 50.0, 15.0, 25.0, 35.0, 45.0, 55.0],
        "quantity" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 1.5, 2.5, 3.5, 4.5, 5.5],
        "revenue" => &[10.0_f64, 40.0, 90.0, 160.0, 250.0, 22.5, 62.5, 122.5, 202.5, 302.5],
        "category" => &["A", "B", "A", "B", "A", "B", "A", "B", "A", "B"],
    }
    .unwrap();

    let mut file = fs::File::create(path).unwrap();
    ParquetWriter::new(&mut file)
        .finish(&mut df.clone())
        .unwrap();
}

#[tokio::test]
async fn run_step_returns_summary_artifact() {
    setup_fixture();

    let source = source::parquet(fixture_path().display().to_string());
    let step = step::summary(["price", "quantity"]);
    let result = StatForge::run_step(&source, &step).await.unwrap();

    assert_eq!(result.metadata().source_type, "parquet");
    assert_eq!(result.execution().steps_executed, 1);
    assert_eq!(result.artifact_names(), &["source", "step_01_eda_summary"]);

    let artifact = result.require_artifact("step_01_eda_summary").unwrap();
    let json = serde_json::to_value(artifact_to_json(artifact)).unwrap();
    assert_eq!(
        json.get("kind").and_then(|value| value.as_str()),
        Some("Summary")
    );
}

#[tokio::test]
async fn run_recipe_file_sets_recipe_path() {
    setup_fixture();

    let recipe_path = output_path("sdk-recipe", "toml");
    let fixture = fixture_path();
    fs::create_dir_all(recipe_path.parent().unwrap()).unwrap();
    fs::write(
        &recipe_path,
        format!(
            r#"[source]
type = "parquet"
path = "{}"

[[steps]]
id = "summary"
type = "eda.summary"
cols = ["price", "quantity"]
"#,
            fixture.display()
        ),
    )
    .unwrap();

    let result = StatForge::run_recipe_file(&recipe_path).await.unwrap();
    let expected_recipe_path = recipe_path.display().to_string();

    assert_eq!(
        result.execution().recipe_path.as_deref(),
        Some(expected_recipe_path.as_str())
    );
    assert!(result.artifact_names().contains(&"summary".to_string()));
}

#[tokio::test]
async fn json_output_can_target_selected_artifacts() {
    setup_fixture();

    let output_path = output_path("summary", "json");
    let _ = fs::remove_file(&output_path);

    let source = source::parquet(fixture_path().display().to_string());
    let step = step::summary(["price", "quantity"]);
    let result = StatForge::run_step(&source, &step).await.unwrap();

    output::to_json(
        &result,
        &["step_01_eda_summary"],
        Some(&output_path.to_string_lossy()),
    )
    .unwrap();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();
    let artifacts = json
        .get("artifacts")
        .and_then(|value| value.as_object())
        .unwrap();

    assert_eq!(artifacts.len(), 1);
    assert!(artifacts.contains_key("step_01_eda_summary"));
}

fn artifact_to_json(artifact: &statforge::Artifact) -> serde_json::Value {
    match artifact {
        statforge::Artifact::Stats(stats) => serde_json::json!({
            "kind": format!("{:?}", stats.kind),
            "values": stats.values,
        }),
        _ => serde_json::Value::Null,
    }
}
