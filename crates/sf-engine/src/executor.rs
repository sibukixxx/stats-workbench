use sf_core::{AlternativeSpec, Artifact, SfError, SfResult, StepSpec};
use sf_stats::eda;
use sf_stats::infer;
use sf_stats::model;
use sf_stats::timeseries;

use crate::context::PipelineContext;

fn to_alternative(spec: &AlternativeSpec) -> infer::ttest::Alternative {
    match spec {
        AlternativeSpec::TwoSided => infer::ttest::Alternative::TwoSided,
        AlternativeSpec::Less => infer::ttest::Alternative::Less,
        AlternativeSpec::Greater => infer::ttest::Alternative::Greater,
    }
}

/// Executes individual steps against the pipeline context.
pub struct StepExecutor;

impl StepExecutor {
    pub fn execute(ctx: &mut PipelineContext, step: &StepSpec, step_index: usize) -> SfResult<()> {
        let name = step.output_name(step_index);

        match step {
            // Steps that require a DataFrame input
            StepSpec::EdaSummary(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = eda::summary::compute_summary(&df, &spec.cols)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::EdaCorrelation(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result =
                    eda::correlation::compute_correlation(&df, &spec.cols, &spec.method)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::EdaDist(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = eda::distribution::compute_distribution(&df, &spec.col, spec.bins)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::OutlierDetect(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = eda::outlier::compute_outlier_detection(&df, &spec.cols)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::TsAdf(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = timeseries::adf::compute_adf(
                    &df,
                    &spec.time_col,
                    &spec.value_col,
                    spec.max_lag,
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::TsDecompose(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = timeseries::decompose::compute_decompose(
                    &df,
                    &spec.time_col,
                    &spec.value_col,
                    spec.period,
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::TsForecast(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = timeseries::arima::compute_arima_forecast(
                    &df,
                    &spec.time_col,
                    &spec.value_col,
                    spec.order,
                    spec.horizon,
                    spec.confidence,
                )?;
                ctx.artifacts.insert(name, Artifact::Forecast(result));
            }
            StepSpec::ModelRegress(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = model::regression::compute_regression(
                    &df,
                    &spec.target,
                    &spec.predictors,
                    spec.confidence_level,
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::ModelDiagnose(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = model::diagnostics::compute_regression_diagnostics(
                    &df,
                    &spec.target,
                    &spec.predictors,
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }

            // Infer steps with DataFrame input
            StepSpec::InferTTestOneSample(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = infer::ttest::compute_ttest_one_sample(
                    &df,
                    &spec.col,
                    spec.mu0,
                    spec.confidence,
                    to_alternative(&spec.alternative),
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferTTestTwoSample(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = infer::ttest::compute_ttest_two_sample(
                    &df,
                    &spec.col,
                    &spec.group_col,
                    spec.confidence,
                    to_alternative(&spec.alternative),
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferMeanCi(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = infer::ci::compute_mean_ci(&df, &spec.col, spec.confidence)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferVarianceTest(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result = infer::variance::compute_variance_test(
                    &df,
                    &spec.col,
                    spec.sigma2_0,
                    spec.confidence,
                    to_alternative(&spec.alternative),
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferVarianceCi(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result =
                    infer::ci::compute_variance_ci(&df, &spec.col, spec.confidence)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferChiSqIndependence(spec) => {
                let df = resolve_input_df(ctx, step.header().input.as_deref())?;
                let result =
                    infer::chi_square::compute_chisq_independence(&df, &spec.col1, &spec.col2)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }

            // Infer steps without DataFrame input (summary statistics provided directly)
            StepSpec::InferProportionTest(spec) => {
                let result = infer::proportion::compute_proportion_test(
                    spec.n,
                    spec.successes,
                    spec.p0,
                    spec.confidence,
                    to_alternative(&spec.alternative),
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferProportionCi(spec) => {
                let result =
                    infer::ci::compute_proportion_ci(spec.n, spec.successes, spec.confidence)?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
            StepSpec::InferChiSqGoodness(spec) => {
                let result = infer::chi_square::compute_chisq_goodness(
                    &spec.observed,
                    spec.expected.as_deref(),
                )?;
                ctx.artifacts.insert(name, Artifact::Stats(result));
            }
        }

        Ok(())
    }
}

/// Resolve the input DataFrame for a step.
///
/// If `input` is specified, look up that artifact. Otherwise use `ctx.current`.
fn resolve_input_df(
    ctx: &PipelineContext,
    input: Option<&str>,
) -> SfResult<polars::prelude::DataFrame> {
    let key = input.unwrap_or(&ctx.current);
    let artifact = ctx.artifacts.require(key)?;

    match artifact {
        Artifact::Dataset(ds) => ds
            .collect()
            .map_err(|e| SfError::Execution(format!("Failed to collect dataset '{}': {}", key, e))),
        Artifact::Table(df) => Ok(df.clone()),
        _ => Err(SfError::Execution(format!(
            "Artifact '{}' is not a dataset",
            key
        ))),
    }
}
