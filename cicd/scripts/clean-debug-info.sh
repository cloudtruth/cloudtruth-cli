#!/usr/bin/env sh
set -e

if [ -d target ]; then
    find target \( -name '*.dSYM' -or -name '*.pdb' \) -print -exec rm -rf {} +
fi
