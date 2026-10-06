#!/usr/bin/env bash
set -euo pipefail

# Download UCI Cleveland Heart Disease Dataset
# Source: https://archive.ics.uci.edu/ml/datasets/heart+disease

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
CSV_URL="https://archive.ics.uci.edu/ml/machine-learning-databases/heart-disease/processed.cleveland.data"
CSV_FILE="${DATA_DIR}/heart.csv"

mkdir -p "${DATA_DIR}"

if [ -f "${CSV_FILE}" ]; then
    echo "Data already downloaded: ${CSV_FILE}"
    exit 0
fi

echo "Downloading Cleveland Heart Disease Dataset..."
curl -L -o "${CSV_FILE}" "${CSV_URL}"

echo "Done. File:"
ls -la "${CSV_FILE}"
