use anyhow::Result;
use sf_core::{DiagnoseStepSpec, OutputFormat, RegressStepSpec, StepHeader, StepSpec};

use crate::cli::{DiagnoseArgs, RegressArgs};
use crate::handlers::common::{execute_single_step, output_results};

const MODEL_OUTPUT_FORMATS: &[OutputFormat] = &[OutputFormat::Json];

pub async fn handle_regress(args: RegressArgs) -> Result<()> {
    let step = StepSpec::ModelRegress(RegressStepSpec {
        header: StepHeader::default(),
        target: args.target,
        predictors: args.predictors,
        confidence_level: args.confidence,
    });
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        MODEL_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_diagnose(args: DiagnoseArgs) -> Result<()> {
    let step = StepSpec::ModelDiagnose(DiagnoseStepSpec {
        header: StepHeader::default(),
        target: args.target,
        predictors: args.predictors,
    });
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        MODEL_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}
