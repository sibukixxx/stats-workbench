use stats_workbench::{output, source, step, StatsWorkbench};

#[tokio::main]
async fn main() -> stats_workbench::Result<()> {
    let source = source::parquet("tests/fixtures/parquet/sales.parquet");
    let step = step::summary(["price", "quantity", "revenue"]);

    let result = StatsWorkbench::run_step(&source, &step).await?;
    output::to_terminal(&result);

    Ok(())
}
