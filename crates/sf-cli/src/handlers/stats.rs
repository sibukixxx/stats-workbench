use anyhow::Result;
use statforge::{step, OutputFormat};

use crate::cli::OutlierArgs;
use crate::handlers::common::{execute_single_step, output_results};

const STATS_OUTPUT_FORMATS: &[OutputFormat] = &[OutputFormat::Json];

pub async fn handle_outlier(args: OutlierArgs) -> Result<()> {
    let step = step::outlier(args.cols);
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        STATS_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}
