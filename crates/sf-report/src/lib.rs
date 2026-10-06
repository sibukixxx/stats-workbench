pub mod json;
pub mod parquet;
pub mod terminal;

use sf_core::{ArtifactStore, ExecutionMeta, OutputFormat, OutputSpec, SfError, SfResult};

/// Write artifacts according to the OutputSpec.
pub fn write_output(
    artifacts: &ArtifactStore,
    spec: &OutputSpec,
    meta: &ExecutionMeta,
) -> SfResult<()> {
    // Determine which artifacts to output
    let keys: Vec<&str> = if spec.artifacts.is_empty() {
        artifacts.keys().iter().map(|s| s.as_str()).collect()
    } else {
        spec.artifacts.iter().map(|s| s.as_str()).collect()
    };

    if spec.terminal {
        for key in &keys {
            if let Some(artifact) = artifacts.get(key) {
                terminal::render(key, artifact);
            }
        }
    }

    if let Some(ref format) = spec.format {
        match format {
            OutputFormat::Json => {
                json::write_json(artifacts, &keys, meta, spec.path.as_deref())?;
            }
            OutputFormat::Parquet => {
                parquet::write_parquet(artifacts, &keys, spec.path.as_deref())?;
            }
            _ => {
                return Err(SfError::Validation(format!(
                    "Output format '{:?}' not yet supported in Phase 1",
                    format
                )));
            }
        }
    }

    Ok(())
}

/// Render all artifacts to terminal (for CLI commands without explicit output spec).
pub fn render_to_terminal(artifacts: &ArtifactStore) {
    for (key, artifact) in artifacts.iter_ordered() {
        terminal::render(key, artifact);
    }
}
