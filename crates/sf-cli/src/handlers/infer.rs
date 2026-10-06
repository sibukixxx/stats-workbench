use anyhow::Result;
use sf_core::{
    AlternativeSpec, ChiSqIndependenceStepSpec, MeanCiStepSpec, OutputFormat,
    ProportionTestStepSpec, SfError, StepHeader, StepSpec, TTestOneSampleStepSpec,
    TTestTwoSampleStepSpec, VarianceCiStepSpec, VarianceTestStepSpec,
};
use statforge::{RunResult, StatForge};

use crate::cli::{
    ChiSqGoodnessArgs, ChiSqIndependenceArgs, MeanCiArgs, PropCiArgs, PropTestArgs, TTest1Args,
    TTest2Args, VarCiArgs, VarTestArgs,
};
use crate::handlers::common::{execute_single_step, output_results};

const INFER_OUTPUT_FORMATS: &[OutputFormat] = &[OutputFormat::Json, OutputFormat::Parquet];

fn parse_alternative(s: &str) -> Result<AlternativeSpec> {
    match s.trim().to_ascii_lowercase().as_str() {
        "two_sided" | "two-sided" | "twosided" => Ok(AlternativeSpec::TwoSided),
        "less" => Ok(AlternativeSpec::Less),
        "greater" => Ok(AlternativeSpec::Greater),
        other => Err(SfError::Validation(format!(
            "Unknown alternative: '{}'. Use 'two_sided', 'less', or 'greater'.",
            other
        ))
        .into()),
    }
}

/// Execute a step that doesn't need a data source (proportion tests, chi-sq goodness).
fn execute_no_source(step: &StepSpec) -> Result<RunResult> {
    Ok(StatForge::run_step_no_source(step)?)
}

pub async fn handle_ttest1(args: TTest1Args) -> Result<()> {
    let alternative = parse_alternative(&args.alternative)?;
    let step = StepSpec::InferTTestOneSample(TTestOneSampleStepSpec {
        header: StepHeader::default(),
        col: args.col,
        mu0: args.mu0,
        confidence: args.confidence,
        alternative,
    });
    let output_name = step.output_name(0);
    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_ttest2(args: TTest2Args) -> Result<()> {
    let alternative = parse_alternative(&args.alternative)?;
    let step = StepSpec::InferTTestTwoSample(TTestTwoSampleStepSpec {
        header: StepHeader::default(),
        col: args.col,
        group_col: args.group_col,
        confidence: args.confidence,
        alternative,
    });
    let output_name = step.output_name(0);
    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_mean_ci(args: MeanCiArgs) -> Result<()> {
    let step = StepSpec::InferMeanCi(MeanCiStepSpec {
        header: StepHeader::default(),
        col: args.col,
        confidence: args.confidence,
    });
    let output_name = step.output_name(0);
    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub fn handle_prop_test(args: PropTestArgs) -> Result<()> {
    let alternative = parse_alternative(&args.alternative)?;
    let step = StepSpec::InferProportionTest(ProportionTestStepSpec {
        header: StepHeader::default(),
        n: args.n,
        successes: args.successes,
        p0: args.p0,
        confidence: args.confidence,
        alternative,
    });
    let output_name = step.output_name(0);
    let ctx = execute_no_source(&step)?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub fn handle_prop_ci(args: PropCiArgs) -> Result<()> {
    let step = StepSpec::InferProportionCi(sf_core::ProportionCiStepSpec {
        header: StepHeader::default(),
        n: args.n,
        successes: args.successes,
        confidence: args.confidence,
    });
    let output_name = step.output_name(0);
    let ctx = execute_no_source(&step)?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_var_test(args: VarTestArgs) -> Result<()> {
    let alternative = parse_alternative(&args.alternative)?;
    let step = StepSpec::InferVarianceTest(VarianceTestStepSpec {
        header: StepHeader::default(),
        col: args.col,
        sigma2_0: args.sigma2_0,
        confidence: args.confidence,
        alternative,
    });
    let output_name = step.output_name(0);
    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_var_ci(args: VarCiArgs) -> Result<()> {
    let step = StepSpec::InferVarianceCi(VarianceCiStepSpec {
        header: StepHeader::default(),
        col: args.col,
        confidence: args.confidence,
    });
    let output_name = step.output_name(0);
    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub fn handle_chisq_goodness(args: ChiSqGoodnessArgs) -> Result<()> {
    let expected = if args.expected.is_empty() {
        None
    } else {
        Some(args.expected)
    };

    let step = StepSpec::InferChiSqGoodness(sf_core::ChiSqGoodnessStepSpec {
        header: StepHeader::default(),
        observed: args.observed,
        expected,
    });
    let output_name = step.output_name(0);
    let ctx = execute_no_source(&step)?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_chisq_independence(args: ChiSqIndependenceArgs) -> Result<()> {
    let step = StepSpec::InferChiSqIndependence(ChiSqIndependenceStepSpec {
        header: StepHeader::default(),
        col1: args.col1,
        col2: args.col2,
    });
    let output_name = step.output_name(0);
    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        INFER_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}
