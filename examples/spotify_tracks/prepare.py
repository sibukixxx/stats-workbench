#!/usr/bin/env python3
"""Convert Spotify Tracks CSV to Parquet for Stats Workbench.

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

csv_path = os.path.join(DATA_DIR, "dataset.csv")
parquet_path = os.path.join(DATA_DIR, "spotify_tracks.parquet")

if not os.path.exists(csv_path):
    print(f"Error: {csv_path} not found. Run ./download.sh first.")
    sys.exit(1)

print("Converting dataset.csv to spotify_tracks.parquet...")
df = pd.read_csv(csv_path)

# Select numeric audio feature columns
cols = [
    "danceability",
    "energy",
    "loudness",
    "speechiness",
    "acousticness",
    "instrumentalness",
    "liveness",
    "valence",
    "tempo",
    "duration_ms",
    "popularity",
]

df = df[cols]

# Drop rows with any NaN in selected columns
before = len(df)
df = df.dropna(subset=cols)
after = len(df)
if before != after:
    print(f"Dropped {before - after} rows with NaN values.")

df.to_parquet(parquet_path, index=False)
print(f"Written: {parquet_path} ({len(df)} rows, {len(df.columns)} cols)")
print("Done.")
