#!/usr/bin/env bash
set -euo pipefail

# Download Titanic Dataset
# Source: https://www.kaggle.com/c/titanic (GitHub mirror)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
CSV_URL="https://raw.githubusercontent.com/datasciencedojo/datasets/master/titanic.csv"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/titanic.csv" ]; then
    echo "Data already downloaded: ${DATA_DIR}/titanic.csv"
    exit 0
fi

echo "Downloading Titanic Dataset..."
curl -L -o "${DATA_DIR}/titanic.csv" "${CSV_URL}"

echo "Done."
