# Example: NOAA Daily Climate Data

## Dataset

- **Source**: [NOAA Climate Data Online (CDO)](https://www.ncei.noaa.gov/cdo-web/) / [GHCN-Daily](https://www.ncei.noaa.gov/products/land-based-station/global-historical-climatology-network-daily)
- **License**: Public domain (US Government)
- **Granularity**: Daily weather observations
- **Station**: Configurable (default: JFK Airport, NYC)

### Columns used

| Column  | Description                        |
|---------|------------------------------------|
| `DATE`  | Observation date                   |
| `TMAX`  | Maximum temperature (tenths of °C) |
| `TMIN`  | Minimum temperature (tenths of °C) |
| `PRCP`  | Precipitation (tenths of mm)       |
| `SNOW`  | Snowfall (mm)                      |
| `SNWD`  | Snow depth (mm)                    |

## What this example demonstrates

- **Time series decomposition** (Phase 2): Daily temperature seasonal patterns
- **Correlation analysis**: Temperature vs precipitation relationships
- **ADF test** (Phase 2): Stationarity of temperature series

## Prepare data

```bash
cd examples/noaa_weather
./download.sh
python3 prepare.py
```

Note: NOAA CDO requires a free API token. The download script uses the bulk
data endpoint which does not require authentication.

## Run

```bash
# Temperature-precipitation correlation
stats-workbench run --recipe examples/noaa_weather/corr_temp_precip.toml

# Or directly:
stats-workbench eda summary --source examples/noaa_weather/data/weather.parquet \
  --cols TMAX,TMIN,PRCP

stats-workbench eda corr --source examples/noaa_weather/data/weather.parquet \
  --cols TMAX,TMIN,PRCP --method pearson
```

## Expected insight

- Daily temperature shows strong seasonal patterns (warm summer, cold winter)
- TMAX and TMIN are highly correlated
- Precipitation has weaker seasonal structure than temperature
- Temperature series is non-stationary (ADF test in Phase 2)
