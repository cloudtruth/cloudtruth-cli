#!/usr/bin/env sh
### Used by GitHub Workflows to install the correct Rust toolchain

# When rustup is updated, it tries to replace its binary, which on Windows is somehow locked.
# This can result in the CI failure, see: https://github.com/rust-lang/rustup/issues/3029
rustup set auto-self-update disable
# legacy CLI version with no rust-toolchain file
if [ -f rust-toolchain ] || [ -f rust-toolchain.toml ]; then
    rustup toolchain install 1.63.0 --profile minimal
else
    # this command forces install of version in rust-toolchain file
    # this might change in the future but for now it's the best way
    rustup show
fi