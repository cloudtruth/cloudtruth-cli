#!/usr/bin/env sh

INPUT_FILE="$1"
OUTPUT_VAR="$2"

CHECKSUM_FILE="$(basename "${INPUT_FILE}").sha512"
sha512sum "${INPUT_FILE}" > "${CHECKSUM_FILE}"
echo "${OUTPUT_VAR}=${CHECKSUM_FILE}" >> "$GITHUB_ENV"