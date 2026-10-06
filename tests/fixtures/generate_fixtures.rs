// This is a helper script to generate test fixtures.
// Run with: cargo run --example generate_fixtures
// (Or invoke directly)

use polars::prelude::*;
use std::fs;

fn main() {
    fs::create_dir_all("tests/fixtures/parquet").unwrap();

    let df = df! {
        "price" => &[10.0_f64, 20.0, 30.0, 40.0, 50.0, 15.0, 25.0, 35.0, 45.0, 55.0],
        "quantity" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 1.5, 2.5, 3.5, 4.5, 5.5],
        "revenue" => &[10.0_f64, 40.0, 90.0, 160.0, 250.0, 22.5, 62.5, 122.5, 202.5, 302.5],
        "category" => &["A", "B", "A", "B", "A", "B", "A", "B", "A", "B"],
    }
    .unwrap();

    let mut file = fs::File::create("tests/fixtures/parquet/sales.parquet").unwrap();
    ParquetWriter::new(&mut file).finish(&mut df.clone()).unwrap();
    println!("Generated tests/fixtures/parquet/sales.parquet");
}
