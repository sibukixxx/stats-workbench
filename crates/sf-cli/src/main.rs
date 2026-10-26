use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;
mod handlers;

use cli::{
    Cli, Command, EdaSubcommand, InferSubcommand, ModelSubcommand, StatsSubcommand, TsSubcommand,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Command::Eda(eda) => match eda.command {
            EdaSubcommand::Summary(args) => handlers::eda::handle_summary(args).await,
            EdaSubcommand::Corr(args) => handlers::eda::handle_corr(args).await,
            EdaSubcommand::Dist(args) => handlers::eda::handle_dist(args).await,
        },
        Command::Stats(stats) => match stats.command {
            StatsSubcommand::Outlier(args) => handlers::stats::handle_outlier(args).await,
        },
        Command::Model(model) => match model.command {
            ModelSubcommand::Regress(args) => handlers::model::handle_regress(args).await,
            ModelSubcommand::Diagnose(args) => handlers::model::handle_diagnose(args).await,
        },
        Command::Ts(ts) => match ts.command {
            TsSubcommand::Adf(args) => handlers::ts::handle_adf(args).await,
            TsSubcommand::Decompose(args) => handlers::ts::handle_decompose(args).await,
            TsSubcommand::Forecast(args) => handlers::ts::handle_forecast(args).await,
        },
        Command::Infer(infer) => match infer.command {
            InferSubcommand::Ttest1(args) => handlers::infer::handle_ttest1(args).await,
            InferSubcommand::Ttest2(args) => handlers::infer::handle_ttest2(args).await,
            InferSubcommand::MeanCi(args) => handlers::infer::handle_mean_ci(args).await,
            InferSubcommand::PropTest(args) => handlers::infer::handle_prop_test(args),
            InferSubcommand::PropCi(args) => handlers::infer::handle_prop_ci(args),
            InferSubcommand::VarTest(args) => handlers::infer::handle_var_test(args).await,
            InferSubcommand::VarCi(args) => handlers::infer::handle_var_ci(args).await,
            InferSubcommand::ChisqGoodness(args) => handlers::infer::handle_chisq_goodness(args),
            InferSubcommand::ChisqIndependence(args) => {
                handlers::infer::handle_chisq_independence(args).await
            }
        },
        Command::Run(args) => handlers::run::handle_run(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
