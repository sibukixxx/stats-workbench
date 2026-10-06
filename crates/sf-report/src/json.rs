use sf_core::{Artifact, ArtifactStore, ExecutionMeta, SfResult};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Write artifacts as structured JSON.
pub fn write_json(
    artifacts: &ArtifactStore,
    keys: &[&str],
    meta: &ExecutionMeta,
    path: Option<&str>,
) -> SfResult<()> {
    let mut output = BTreeMap::new();

    // Execution metadata
    output.insert(
        "execution".to_string(),
        serde_json::json!({
            "tool": meta.tool,
            "version": meta.version,
            "recipe": meta.recipe_path,
            "started_at": meta.started_at,
            "steps_executed": meta.steps_executed,
        }),
    );

    // Artifacts
    let mut artifact_map = BTreeMap::new();
    for key in keys {
        if let Some(artifact) = artifacts.get(key) {
            artifact_map.insert(key.to_string(), artifact_to_json(artifact));
        }
    }
    output.insert("artifacts".to_string(), serde_json::to_value(artifact_map)?);

    let json = serde_json::to_string_pretty(&output)?;

    match path {
        Some(p) => {
            if let Some(parent) = Path::new(p).parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(p, &json)?;
            tracing::info!("JSON output written to {}", p);
        }
        None => {
            println!("{}", json);
        }
    }
    Ok(())
}

fn artifact_to_json(artifact: &Artifact) -> serde_json::Value {
    match artifact {
        Artifact::Stats(stats) => serde_json::json!({
            "kind": stats.kind,
            "values": stats.values,
        }),
        Artifact::Text(text) => serde_json::json!({ "text": text }),
        Artifact::Forecast(fc) => serde_json::to_value(fc).unwrap_or_default(),
        Artifact::Dataset(ds) => {
            if let Ok(df) = ds.collect() {
                serde_json::json!({
                    "kind": "Dataset",
                    "rows": df.height(),
                    "cols": df.width(),
                })
            } else {
                serde_json::json!({ "kind": "Dataset", "error": "failed to collect" })
            }
        }
        Artifact::Table(df) => {
            serde_json::json!({
                "kind": "Table",
                "rows": df.height(),
                "cols": df.width(),
            })
        }
        Artifact::Plot(plot) => {
            serde_json::json!({
                "kind": "Plot",
                "title": plot.title,
            })
        }
        Artifact::Model(m) => {
            serde_json::json!({
                "kind": "Model",
                "name": m.name,
                "size_bytes": m.data.len(),
            })
        }
    }
}
