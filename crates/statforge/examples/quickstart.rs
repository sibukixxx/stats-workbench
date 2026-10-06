use statforge::{output, source, step, StatForge};

#[tokio::main]
async fn main() -> statforge::Result<()> {
    let source = source::parquet("tests/fixtures/parquet/sales.parquet");
    let step = step::summary(["price", "quantity", "revenue"]);

    let result = StatForge::run_step(&source, &step).await?;
    output::to_terminal(&result);

    Ok(())
}
