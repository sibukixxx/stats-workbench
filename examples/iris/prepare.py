#!/usr/bin/env python3
"""Convert Iris CSV to Parquet for StatForge.

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

csv_path = os.path.join(DATA_DIR, "iris.csv")
parquet_path = os.path.join(DATA_DIR, "iris.parquet")

if not os.path.exists(csv_path):
    print(f"Error: {csv_path} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting iris.csv to iris.parquet...")

# Original UCI iris.data has no header
df = pd.read_csv(csv_path, header=None,
                 names=["sepal_length", "sepal_width", "petal_length", "petal_width", "species"])

# Drop species column — StatForge works with numeric columns
df = df.drop(columns=["species"])

df.to_parquet(parquet_path, index=False)
print(f"Written: {parquet_path} ({len(df)} rows, {len(df.columns)} cols)")

print("Done.")
