#!/usr/bin/env bash
set -euo pipefail

# Download NOAA GHCN-Daily data for a single station
# Default: JFK Airport (USW00094789)
#
# Source: https://www.ncei.noaa.gov/products/land-based-station/global-historical-climatology-network-daily

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/data"
STATION_ID="${STATION_ID:-USW00094789}"  # JFK Airport, NY

# GHCN-Daily CSV from NOAA
URL="https://www.ncei.noaa.gov/data/global-historical-climatology-network-daily/access/${STATION_ID}.csv"
FILENAME="${STATION_ID}.csv"

mkdir -p "${DATA_DIR}"

if [ -f "${DATA_DIR}/${FILENAME}" ]; then
    echo "Data already downloaded: ${DATA_DIR}/${FILENAME}"
    exit 0
fi

echo "Downloading GHCN-Daily data for station ${STATION_ID}..."
echo "URL: ${URL}"
curl -L -o "${DATA_DIR}/${FILENAME}" "${URL}"

echo "Done. File size:"
ls -lh "${DATA_DIR}/${FILENAME}"
