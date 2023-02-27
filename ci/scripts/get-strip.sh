#!/usr/bin/env sh
# Define the executable for stripping binaries, if one exists.
STRIP="strip"
case $TARGET in
    arm-unknown-linux-gnueabihf)
        STRIP="arm-linux-gnueabihf-strip"
    ;;
    aarch64-unknown-linux-gnu)
        STRIP="aarch64-linux-gnu-strip"
    ;;
esac;
echo $STRIP