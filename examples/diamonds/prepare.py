#!/usr/bin/env python3
"""Convert Diamonds CSV to Parquet for Stats Workbench.

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

csv_path = os.path.join(DATA_DIR, "diamonds.csv")
parquet_path = os.path.join(DATA_DIR, "diamonds.parquet")

if not os.path.exists(csv_path):
    print(f"Error: {csv_path} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting diamonds.csv to diamonds.parquet...")
df = pd.read_csv(csv_path)

# Keep only numeric columns
numeric_cols = ["carat", "depth", "table", "price", "x", "y", "z"]
df = df[numeric_cols]

# Drop rows with invalid measurements (x=0, y=0, or z=0)
before = len(df)
df = df[(df["x"] != 0) & (df["y"] != 0) & (df["z"] != 0)]
after = len(df)
if before != after:
    print(f"Dropped {before - after} rows with zero dimensions.")

df.to_parquet(parquet_path, index=False)
print(f"Written: {parquet_path} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
