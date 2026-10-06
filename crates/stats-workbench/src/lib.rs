//! Public Rust SDK for Stats Workbench.
//!
//! The `statforge` crate exposes stable, high-level entrypoints for running
//! analysis steps or full TOML recipes without depending on internal crates.

pub mod output;
pub mod prelude;
pub mod source;
pub mod step;

use sf_core::{SfError, SfResult};
use sf_engine::{PipelineContext, PipelineEngine};
use std::path::Path;

pub use sf_core::{
    Artifact, CorrMethod, CorrelationStepSpec, DatasetMeta, DistStepSpec, ExecutionMeta,
    OutlierAction, OutlierMethod, OutlierStepSpec, OutputFormat, OutputSpec, Recipe, StatsArtifact,
    StatsKind, StepHeader, StepSpec, SummaryStepSpec,
};

pub type Result<T> = SfResult<T>;
pub type StatsWorkbenchError = SfError;
pub type Source = sf_core::SourceSpec;
pub type Step = sf_core::StepSpec;

/// High-level SDK entrypoints.
pub struct StatsWorkbench;

impl StatsWorkbench {
    /// Parse a recipe TOML document with environment-variable expansion.
    ///
    /// Only `SF_*` variables and `DATABASE_URL` may be referenced, and output
    /// paths must stay under the working directory (see `sf-engine`).
    pub fn parse_recipe(content: &str) -> Result<Recipe> {
        PipelineEngine::parse_recipe(content)
    }

    /// Read and parse a recipe TOML file.
    pub fn load_recipe(path: impl AsRef<Path>) -> Result<Recipe> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|error| {
            SfError::Io(format!(
                "Cannot read recipe '{}': {}",
                path.display(),
                error
            ))
        })?;
        PipelineEngine::parse_recipe_in(&content, path.parent())
    }

    /// Execute a validated recipe.
    pub async fn run_recipe(recipe: &Recipe) -> Result<RunResult> {
        Ok(RunResult::new(PipelineEngine::execute(recipe).await?))
    }

    /// Read, parse, and execute a recipe TOML file.
    pub async fn run_recipe_file(path: impl AsRef<Path>) -> Result<RunResult> {
        let path = path.as_ref();
        let recipe = Self::load_recipe(path)?;
        let mut result = Self::run_recipe(&recipe).await?;
        result.context.execution.recipe_path = Some(path.display().to_string());
        Ok(result)
    }

    /// Execute a single step against a source.
    pub async fn run_step(source: &Source, step: &Step) -> Result<RunResult> {
        Ok(RunResult::new(
            PipelineEngine::execute_single(source, step).await?,
        ))
    }

    /// Execute a step that doesn't require a data source (e.g. proportion tests).
    pub fn run_step_no_source(step: &Step) -> Result<RunResult> {
        Ok(RunResult::new(PipelineEngine::execute_no_source(step)?))
    }
}

/// Stable SDK result wrapper around the internal pipeline context.
#[derive(Debug)]
pub struct RunResult {
    context: PipelineContext,
}

impl RunResult {
    fn new(context: PipelineContext) -> Self {
        Self { context }
    }

    /// Names of artifacts produced during execution, in insertion order.
    pub fn artifact_names(&self) -> &[String] {
        self.context.artifacts.keys()
    }

    /// Fetch a produced artifact by name.
    pub fn artifact(&self, name: &str) -> Option<&Artifact> {
        self.context.artifacts.get(name)
    }

    /// Fetch a produced artifact, returning an SDK error when absent.
    pub fn require_artifact(&self, name: &str) -> Result<&Artifact> {
        self.context.artifacts.require(name)
    }

    /// Dataset metadata captured during execution.
    pub fn metadata(&self) -> &DatasetMeta {
        &self.context.metadata
    }

    /// Execution metadata captured during execution.
    pub fn execution(&self) -> &ExecutionMeta {
        &self.context.execution
    }

    pub(crate) fn artifacts(&self) -> &sf_core::ArtifactStore {
        &self.context.artifacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_recipe_from_toml_text() {
        let recipe = StatsWorkbench::parse_recipe(
            r#"
[source]
type = "parquet"
path = "tests/fixtures/parquet/sales.parquet"

[[steps]]
type = "eda.summary"
cols = ["price"]
"#,
        )
        .unwrap();

        assert_eq!(recipe.steps.len(), 1);
    }
}
