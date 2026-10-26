use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[allow(dead_code)]
pub fn fixture_path() -> String {
    workspace_root()
        .join("tests/fixtures/parquet/sales.parquet")
        .to_string_lossy()
        .to_string()
}

#[allow(dead_code)]
pub fn setup_fixture() {
    let root = workspace_root();
    let path = root.join("tests/fixtures/parquet/sales.parquet");
    if path.exists() {
        return;
    }

    use polars::prelude::*;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let df = df! {
        "price" => &[10.0_f64, 20.0, 30.0, 40.0, 50.0, 15.0, 25.0, 35.0, 45.0, 55.0],
        "quantity" => &[1.0_f64, 2.0, 3.0, 4.0, 5.0, 1.5, 2.5, 3.5, 4.5, 5.5],
        "revenue" => &[10.0_f64, 40.0, 90.0, 160.0, 250.0, 22.5, 62.5, 122.5, 202.5, 302.5],
        "category" => &["A", "B", "A", "B", "A", "B", "A", "B", "A", "B"],
    }
    .unwrap();

    // Publish only a complete fixture: integration tests run concurrently.
    let temporary = output_path("fixture", "parquet");
    std::fs::create_dir_all(temporary.parent().unwrap()).unwrap();
    let mut file = std::fs::File::create(&temporary).unwrap();
    ParquetWriter::new(&mut file)
        .finish(&mut df.clone())
        .unwrap();
    drop(file);
    std::fs::rename(temporary, path).unwrap();
}

#[allow(dead_code)]
pub fn setup_sqlite_fixture() -> String {
    let path = output_path("sales", "sqlite");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(&format!(
            "sqlite://{}",
            path.to_string_lossy()
        ))
        .unwrap()
        .create_if_missing(true);

        let pool = sqlx::sqlite::SqlitePool::connect_with(options)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE orders (
                id INTEGER PRIMARY KEY,
                price REAL NOT NULL,
                quantity REAL NOT NULL,
                revenue REAL NOT NULL,
                category TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let rows = [
            (1_i64, 10.0_f64, 1.0_f64, 10.0_f64, "A"),
            (2, 20.0, 2.0, 40.0, "B"),
            (3, 30.0, 3.0, 90.0, "A"),
            (4, 40.0, 4.0, 160.0, "B"),
            (5, 50.0, 5.0, 250.0, "A"),
            (6, 15.0, 1.5, 22.5, "B"),
            (7, 25.0, 2.5, 62.5, "A"),
            (8, 35.0, 3.5, 122.5, "B"),
            (9, 45.0, 4.5, 202.5, "A"),
            (10, 55.0, 5.5, 302.5, "B"),
        ];

        for (id, price, quantity, revenue, category) in rows {
            sqlx::query(
                "INSERT INTO orders (id, price, quantity, revenue, category)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(id)
            .bind(price)
            .bind(quantity)
            .bind(revenue)
            .bind(category)
            .execute(&pool)
            .await
            .unwrap();
        }

        pool.close().await;
    });

    format!("sqlite://{}", path.to_string_lossy())
}

/// Fixture with non-collinear predictors for regression tests.
#[allow(dead_code)]
pub fn regression_fixture_path() -> String {
    workspace_root()
        .join("tests/fixtures/parquet/regression.parquet")
        .to_string_lossy()
        .to_string()
}

#[allow(dead_code)]
pub fn setup_regression_fixture() {
    let root = workspace_root();
    let path = root.join("tests/fixtures/parquet/regression.parquet");
    if path.exists() {
        return;
    }

    use polars::prelude::*;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    // price and quantity are NOT perfectly collinear here
    let df = df! {
        "price" => &[10.0_f64, 20.0, 30.0, 40.0, 50.0, 15.0, 25.0, 35.0, 45.0, 55.0],
        "quantity" => &[5.0_f64, 3.0, 7.0, 2.0, 8.0, 4.0, 6.0, 1.0, 9.0, 3.0],
        "revenue" => &[55.0_f64, 65.0, 105.0, 85.0, 135.0, 60.0, 90.0, 72.0, 130.0, 120.0],
    }
    .unwrap();

    // Publish only a complete fixture: integration tests run concurrently.
    let temporary = output_path("fixture", "parquet");
    std::fs::create_dir_all(temporary.parent().unwrap()).unwrap();
    let mut file = std::fs::File::create(&temporary).unwrap();
    ParquetWriter::new(&mut file)
        .finish(&mut df.clone())
        .unwrap();
    drop(file);
    std::fs::rename(temporary, path).unwrap();
}

#[allow(dead_code)]
pub fn output_path(stem: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    workspace_root()
        .join("target")
        .join("test-output")
        .join(format!("{}-{}.{}", stem, nanos, extension))
}
