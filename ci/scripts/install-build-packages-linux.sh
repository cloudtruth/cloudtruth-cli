#!/usr/bin/env sh
set -e
case $TARGET in
    arm-unknown-linux-gnueabihf)
        sudo apt-get -y update
        sudo apt-get -y install gcc-arm-linux-gnueabihf
    ;;
    aarch64-unknown-linux-gnu)
        sudo apt-get -y update
        sudo apt-get -y install gcc-aarch64-linux-gnu
    ;;
esac
