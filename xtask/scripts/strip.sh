#!/usr/bin/env sh
# Define the executable for stripping binaries, if one exists.
set -e
echo 'Pre-stripped binary size'
ls -lh "target/$TARGET/release/"
# Find strip executable
STRIP="strip"
case $TARGET in
    arm-unknown-linux-gnueabihf)
        STRIP="arm-linux-gnueabihf-strip"
    ;;
    arm7-unknown-linux-gnueabihf)
        STRIP="arm-linux-gnueabihf-strip"
    ;;
    aarch64-unknown-linux-gnu)
        STRIP="aarch64-linux-gnu-strip"
    ;;
esac;
# Strip binaries
"$STRIP" "target/$TARGET/release/$PROJECT_NAME"
echo 'Post-stripped binary size'
ls -lh "target/$TARGET/release/"
