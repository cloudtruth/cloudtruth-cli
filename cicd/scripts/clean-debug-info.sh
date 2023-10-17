#!/usr/bin/env sh
set -e

find target \( -name '*.dSYM' -or -name '*.pdb' \) -exec rm -rf {} \;