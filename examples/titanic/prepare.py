#!/usr/bin/env python3
"""Convert Titanic CSV to Parquet for StatForge.

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

csv_path = os.path.join(DATA_DIR, "titanic.csv")
parquet_path = os.path.join(DATA_DIR, "titanic.parquet")

if not os.path.exists(csv_path):
    print(f"Error: {csv_path} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting titanic.csv to titanic.parquet...")
df = pd.read_csv(csv_path)

# Encode Sex as numeric (male=0, female=1)
df["sex_encoded"] = df["Sex"].map({"male": 0, "female": 1}).astype("Int64")

# Select columns for analysis
cols = ["Survived", "Pclass", "sex_encoded", "Age", "SibSp", "Parch", "Fare"]
df = df[cols]

# Drop rows where Age is null
df = df.dropna(subset=["Age"])

df.to_parquet(parquet_path, index=False)
print(f"Written: {parquet_path} ({len(df)} rows, {len(df.columns)} cols)")

print("Done.")
