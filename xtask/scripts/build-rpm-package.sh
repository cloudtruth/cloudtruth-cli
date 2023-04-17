#!/usr/bin/env sh
set -e
rpm_arch=$(echo "$TARGET" | cut -d- -f1)
rpm_toml_version="version = '${PACKAGE_VERSION}${RPM_SNAPSHOT:+^$RPM_SNAPSHOT}'"
cargo generate-rpm \
--arch "$rpm_arch" \
--target "$TARGET" \
--set-metadata "$rpm_toml_version" \
--payload-compress none

#rename RPM to use the github release tag
rpm_dir="target/$TARGET/generate-rpm/"
rpm_path="${rpm_dir}/cloudtruth-${RELEASE_TAG}-1.${rpm_arch}.rpm"
# rename generated RPM to match github release name
mv -f "${rpm_dir}/cloudtruth-"*"${rpm_arch}.rpm" "${rpm_path}" || true
echo "RPM_PATH=${rpm_path}" >> "$GITHUB_ENV"
echo "RPM_NAME=$(basename "$rpm_path")" >> "$GITHUB_ENV"
