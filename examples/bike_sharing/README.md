# Example: Bike Sharing (UCI)

## Dataset

- **Source**: [UCI Machine Learning Repository — Bike Sharing Dataset](https://archive.ics.uci.edu/dataset/275/bike+sharing+dataset)
- **License**: Creative Commons Attribution 4.0 International (CC BY 4.0)
- **Period**: 2011–2012, Capital Bikeshare (Washington D.C.)
- **Granularity**: Daily (`day.csv`) and hourly (`hour.csv`)

### Columns used

| Column      | Description                          |
|-------------|--------------------------------------|
| `dteday`    | Date                                 |
| `season`    | Season (1=spring, 2=summer, ...)     |
| `temp`      | Normalized temperature               |
| `hum`       | Normalized humidity                  |
| `windspeed` | Normalized wind speed                |
| `cnt`       | Total rental count                   |
| `casual`    | Casual user count                    |
| `registered`| Registered user count                |
| `weathersit`| Weather situation (1=clear, 4=heavy) |

## What this example demonstrates

- **EDA summary**: Basic statistics of weather and rental features
- **Correlation analysis**: Relationship between weather conditions and demand
- **Outlier detection**: Identify unusual rental days
- **Time series** (Phase 2): Daily count decomposition

## Prepare data

```bash
cd examples/bike_sharing
./download.sh
python3 prepare.py
```

## Run

```bash
# Summary statistics
stats-workbench run --recipe examples/bike_sharing/eda_summary.toml

# Correlation: weather vs demand
stats-workbench run --recipe examples/bike_sharing/corr_weather.toml

# Or run directly:
stats-workbench eda summary --source examples/bike_sharing/data/day.parquet --cols temp,hum,windspeed,cnt
stats-workbench eda corr --source examples/bike_sharing/data/day.parquet --cols temp,hum,windspeed,cnt --method pearson
```

## Expected insight

- Temperature is positively correlated with rental demand
- Humidity and wind speed may reduce demand
- Daily counts show clear seasonal structure (higher in summer)
- Some days have unusually high or low demand (holidays, storms)
