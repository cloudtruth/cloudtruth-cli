#!/usr/bin/env sh
set -e

ARCHIVE_FILE="$1"

DEBUG_FILES="$(find target -name 'cloudtruth*.dSYM' -or -name 'test_*.dSYM' -or -name '*.pdb')"

if [ -n "$DEBUG_FILES" ]; then
    zstd -d "$ARCHIVE_FILE" -o tmp-archive.tar
    # shellcheck disable=SC2086
    tar rvf tmp-archive.tar $DEBUG_FILES
    zstd --rm -f tmp-archive.tar -o "$ARCHIVE_FILE"
fi
