#!/usr/bin/env bash
set -euo pipefail

# Download FIFA 22 Complete Player Dataset
# Source: https://www.kaggle.com/datasets/stefanoleone992/fifa-22-complete-player-dataset

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/players_22.csv" ]; then
    echo "Data already downloaded: ${DATA_DIR}/players_22.csv"
    exit 0
fi

if command -v kaggle &> /dev/null; then
    echo "Downloading FIFA 22 Player Dataset via Kaggle CLI..."
    kaggle datasets download -d stefanoleone992/fifa-22-complete-player-dataset -p "${DATA_DIR}" --unzip
else
    echo "Error: kaggle CLI not found."
    echo ""
    echo "Option 1: Install kaggle CLI"
    echo "  pip install kaggle"
    echo "  # Set up ~/.kaggle/kaggle.json with your API token"
    echo "  # See: https://www.kaggle.com/docs/api"
    echo ""
    echo "Option 2: Manual download"
    echo "  1. Visit https://www.kaggle.com/datasets/stefanoleone992/fifa-22-complete-player-dataset"
    echo "  2. Download and extract to ${DATA_DIR}/"
    echo "  3. Ensure ${DATA_DIR}/players_22.csv exists"
    exit 1
fi

echo "Done."
