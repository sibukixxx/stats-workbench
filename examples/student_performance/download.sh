#!/usr/bin/env bash
set -euo pipefail

# Download Student Performance Dataset from UCI ML Repository
# Source: https://archive.ics.uci.edu/ml/datasets/Student+Performance

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
ZIP_URL="https://archive.ics.uci.edu/ml/machine-learning-databases/00320/student.zip"
ZIP_FILE="${DATA_DIR}/student.zip"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/student-mat.csv" ]; then
    echo "Data already downloaded: ${DATA_DIR}/student-mat.csv"
    exit 0
fi

echo "Downloading Student Performance Dataset..."
curl -L -o "${ZIP_FILE}" "${ZIP_URL}"

echo "Extracting..."
unzip -o "${ZIP_FILE}" -d "${DATA_DIR}"

echo "Done."
