#!/usr/bin/env python3
"""Convert Student Performance CSV to Parquet for Stats Workbench.

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

csv_path = os.path.join(DATA_DIR, "student-mat.csv")
parquet_path = os.path.join(DATA_DIR, "student_performance.parquet")

if not os.path.exists(csv_path):
    print(f"Error: {csv_path} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting student-mat.csv to student_performance.parquet...")
df = pd.read_csv(csv_path, sep=";")

# Select numeric columns relevant for analysis
numeric_cols = ["age", "studytime", "failures", "absences", "G1", "G2", "G3"]
df = df[numeric_cols]

df.to_parquet(parquet_path, index=False)
print(f"Written: {parquet_path} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
