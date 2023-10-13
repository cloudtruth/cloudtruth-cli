#!/usr/bin/env sh
set -e
# cargo-deb only works with release profile so we need to copy everything to the release dir
if [ "$CARGO_PROFILE" != release ]; then
    cp -rf "target/$TARGET/$CARGO_PROFILE" "target/$TARGET/release/"  
fi
cargo deb --no-build --no-strip \
--target "$TARGET" \
--deb-version "$PACKAGE_VERSION" \
${DEB_REVISION:+ --deb-revision "$DEB_REVISION" }
# rename deb package to use the github release tag
deb_dir="target/$TARGET/debian"
src_deb_path=$(ls "$deb_dir"/*.deb)
dest_deb_path=$(echo "$src_deb_path" | sed -E "s/${PACKAGE_VERSION}/${RELEASE_TAG}/")
mv -f "$src_deb_path" "$dest_deb_path" || true
echo "DEB_PATH=$dest_deb_path" >> "$GITHUB_ENV"
echo "DEB_NAME=$(basename "$DEB_PATH")" >> "$GITHUB_ENV"
