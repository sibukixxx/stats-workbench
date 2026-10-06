#!/usr/bin/env bash
set -euo pipefail

# Download UCI Bike Sharing Dataset
# Source: https://archive.ics.uci.edu/dataset/275/bike+sharing+dataset

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
ZIP_URL="https://archive.ics.uci.edu/static/public/275/bike+sharing+dataset.zip"
ZIP_FILE="${DATA_DIR}/bike_sharing.zip"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/day.csv" ]; then
    echo "Data already downloaded: ${DATA_DIR}/day.csv"
    exit 0
fi

echo "Downloading Bike Sharing Dataset..."
curl -L -o "${ZIP_FILE}" "${ZIP_URL}"

echo "Extracting..."
unzip -o "${ZIP_FILE}" -d "${DATA_DIR}"

echo "Done. Files:"
ls -la "${DATA_DIR}"/*.csv 2>/dev/null || echo "  (check ${DATA_DIR} for extracted files)"
