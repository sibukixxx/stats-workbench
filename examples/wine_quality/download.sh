#!/usr/bin/env bash
set -euo pipefail

# Download UCI Red Wine Quality Dataset
# Source: https://archive.ics.uci.edu/ml/datasets/wine+quality

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
CSV_URL="https://archive.ics.uci.edu/ml/machine-learning-databases/wine-quality/winequality-red.csv"
CSV_FILE="${DATA_DIR}/winequality-red.csv"

mkdir -p "${DATA_DIR}"

if [ -f "${CSV_FILE}" ]; then
    echo "Data already downloaded: ${CSV_FILE}"
    exit 0
fi

echo "Downloading Red Wine Quality Dataset..."
curl -L -o "${CSV_FILE}" "${CSV_URL}"

echo "Done. File:"
ls -la "${CSV_FILE}"
