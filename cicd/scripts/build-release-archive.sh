#!/usr/bin/env sh
set -e

case $TARGET in
    *windows*)
        BIN_NAME="${PROJECT_NAME}.exe"
    ;;
    *)
        BIN_NAME="$PROJECT_NAME"
    ;;
esac

ARCHIVE_NAME="$PROJECT_NAME-$RELEASE_TAG-$TARGET"

mkdir -p "$ARCHIVE_NAME"

# Find where the build script put the generated completions files.
src_completions_dir="$(find . -name cloudtruth.bash -print0 | xargs -0 ls -t | head -n 1 | xargs dirname)"

# Move the generated shell completion files to a location cargo-deb will read from.
dst_completions_dir="target/$TARGET/release/completions"
mv "$src_completions_dir" "$dst_completions_dir"

cp README.md LICENSE "$ARCHIVE_NAME/"
cp -a "$dst_completions_dir" "$ARCHIVE_NAME/"
cp "target/$TARGET/release/$BIN_NAME" "$ARCHIVE_NAME/"

case $TARGET in
    *windows*)
        7z a "$ARCHIVE_NAME.zip" "$ARCHIVE_NAME"
        echo "ASSET=$ARCHIVE_NAME.zip" >> "$GITHUB_ENV"
    ;;

    *)
        tar -czf "$ARCHIVE_NAME.tar.gz" "$ARCHIVE_NAME"
        echo "ASSET=$ARCHIVE_NAME.tar.gz" >> "$GITHUB_ENV"
    ;;
esac
