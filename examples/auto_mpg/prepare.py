#!/usr/bin/env python3
"""Convert Auto MPG data to Parquet for StatForge."""
import os
import sys

try:
    import pandas as pd
except ImportError:
    print("Error: pandas is required. Install with: pip install pandas pyarrow")
    sys.exit(1)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "data")

src = os.path.join(DATA_DIR, "auto-mpg.data")
dst = os.path.join(DATA_DIR, "auto_mpg.parquet")

if not os.path.exists(src):
    print(f"Error: {src} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting auto-mpg.data to auto_mpg.parquet...")

col_names = ["mpg", "cylinders", "displacement", "horsepower", "weight",
             "acceleration", "model_year", "origin", "car_name"]

df = pd.read_csv(src, sep=r"\s+", names=col_names, na_values="?",
                 comment=None, quotechar='"')

# Drop car_name (string) and origin (categorical code)
df = df.drop(columns=["car_name", "origin"])
# Drop rows with missing horsepower
df = df.dropna(subset=["horsepower"])
# Ensure horsepower is float
df["horsepower"] = df["horsepower"].astype(float)

df.to_parquet(dst, index=False)
print(f"Written: {dst} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
