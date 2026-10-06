#!/usr/bin/env bash
set -euo pipefail

# Download NYC TLC Yellow Taxi Trip Data (one month, Parquet)
# Source: https://www.nyc.gov/site/tlc/about/tlc-trip-record-data.page

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"

# Using January 2024 as a stable reference month
YEAR="2024"
MONTH="01"
URL="https://d37ci6vzurychx.cloudfront.net/trip-data/yellow_tripdata_${YEAR}-${MONTH}.parquet"
FILENAME="yellow_tripdata_${YEAR}-${MONTH}.parquet"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/${FILENAME}" ]; then
    echo "Data already downloaded: ${DATA_DIR}/${FILENAME}"
    exit 0
fi

echo "Downloading NYC Yellow Taxi data (${YEAR}-${MONTH})..."
echo "URL: ${URL}"
curl -L -o "${DATA_DIR}/${FILENAME}" "${URL}"

echo "Done. File size:"
ls -lh "${DATA_DIR}/${FILENAME}"
