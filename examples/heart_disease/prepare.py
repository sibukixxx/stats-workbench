#!/usr/bin/env python3
"""Convert Cleveland Heart Disease data to Parquet for Stats Workbench.

Usage:
    python3 prepare.py

Requires: pip install pandas pyarrow
"""

import os
import sys

try:
    import pandas as pd
    import numpy as np
except ImportError:
    print("Error: pandas is required. Install with: pip install pandas pyarrow")
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "data")

csv_file = os.path.join(DATA_DIR, "heart.csv")
parquet_file = os.path.join(DATA_DIR, "heart_disease.parquet")

if not os.path.exists(csv_file):
    print(f"Error: {csv_file} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting heart.csv to heart_disease.parquet...")

# No header in the raw file; assign column names
col_names = [
    "age", "sex", "cp", "trestbps", "chol", "fbs", "restecg",
    "thalach", "exang", "oldpeak", "slope", "ca", "thal", "target",
]
df = pd.read_csv(csv_file, header=None, names=col_names)

# Replace "?" with NaN and drop incomplete rows
df = df.replace("?", np.nan)
df = df.dropna()

# Convert target: >0 → 1 (presence of heart disease)
df = df.astype(float)
df["target"] = (df["target"] > 0).astype(float)

df.to_parquet(parquet_file, index=False)
print(f"Written: {parquet_file} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
