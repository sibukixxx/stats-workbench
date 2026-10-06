#!/usr/bin/env python3
"""Extract a sample subset from NYC Taxi Parquet for StatForge examples.

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
SAMPLE_SIZE = 100_000
OUTPUT = os.path.join(DATA_DIR, "yellow_tripdata_sample.parquet")

# Find the downloaded file
parquet_files = glob.glob(os.path.join(DATA_DIR, "yellow_tripdata_*.parquet"))
parquet_files = [f for f in parquet_files if "sample" not in f]

if not parquet_files:
    print("Error: No trip data found. Run ./download.sh first.")
    sys.exit(1)

source = parquet_files[0]
print(f"Reading: {source}")

# Read and sample
df = pd.read_parquet(source)
print(f"Full dataset: {len(df):,} rows")

# Select key columns
cols = ["tpep_pickup_datetime", "tpep_dropoff_datetime",
        "passenger_count", "trip_distance",
        "fare_amount", "tip_amount", "total_amount"]
available = [c for c in cols if c in df.columns]
df = df[available]

# Filter out obvious bad data
df = df[df["fare_amount"] > 0]
df = df[df["trip_distance"] > 0]
df = df[df["total_amount"] > 0]

# Sample
if len(df) > SAMPLE_SIZE:
    df = df.sample(n=SAMPLE_SIZE, random_state=42)

df = df.reset_index(drop=True)
df.to_parquet(OUTPUT, index=False)
print(f"Written: {OUTPUT} ({len(df):,} rows, {len(df.columns)} cols)")
print("Done.")
