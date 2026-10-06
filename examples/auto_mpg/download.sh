#!/usr/bin/env bash
set -euo pipefail

# Download Auto MPG Dataset
# Source: UCI Machine Learning Repository

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
DATA_URL="https://archive.ics.uci.edu/ml/machine-learning-databases/auto-mpg/auto-mpg.data"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/auto-mpg.data" ]; then
    echo "Data already downloaded: ${DATA_DIR}/auto-mpg.data"
    exit 0
fi

echo "Downloading Auto MPG Dataset..."
curl -L -o "${DATA_DIR}/auto-mpg.data" "${DATA_URL}"

echo "Done."
