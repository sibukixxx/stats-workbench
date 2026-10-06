use sf_core::{Artifact, DatasetRef, Recipe, SfError, SfResult};
use sf_data::SourceLoader;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::context::PipelineContext;
use crate::env::expand_env_vars;
use crate::executor::StepExecutor;

/// Orchestrates recipe execution.
///
/// Responsibilities:
/// 1. Validate recipe
/// 2. Load source
/// 3. Seed artifact store
/// 4. Execute steps
/// 5. Return context for output handling
pub struct PipelineEngine;

impl PipelineEngine {
    /// Execute a full recipe pipeline.
    pub async fn execute(recipe: &Recipe) -> SfResult<PipelineContext> {
        Self::validate_recipe(recipe)?;
        let mut ctx = Self::prepare_context(&recipe.source).await?;

        // 2. Execute steps
        for (i, step) in recipe.steps.iter().enumerate() {
            let step_name = step.kind_name();
            tracing::info!("Executing step {}: {}", i + 1, step_name);
            let start = Instant::now();

            StepExecutor::execute(&mut ctx, step, i).map_err(|e| {
                SfError::Execution(format!("Step {} ({}) failed: {}", i + 1, step_name, e))
            })?;

            ctx.execution.steps_executed += 1;
            tracing::info!("Step {} completed in {:?}", i + 1, start.elapsed());
        }

        Ok(ctx)
    }

    /// Execute a single step (for CLI ad-hoc commands).
    pub async fn execute_single(
        source: &sf_core::SourceSpec,
        step: &sf_core::StepSpec,
    ) -> SfResult<PipelineContext> {
        source.validate()?;
        step.validate(1)?;

        let mut ctx = Self::prepare_context(source).await?;

        StepExecutor::execute(&mut ctx, step, 0)?;
        ctx.execution.steps_executed = 1;

        Ok(ctx)
    }

    /// Execute a single step that doesn't require a data source.
    ///
    /// Used for steps like proportion tests or chi-squared goodness-of-fit
    /// that take pre-computed summary statistics instead of raw data.
    pub fn execute_no_source(step: &sf_core::StepSpec) -> SfResult<PipelineContext> {
        step.validate(1)?;
        let mut ctx = PipelineContext::new();
        ctx.execution.started_at = chrono::Utc::now().to_rfc3339();

        StepExecutor::execute(&mut ctx, step, 0)?;
        ctx.execution.steps_executed = 1;

        Ok(ctx)
    }

    /// Parse a recipe TOML string with env var expansion.
    ///
    /// Output paths are validated against the current working directory only;
    /// use [`parse_recipe_in`](Self::parse_recipe_in) when the recipe came from
    /// a file so outputs next to the recipe are accepted too.
    pub fn parse_recipe(content: &str) -> SfResult<Recipe> {
        Self::parse_recipe_in(content, None)
    }

    /// Parse a recipe TOML string that was read from a file in `recipe_dir`.
    pub fn parse_recipe_in(content: &str, recipe_dir: Option<&Path>) -> SfResult<Recipe> {
        let expanded = expand_env_vars(content)?;
        let recipe: Recipe = toml::from_str(&expanded)?;
        Self::validate_recipe(&recipe)?;
        Self::validate_output_paths(&recipe, recipe_dir)?;
        Ok(recipe)
    }

