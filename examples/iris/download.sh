#!/usr/bin/env bash
set -euo pipefail

# Download Iris Dataset
# Source: UCI Machine Learning Repository

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
CSV_URL="https://archive.ics.uci.edu/ml/machine-learning-databases/iris/iris.data"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/iris.csv" ]; then
    echo "Data already downloaded: ${DATA_DIR}/iris.csv"
    exit 0
fi

echo "Downloading Iris Dataset..."
curl -L -o "${DATA_DIR}/iris.csv" "${CSV_URL}"

echo "Done."
