#!/usr/bin/env sh
rpm_arch=$(echo "$TARGET" | cut -d- -f1)
rpm_toml_version="version = '${PACKAGE_VERSION}${RPM_SNAPSHOT:+^$RPM_SNAPSHOT}'"
cargo generate-rpm \
--arch "$rpm_arch" \
--target "$TARGET" \
--set-metadata "$rpm_toml_version"

#rename RPM to use the github release tag
rpm_dir="target/$TARGET/generate-rpm/"
rpm_path="${rpm_dir}/cloudtruth-${RELEASE_TAG}-1.${rpm_arch}.rpm"
# rename generated RPM to match github release name
mv -f "${rpm_dir}/cloudtruth-*${rpm_arch}.rpm" "${rpm_path}"
echo "$rpm_path"