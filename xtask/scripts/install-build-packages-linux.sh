#!/usr/bin/env sh
set -e
case $TARGET in
    arm-unknown-linux-gnueabihf | armv7-unknown-linux-gnueabihf)
        sudo apt-get -y update
        sudo apt-get -y install gcc-arm-linux-gnueabihf
    ;;
    arm-unknown-linux-musleabihf | armv7-unknown-linux-musleabihf)
        sudo apt-get -y update
        sudo apt-get -y install gcc-arm-linux-musleabihf
    ;;
    aarch64-unknown-linux-gnu)
        sudo apt-get -y update
        sudo apt-get -y install gcc-aarch64-linux-gnu
    ;;
    aarch64-unknown-linux-musl)
        sudo dpkg --add-architecture arm64
        sudo apt update
        sudo apt install -y --no-install-recommends musl:arm64
esac
