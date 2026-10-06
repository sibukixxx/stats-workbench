// サンプルデータ生成スクリプト
//
// Usage:
//   cargo run --example generate_sample_data
//
// examples/ 配下のレシピで使うサンプル Parquet を生成します。
// 外部データのダウンロード不要で、すぐに StatForge を試せます。

use polars::prelude::*;
use std::fs;

fn main() {
    fs::create_dir_all("examples/data").unwrap();

    generate_sales();
    generate_employees();
    generate_sensors();

    println!("Done. Generated files in examples/data/");
}

/// 売上データ (50行)
/// columns: date, product, category, price, quantity, revenue, discount, region
fn generate_sales() {
    let n = 50;
    let products = ["Widget A", "Widget B", "Gadget X", "Gadget Y", "Service Z"];
    let categories = [
        "Hardware",
        "Hardware",
        "Electronics",
        "Electronics",
        "Service",
    ];
    let regions = ["East", "West", "North", "South"];

    let date: Vec<String> = (0..n)
        .map(|i| format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1))
        .collect();
    let product: Vec<&str> = (0..n).map(|i| products[i % products.len()]).collect();
    let category: Vec<&str> = (0..n).map(|i| categories[i % categories.len()]).collect();
    let price: Vec<f64> = (0..n)
        .map(|i| 10.0 + (i as f64) * 3.5 + ((i * 7) % 13) as f64)
        .collect();
    let quantity: Vec<f64> = (0..n).map(|i| 1.0 + ((i * 3 + 2) % 20) as f64).collect();
    let revenue: Vec<f64> = (0..n).map(|i| price[i] * quantity[i]).collect();
    let discount: Vec<f64> = (0..n)
        .map(|i| {
            if i % 5 == 0 {
                0.1
            } else if i % 3 == 0 {
                0.05
            } else {
                0.0
            }
        })
        .collect();
    let region: Vec<&str> = (0..n).map(|i| regions[i % regions.len()]).collect();

    let mut df = df! {
        "date" => &date,
        "product" => &product,
        "category" => &category,
        "price" => &price,
        "quantity" => &quantity,
        "revenue" => &revenue,
        "discount" => &discount,
        "region" => &region,
    }
    .unwrap();

    let mut file = fs::File::create("examples/data/sales.parquet").unwrap();
    ParquetWriter::new(&mut file).finish(&mut df).unwrap();
    println!("  examples/data/sales.parquet (50 rows)");
}

/// 従業員データ (30行)
/// columns: emp_id, department, years_exp, salary, performance_score, projects_completed
fn generate_employees() {
    let n = 30;
    let departments = ["Engineering", "Sales", "Marketing", "Operations", "HR"];

    let emp_id: Vec<String> = (0..n).map(|i| format!("EMP-{:03}", i + 1)).collect();
    let department: Vec<&str> = (0..n).map(|i| departments[i % departments.len()]).collect();
    let years_exp: Vec<f64> = (0..n).map(|i| 1.0 + ((i * 7 + 3) % 25) as f64).collect();
    let salary: Vec<f64> = (0..n)
        .map(|i| 40000.0 + years_exp[i] * 3000.0 + ((i * 13) % 17) as f64 * 1000.0)
        .collect();
    let performance_score: Vec<f64> = (0..n)
        .map(|i| 2.0 + ((i * 11 + 5) % 30) as f64 / 10.0)
        .collect();
    let projects_completed: Vec<f64> = (0..n).map(|i| ((i * 3 + 1) % 15) as f64).collect();

    let mut df = df! {
        "emp_id" => &emp_id,
        "department" => &department,
        "years_exp" => &years_exp,
        "salary" => &salary,
        "performance_score" => &performance_score,
        "projects_completed" => &projects_completed,
    }
    .unwrap();

    let mut file = fs::File::create("examples/data/employees.parquet").unwrap();
    ParquetWriter::new(&mut file).finish(&mut df).unwrap();
    println!("  examples/data/employees.parquet (30 rows)");
}

/// IoTセンサーデータ (100行)
/// columns: timestamp, sensor_id, temperature, humidity, pressure, battery_level
fn generate_sensors() {
    let n = 100;
    let sensor_ids = [
        "sensor-01",
        "sensor-02",
        "sensor-03",
        "sensor-04",
        "sensor-05",
    ];

    let timestamp: Vec<String> = (0..n)
        .map(|i| {
            format!(
                "2024-06-{:02}T{:02}:{:02}:00Z",
                (i / 24) + 1,
                i % 24,
                (i * 15) % 60
            )
        })
        .collect();
    let sensor_id: Vec<&str> = (0..n).map(|i| sensor_ids[i % sensor_ids.len()]).collect();
    let temperature: Vec<f64> = (0..n)
        .map(|i| 18.0 + (i as f64 * 0.3).sin() * 8.0 + ((i * 7) % 5) as f64)
        .collect();
    let humidity: Vec<f64> = (0..n)
        .map(|i| 40.0 + (i as f64 * 0.2).cos() * 20.0 + ((i * 3) % 10) as f64)
        .collect();
    let pressure: Vec<f64> = (0..n)
        .map(|i| 1010.0 + (i as f64 * 0.1).sin() * 15.0)
        .collect();
    let battery_level: Vec<f64> = (0..n)
        .map(|i| 100.0 - (i as f64) * 0.5 + ((i * 11) % 7) as f64)
        .collect();

    let mut df = df! {
        "timestamp" => &timestamp,
        "sensor_id" => &sensor_id,
        "temperature" => &temperature,
        "humidity" => &humidity,
        "pressure" => &pressure,
        "battery_level" => &battery_level,
    }
    .unwrap();

    let mut file = fs::File::create("examples/data/sensors.parquet").unwrap();
    ParquetWriter::new(&mut file).finish(&mut df).unwrap();
    println!("  examples/data/sensors.parquet (100 rows)");
}
