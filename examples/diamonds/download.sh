#!/usr/bin/env bash
set -euo pipefail

# Download Diamonds CSV from ggplot2 GitHub
# Source: https://github.com/tidyverse/ggplot2/blob/main/data-raw/diamonds.csv

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
CSV_URL="https://raw.githubusercontent.com/tidyverse/ggplot2/main/data-raw/diamonds.csv"
CSV_FILE="${DATA_DIR}/diamonds.csv"

mkdir -p "${DATA_DIR}"

if [ -f "${CSV_FILE}" ]; then
    echo "Data already downloaded: ${CSV_FILE}"
    exit 0
fi

echo "Downloading Diamonds Dataset..."
curl -L -o "${CSV_FILE}" "${CSV_URL}"

echo "Done. File:"
ls -la "${CSV_FILE}"
