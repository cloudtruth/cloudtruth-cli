#!/usr/bin/env sh
### Installs cargo-binstall on the GitHub Actions runner.
### This speeds up the installation of utilities such as cargo-deb and cross
### by installation pre-built binaries rather than compiling from source

BASE_BINSTALL_URL='https://github.com/cargo-bins/cargo-binstall/releases/latest/download/'
case $(uname) in
    Linux)
        BINSTALL_FILE='cargo-binstall-x86_64-unknown-linux-musl.tgz'
        BINSTALL_URL="$BASE_BINSTALL_URL/$BINSTALL_FILE"
        curl -LSs "$BINSTALL_URL" | tar -C "$HOME/.cargo/bin" -xvz
    ;;
    Darwin)
        BINSTALL_FILE='cargo-binstall-universal-apple-darwin.zip'
        BINSTALL_URL="$BASE_BINSTALL_URL/$BINSTALL_FILE"
        curl -LSs "$BINSTALL_URL" > "/tmp/$BINSTALL_FILE"
        unzip -o -d "$HOME/.cargo/bin" "/tmp/$BINSTALL_FILE"
    ;;
    *Windows*)
        BINSTALL_FILE='cargo-binstall-x86_64-pc-windows-msvc.zip'
        BINSTALL_URL="$BASE_BINSTALL_URL/$BINSTALL_FILE"
        curl -LSs "$BINSTALL_URL" > "/tmp/$BINSTALL_FILE"
        unzip -o -d "$HOME/.cargo/bin" "/tmp/$BINSTALL_FILE"
esac;