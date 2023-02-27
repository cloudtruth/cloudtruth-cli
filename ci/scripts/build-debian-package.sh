#!/usr/bin/env sh
cargo deb --no-build --no-strip \
--target "$TARGET" \
--deb-version "$PACKAGE_VERSION" \
${DEB_REVISION:+ --deb-revision "$DEB_REVISION" }
# rename deb package to use the github release tag
deb_dir="target/$TARGET/debian"
src_deb_path=$(ls "$deb_dir"/*.deb)
dest_deb_path=$(echo "$src_deb_path" | sed -E "s/${PACKAGE_VERSION}/${RELEASE_TAG}/")
mv -f "$src_deb_path" "$dest_deb_path"
echo "$dest_deb_path"