    /// Validate recipe before execution.
    fn validate_recipe(recipe: &Recipe) -> SfResult<()> {
        if recipe.steps.is_empty() {
            return Err(SfError::Validation("Recipe has no steps".into()));
        }

        recipe.source.validate()?;

        // Validate individual steps
        for (i, step) in recipe.steps.iter().enumerate() {
            step.validate(i + 1)?;
        }

        // Validate output artifacts don't contain empty strings
        if let Some(ref output) = recipe.output {
            for art in &output.artifacts {
                if art.trim().is_empty() {
                    return Err(SfError::Validation(
                        "Output artifacts list contains empty string".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Output-path checks apply to recipes parsed from text/files (the only
    /// untrusted input); programmatic `Recipe` values built through the SDK
    /// are the caller's own.
    fn validate_output_paths(recipe: &Recipe, recipe_dir: Option<&Path>) -> SfResult<()> {
        if let Some(ref output) = recipe.output {
            for p in [output.path.as_deref(), output.report.as_deref()]
                .into_iter()
                .flatten()
            {
                Self::validate_output_path(p, recipe_dir)?;
            }
        }
        Ok(())
    }

    /// Recipe output paths must stay inside the working directory or the
    /// directory the recipe was loaded from. `..` components are never
    /// allowed, and an absolute path is accepted only when it lies under one
    /// of those two roots. A shared recipe must not be able to overwrite
    /// `~/.bashrc` or `.git/config`. The CLI `--out` flag is not subject to
    /// this check because the operator typed it themselves.
    fn validate_output_path(path: &str, recipe_dir: Option<&Path>) -> SfResult<()> {
        use std::path::Component;

        let p = Path::new(path);
        if p.components().any(|c| matches!(c, Component::ParentDir | Component::Prefix(_))) {
            return Err(SfError::Validation(format!(
                "Recipe output path '{}' must not contain '..'",
                path
            )));
        }
        if !p.is_absolute() {
            return Ok(());
        }

        let mut roots: Vec<PathBuf> = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            roots.push(cwd);
        }
        if let Some(dir) = recipe_dir {
            roots.push(dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf()));
        }
        if roots.iter().any(|root| p.starts_with(root)) {
            return Ok(());
        }
        Err(SfError::Validation(format!(
            "Recipe output path '{}' must be inside the working directory or the recipe's directory",
            path
        )))
    }

    /// Strip the password from a URL-style location before storing it in
    /// metadata, so serialised `DatasetMeta` never carries DB credentials.
    pub fn redact_location(location: &str) -> String {
        let Some(scheme_end) = location.find("://") else {
            return location.to_string();
        };
        let rest = &location[scheme_end + 3..];
        let authority_end = rest.find('/').unwrap_or(rest.len());
        let authority = &rest[..authority_end];
        let Some(at) = authority.rfind('@') else {
            return location.to_string();
        };
        let userinfo = &authority[..at];
        let redacted = match userinfo.split_once(':') {
            Some((user, _)) => format!("{}:***", user),
            None => userinfo.to_string(),
        };
        format!(
            "{}://{}@{}{}",
            &location[..scheme_end],
            redacted,
            &authority[at + 1..],
            &rest[authority_end..]
        )
    }

    async fn prepare_context(source: &sf_core::SourceSpec) -> SfResult<PipelineContext> {
        let mut ctx = PipelineContext::new();
        ctx.execution.started_at = chrono::Utc::now().to_rfc3339();
        Self::load_source(&mut ctx, source).await?;
        Ok(ctx)
    }

    async fn load_source(ctx: &mut PipelineContext, source: &sf_core::SourceSpec) -> SfResult<()> {
        tracing::info!("Loading source...");
        let dataset = SourceLoader::load(source).await?;

        ctx.metadata.source_type = source.kind_name().to_string();
        ctx.metadata.source_path = Self::redact_location(source.location());

        match dataset.collect() {
            Ok(df) => {
                ctx.metadata.row_count = Some(df.height() as u64);
                ctx.metadata.col_count = Some(df.width() as u64);
                ctx.metadata.columns = df
                    .get_column_names()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect();
                ctx.artifacts
                    .insert("source", Artifact::Dataset(DatasetRef::Materialized(df)));
            }
            Err(_) => {
                ctx.artifacts.insert("source", Artifact::Dataset(dataset));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_path_rejects_parent_dir_and_absolute_outside() {
        assert!(PipelineEngine::validate_output_path("../x.json", None).is_err());
        assert!(PipelineEngine::validate_output_path("/etc/passwd", None).is_err());
        assert!(PipelineEngine::validate_output_path("output/result.json", None).is_ok());
        let cwd = std::env::current_dir().unwrap();
        let inside = cwd.join("out.json");
        assert!(PipelineEngine::validate_output_path(inside.to_str().unwrap(), None).is_ok());
    }

    #[test]
    fn test_redact_location_hides_password() {
        assert_eq!(
            PipelineEngine::redact_location("postgresql://u:secret@db:5432/app"),
            "postgresql://u:***@db:5432/app"
        );
        assert_eq!(
            PipelineEngine::redact_location("postgresql://u@db/app"),
            "postgresql://u@db/app"
        );
        assert_eq!(PipelineEngine::redact_location("data/x.parquet"), "data/x.parquet");
    }

    #[test]
    fn test_parse_recipe_summary() {
        let text = r#"
[source]
type = "parquet"
path = "sales.parquet"

[[steps]]
type = "eda.summary"
cols = ["price"]
"#;

        let recipe = PipelineEngine::parse_recipe(text).unwrap();
        assert_eq!(recipe.steps.len(), 1);
        assert!(matches!(recipe.steps[0], sf_core::StepSpec::EdaSummary(_)));
    }

    #[test]
    fn test_parse_recipe_full() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
id = "summary_main"
type = "eda.summary"
cols = ["price", "qty"]

[[steps]]
type = "eda.correlation"
cols = ["price", "qty"]
method = "spearman"

[[steps]]
type = "eda.dist"
col = "price"
bins = 20

[[steps]]
type = "stats.outlier_detect"
cols = ["price"]
method = "iqr"
action = "flag"

[output]
terminal = true
format = "json"
path = "output/result.json"
artifacts = ["summary_main"]
"#;

        let recipe = PipelineEngine::parse_recipe(text).unwrap();
        assert_eq!(recipe.steps.len(), 4);
    }

    #[test]
    fn test_parse_recipe_no_steps() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"
"#;

        let result = PipelineEngine::parse_recipe(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_recipe_invalid_toml() {
        let result = PipelineEngine::parse_recipe("not valid {{{ toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_recipe_ts_adf() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.adf"
time_col = "date"
value_col = "price"
max_lag = 4
"#;

        let recipe = PipelineEngine::parse_recipe(text).unwrap();
        assert_eq!(recipe.steps.len(), 1);
        assert!(matches!(recipe.steps[0], sf_core::StepSpec::TsAdf(_)));
    }

    #[test]
    fn test_parse_recipe_ts_decompose() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.decompose"
time_col = "date"
value_col = "price"
period = 12
"#;

        let recipe = PipelineEngine::parse_recipe(text).unwrap();
        assert_eq!(recipe.steps.len(), 1);
        assert!(matches!(
            recipe.steps[0],
            sf_core::StepSpec::TsDecompose(_)
        ));
    }

    #[test]
    fn test_parse_recipe_ts_forecast() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.forecast"
time_col = "date"
value_col = "price"
order = [1, 1, 1]
horizon = 10
confidence = 0.95
"#;

        let recipe = PipelineEngine::parse_recipe(text).unwrap();
        assert_eq!(recipe.steps.len(), 1);
        assert!(matches!(
            recipe.steps[0],
            sf_core::StepSpec::TsForecast(_)
        ));
    }

    #[test]
    fn test_parse_recipe_ts_adf_validation_empty_time_col() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.adf"
time_col = ""
value_col = "price"
"#;

        let result = PipelineEngine::parse_recipe(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_recipe_ts_decompose_zero_period() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.decompose"
time_col = "date"
value_col = "price"
period = 0
"#;

        let result = PipelineEngine::parse_recipe(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_recipe_ts_forecast_zero_horizon() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.forecast"
time_col = "date"
value_col = "price"
order = [1, 1, 0]
horizon = 0
"#;

        let result = PipelineEngine::parse_recipe(text);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_recipe_ts_mixed_pipeline() {
        let text = r#"
[source]
type = "parquet"
path = "data.parquet"

[[steps]]
type = "ts.adf"
time_col = "date"
value_col = "price"

[[steps]]
type = "ts.decompose"
time_col = "date"
value_col = "price"
period = 12

[[steps]]
type = "ts.forecast"
time_col = "date"
value_col = "price"
order = [1, 1, 1]
horizon = 5
"#;

        let recipe = PipelineEngine::parse_recipe(text).unwrap();
        assert_eq!(recipe.steps.len(), 3);
    }

    #[test]
    fn test_parse_recipe_db_no_table() {
        let text = r#"
[source]
type = "sqlite"
path = "test.db"

[[steps]]
type = "eda.summary"
cols = ["price"]
"#;

        let result = PipelineEngine::parse_recipe(text);
        assert!(result.is_err());
    }
}
