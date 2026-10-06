#!/usr/bin/env python3
"""Generate California Housing Parquet from scikit-learn."""
import os
import sys

try:
    import pandas as pd
except ImportError:
    print("Error: pandas is required. Install with: pip install pandas pyarrow")
    sys.exit(1)

try:
    from sklearn.datasets import fetch_california_housing
except ImportError:
    print("Error: scikit-learn is required. Install with: pip install scikit-learn")
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "data")
dst = os.path.join(DATA_DIR, "california_housing.parquet")

os.makedirs(DATA_DIR, exist_ok=True)

if os.path.exists(dst):
    print(f"Already exists: {dst}")
    sys.exit(0)

print("Fetching California Housing dataset from scikit-learn...")
data = fetch_california_housing(as_frame=True)
df = data.frame  # includes target column "MedHouseVal"

df.to_parquet(dst, index=False)
print(f"Written: {dst} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
