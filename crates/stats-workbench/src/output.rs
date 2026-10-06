use crate::RunResult;
use sf_core::{OutputSpec, SfResult};

/// Render all produced artifacts to the terminal.
pub fn to_terminal(result: &RunResult) {
    sf_report::render_to_terminal(result.artifacts());
}

/// Write artifacts according to an `OutputSpec`.
pub fn write_output(result: &RunResult, spec: &OutputSpec) -> SfResult<()> {
    sf_report::write_output(result.artifacts(), spec, result.execution())
}

/// Write selected artifacts to JSON. An empty `artifacts` slice writes all artifacts.
pub fn to_json(result: &RunResult, artifacts: &[&str], path: Option<&str>) -> SfResult<()> {
    let keys = selected_keys(result, artifacts);
    sf_report::json::write_json(result.artifacts(), &keys, result.execution(), path)
}

/// Write selected artifacts to Parquet. An empty `artifacts` slice writes the first compatible artifact.
pub fn to_parquet(result: &RunResult, artifacts: &[&str], path: Option<&str>) -> SfResult<()> {
    let keys = selected_keys(result, artifacts);
    sf_report::parquet::write_parquet(result.artifacts(), &keys, path)
}

fn selected_keys<'a>(result: &'a RunResult, artifacts: &'a [&'a str]) -> Vec<&'a str> {
    if artifacts.is_empty() {
        result
            .artifact_names()
            .iter()
            .map(std::string::String::as_str)
            .collect()
    } else {
        artifacts.to_vec()
    }
}
