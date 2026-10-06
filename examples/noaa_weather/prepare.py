#!/usr/bin/env python3
"""Convert NOAA GHCN-Daily CSV to Parquet for StatForge.

Usage:
    python3 prepare.py

Requires: pip install pandas pyarrow
"""

import os
import sys
import glob

try:
    import pandas as pd
except ImportError:
    print("Error: pandas is required. Install with: pip install pandas pyarrow")
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "data")
OUTPUT = os.path.join(DATA_DIR, "weather.parquet")

# Find downloaded CSV
csv_files = glob.glob(os.path.join(DATA_DIR, "*.csv"))
if not csv_files:
    print("Error: No CSV files found. Run ./download.sh first.")
    sys.exit(1)

source = csv_files[0]
print(f"Reading: {source}")

df = pd.read_csv(source, low_memory=False)
print(f"Raw rows: {len(df):,}")

# Select key weather columns
target_cols = ["DATE", "TMAX", "TMIN", "PRCP", "SNOW", "SNWD"]
available = [c for c in target_cols if c in df.columns]

if "DATE" not in available:
    print("Error: DATE column not found in data.")
    sys.exit(1)

df = df[available].copy()

# Convert date
df["DATE"] = pd.to_datetime(df["DATE"])

# Filter to recent years (2015+) for manageable size
df = df[df["DATE"] >= "2015-01-01"]

# Drop rows where all weather values are null
weather_cols = [c for c in available if c != "DATE"]
df = df.dropna(subset=weather_cols, how="all")

# Convert temperatures from tenths of °C to °C
for col in ["TMAX", "TMIN"]:
    if col in df.columns:
        df[col] = df[col] / 10.0

# Convert PRCP from tenths of mm to mm
if "PRCP" in df.columns:
    df["PRCP"] = df["PRCP"] / 10.0

df = df.reset_index(drop=True)
df.to_parquet(OUTPUT, index=False)
print(f"Written: {OUTPUT} ({len(df):,} rows, {len(df.columns)} cols)")
print("Done.")
