#!/usr/bin/env python3
"""Convert Wine Quality CSV to Parquet for StatForge.

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

csv_file = os.path.join(DATA_DIR, "winequality-red.csv")
parquet_file = os.path.join(DATA_DIR, "wine_quality.parquet")

if not os.path.exists(csv_file):
    print(f"Error: {csv_file} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting winequality-red.csv to wine_quality.parquet...")
df = pd.read_csv(csv_file, sep=";")

# Rename columns: replace spaces with underscores
df.columns = [c.strip().replace(" ", "_") for c in df.columns]

# Keep all columns
cols = [
    "fixed_acidity", "volatile_acidity", "citric_acid", "residual_sugar",
    "chlorides", "free_sulfur_dioxide", "total_sulfur_dioxide", "density",
    "pH", "sulphates", "alcohol", "quality",
]
df = df[cols]

df.to_parquet(parquet_file, index=False)
print(f"Written: {parquet_file} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
