use anyhow::Result;
use stats_workbench::{step, CorrMethod, OutputFormat};

use crate::cli::{EdaCorrArgs, EdaDistArgs, EdaSummaryArgs};
use crate::handlers::common::{execute_single_step, output_results};

const EDA_OUTPUT_FORMATS: &[OutputFormat] = &[OutputFormat::Json, OutputFormat::Parquet];

pub async fn handle_summary(args: EdaSummaryArgs) -> Result<()> {
    let step = step::summary(args.cols);
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        EDA_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_corr(args: EdaCorrArgs) -> Result<()> {
    let step = step::correlation(args.cols, args.method.parse::<CorrMethod>()?);
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        EDA_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_dist(args: EdaDistArgs) -> Result<()> {
    let step = step::distribution(args.col, args.bins);
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        EDA_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}
