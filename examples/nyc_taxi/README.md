# Example: NYC TLC Trip Record Data

## Dataset

- **Source**: [NYC Taxi and Limousine Commission](https://www.nyc.gov/site/tlc/about/tlc-trip-record-data.page)
- **License**: Public domain (NYC Open Data)
- **Format**: Parquet (native — no CSV conversion needed)
- **Granularity**: Per-trip records

### Columns used

| Column              | Description                    |
|---------------------|--------------------------------|
| `trip_distance`     | Trip distance in miles         |
| `fare_amount`       | Fare amount in USD             |
| `tip_amount`        | Tip amount in USD              |
| `total_amount`      | Total charge in USD            |
| `passenger_count`   | Number of passengers           |
| `tpep_pickup_datetime` | Pickup timestamp            |

## What this example demonstrates

- **Large Parquet handling**: Stats Workbench reads NYC taxi Parquet directly — no conversion step
- **EDA summary**: Fare, distance, tip distributions
- **Distribution analysis**: Histogram of fare amounts
- **Outlier detection**: Identify anomalous fares and distances

## Prepare data

```bash
cd examples/nyc_taxi
./download.sh
python3 prepare.py
```

The download script fetches one month of Yellow Taxi data (~40MB Parquet).
The prepare script extracts a 100K-row subset for fast iteration.

## Run

```bash
# Summary of fare and trip features
sf run --recipe examples/nyc_taxi/eda_fare_trip.toml

# Or directly:
sf eda summary --source examples/nyc_taxi/data/yellow_tripdata_sample.parquet \
  --cols trip_distance,fare_amount,tip_amount,total_amount

sf eda dist --source examples/nyc_taxi/data/yellow_tripdata_sample.parquet \
  --col fare_amount --bins 20

sf stats outlier --source examples/nyc_taxi/data/yellow_tripdata_sample.parquet \
  --cols fare_amount,trip_distance
```

## Expected insight

- Most trips are short (< 5 miles) with fares under $20
- Tip amounts cluster around 15-20% of fare
- Some extreme outliers exist (very long trips, very high fares)
- Trip distance and fare amount are strongly correlated
