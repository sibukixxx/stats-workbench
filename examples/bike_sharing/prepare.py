#!/usr/bin/env python3
"""Convert Bike Sharing CSV to Parquet for StatForge.

Usage:
    python3 prepare.py

Requires: pip install pandas pyarrow
"""

import os
import sys

try:
    import pandas as pd
except ImportError:
    print("Error: pandas is required. Install with: pip install pandas pyarrow")
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "data")

# Daily data
day_csv = os.path.join(DATA_DIR, "day.csv")
day_parquet = os.path.join(DATA_DIR, "day.parquet")

if not os.path.exists(day_csv):
    print(f"Error: {day_csv} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting day.csv to day.parquet...")
df = pd.read_csv(day_csv)

# Select and rename key columns for clarity
cols = ["dteday", "season", "yr", "mnth", "holiday", "weekday", "workingday",
        "weathersit", "temp", "atemp", "hum", "windspeed", "casual", "registered", "cnt"]
df = df[cols]

# Convert date
df["dteday"] = pd.to_datetime(df["dteday"])

df.to_parquet(day_parquet, index=False)
print(f"Written: {day_parquet} ({len(df)} rows, {len(df.columns)} cols)")

# Hourly data
hour_csv = os.path.join(DATA_DIR, "hour.csv")
hour_parquet = os.path.join(DATA_DIR, "hour.parquet")

if os.path.exists(hour_csv):
    print("Converting hour.csv to hour.parquet...")
    dfh = pd.read_csv(hour_csv)
    cols_h = ["dteday", "season", "yr", "mnth", "hr", "holiday", "weekday",
              "workingday", "weathersit", "temp", "atemp", "hum", "windspeed",
              "casual", "registered", "cnt"]
    dfh = dfh[cols_h]
    dfh["dteday"] = pd.to_datetime(dfh["dteday"])
    dfh.to_parquet(hour_parquet, index=False)
    print(f"Written: {hour_parquet} ({len(dfh)} rows, {len(dfh.columns)} cols)")

print("Done.")
