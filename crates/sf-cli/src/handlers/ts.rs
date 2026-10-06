use anyhow::Result;
use sf_core::{
    AdfStepSpec, DecomposeStepSpec, ForecastStepSpec, OutputFormat, StepHeader, StepSpec,
};

use crate::cli::{TsAdfArgs, TsDecomposeArgs, TsForecastArgs};
use crate::handlers::common::{execute_single_step, output_results};

const TS_OUTPUT_FORMATS: &[OutputFormat] = &[OutputFormat::Json];

pub async fn handle_adf(args: TsAdfArgs) -> Result<()> {
    let step = StepSpec::TsAdf(AdfStepSpec {
        header: StepHeader::default(),
        time_col: args.time_col,
        value_col: args.value_col,
        max_lag: args.max_lag,
    });
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        TS_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_decompose(args: TsDecomposeArgs) -> Result<()> {
    let step = StepSpec::TsDecompose(DecomposeStepSpec {
        header: StepHeader::default(),
        time_col: args.time_col,
        value_col: args.value_col,
        period: args.period,
    });
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        TS_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}

pub async fn handle_forecast(args: TsForecastArgs) -> Result<()> {
    let order: [usize; 3] = if args.order.len() == 3 {
        [args.order[0], args.order[1], args.order[2]]
    } else {
        return Err(anyhow::anyhow!(
            "Order must be exactly 3 values (p,d,q), got {}",
            args.order.len()
        ));
    };

    let step = StepSpec::TsForecast(ForecastStepSpec {
        header: StepHeader::default(),
        time_col: args.time_col,
        value_col: args.value_col,
        order,
        horizon: args.horizon,
        confidence: args.confidence,
    });
    let output_name = step.output_name(0);

    let ctx = execute_single_step(&args.source, step).await?;
    output_results(
        &ctx,
        &args.output,
        TS_OUTPUT_FORMATS,
        std::slice::from_ref(&output_name),
    )
}
