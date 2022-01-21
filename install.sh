#!/usr/bin/env sh
#
# Copyright (C) 2021 CloudTruth, Inc.
#

set -e

### Control     ############################################################

CT_CLI_VERSION=
CT_DRAFT_AUTH_TOKEN=
CT_DRAFT_RELEASE_ID=
CT_DRY_RUN=0
CT_INSTALL_PREREQUISITES=1

### Detection     ############################################################

# detect the architecture
ARCH=$(uname -m)

if [ -z "${ARCH}" ]; then
    >&2 echo "Cannot determine system architecture."
    exit 42
fi

# detect which package type the system wants
OS=$(uname)
PKG=
if [ "${OS}" = "Linux" ]; then
    command -v apk && PKG=apk
    command -v apt-get && PKG=deb
    command -v yum && PKG=rpm

elif [ "${OS}" = "Darwin" ]; then
    PKG=macos
fi

if [ -z "${PKG}" ]; then
    >&2 echo "[error] cannot determine system package format"
    exit 43
fi

### Arguments     ############################################################

while true; do
    case $1 in
      (-a|--auth-token)
            CT_DRAFT_AUTH_TOKEN=$2
            shift 2;;
      (-d|--debug)
            echo "[debug] enabled"
            EXTRA_CURL_OPTIONS="-v"
            set -x
            shift;;
      (-y|--dry-run)
            CT_DRY_RUN=1
            echo "[dry-run] enabled"
            shift;;
      (-h|--help)
            echo "Usage: install.sh [ OPTIONS ]"
            echo ""
            echo "OPTIONS:"
            echo ""
            echo "  -d | --debug             enable shell debug output"
            echo "  -h | --help              show usage"
            echo "  -n | --no-prerequisites  do not attempt to install prerequisites"
            echo "  -v | --version <VER>     use a specific version"
            echo "  -y | --dry-run           download but do not install (may fail if prerequisites are missing)"
            echo ""
            echo "These options are only used for testing during the CloudTruth release workflow:"
            echo ""
            echo "  -a | --auth-token <TOK>  authorization token to access draft release"
            echo "  -r | --release-id <ID>   identity of draft release"
            echo ""
            exit 2;;
      (-n|--no-prerequisites)
            CT_INSTALL_PREREQUISITES=0
            shift;;
      (-r|--release-id)
            CT_DRAFT_RELEASE_ID=$2
            shift 2;;
      (-v|--version)
            CT_CLI_VERSION=$2
            shift 2;;
      (--)  shift; break;;
      (*)
            if [ -n "${1}" ]; then
                echo "[error] invalid parameter: ${1}"
                exit 1; # error
            fi
            break;;
    esac
done
# remaining="$@"

### Prerequisites ############################################################

PREREQUISITES="curl"
if [ -n "${CT_DRAFT_RELEASE_ID}" ]; then
  # additional requirements to handle GitHub draft release integration testing
  PREREQUISITES="${PREREQUISITES} ca-certificates jq"
fi

# shellcheck disable=SC2086
prerequisites() {
    case "$PKG" in
        (apk)
            # alpine - no package format yet, use generic
            if [ ${CT_DRY_RUN} -ne 0 ]; then
                CT_PREREQ_DRY_RUN="--simulate"
            fi
            apk add ${CT_PREREQ_DRY_RUN} ${PREREQUISITES}
            ;;
        (deb)
            # debian based
            # had problems downloading from GitHub on debian buster without ca-certificates update
            PREREQUISITES="${PREREQUISITES} ca-certificates"
            if [ ${CT_DRY_RUN} -ne 0 ]; then
                CT_PREREQ_DRY_RUN="--dry-run"
            fi
            if [ -f /.dockerenv ]; then
                apt-get update
            fi
            apt-get install --no-install-recommends --yes ${CT_PREREQ_DRY_RUN} ${PREREQUISITES}
            if [ -f /.dockerenv ]; then
                apt-get purge
            fi
            ;;
        (rpm)
            # centos, rhel
            if [ ${CT_DRY_RUN} -ne 0 ]; then
                CT_PREREQ_DRY_RUN="--setopt tsflags=test"
            fi
            if [ -n "${CT_DRAFT_RELEASE_ID}" ] && [ "$(rpm -E "%{rhel}")" -eq 7 ]; then
              # jq is needed for draft release parsing, jq is in centos7 epel repository
              yum -y install ${CT_PREREQ_DRY_RUN} epel-release
              yum repolist
            fi
            yum -y install ${CT_PREREQ_DRY_RUN} ${PREREQUISITES}
            ;;
    esac
}

if [ ${CT_INSTALL_PREREQUISITES} -eq 1 ]; then
  prerequisites
fi

### Auto-Version  ############################################################

if [ -z "${CT_CLI_VERSION}" ]; then
    CT_VER_FILE_URL="https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest"
    CT_CLI_VERSION=$(curl --silent "${CT_VER_FILE_URL}" | \
              grep "tag_name" | \
              sed -E 's/.*"([^"]+)".*/\1/')
    echo "[cloudtruth] found latest version: ${CT_CLI_VERSION}"
else
    echo "[cloudtruth] using requested version: ${CT_CLI_VERSION}"
fi

### Install       ############################################################

cleanup() {
  cd "${ORIG_DIR}"
  rm -r "${TMP_DIR}"
}

ORIG_DIR=$(pwd)
TMP_DIR=$(mktemp -d)
trap cleanup EXIT
cd "${TMP_DIR}"

download() {
    if [ -z "${CT_DRAFT_RELEASE_ID}" ]; then
      download_release "$1"
    else
      download_draft "$1"
    fi
}

# this is used to download release assets
download_release() {
    package=$1
    base_url="https://github.com/cloudtruth/cloudtruth-cli/releases/download"
    download_url="${base_url}/${CT_CLI_VERSION}/${package}"
    curl ${EXTRA_CURL_OPTIONS} -fsL -H "Accept: application/octet-stream" -o "${package}" "${download_url}"
}

# this is used to download a draft release during integration testing
download_draft() {
    package=$1
    assetfile="${CT_DRAFT_RELEASE_ID}.assets.json"

    # get all the assets for the release
    curl ${EXTRA_CURL_OPTIONS} -fs -H "Authorization: token ${CT_DRAFT_AUTH_TOKEN}" -o "${assetfile}" \
        "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/${CT_DRAFT_RELEASE_ID}/assets"

    # find the asset id for the given package
    asset_id=$(jq ".[] | select(.name==\"${package}\") | .id" "${assetfile}")
    rm "${assetfile}"

    download_url="https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/assets/${asset_id}"
    curl ${EXTRA_CURL_OPTIONS} -fs --location-trusted -H "Authorization: token ${CT_DRAFT_AUTH_TOKEN}" -H "Accept: application/octet-stream" -o "${package}" "${download_url}"
}

# alpine, macos - no package format yet, use generic binary
if [ "${PKG}" = "apk" ] || [ "${PKG}" = "macos" ]; then
    if [ "${PKG}" = "macos" ]; then
        if [ "${ARCH}" = "arm64" ]; then
            ARCH="aarch64"
        fi
        PACKAGE_DIR=cloudtruth-${CT_CLI_VERSION}-${ARCH}-apple-darwin
    else
        PACKAGE_DIR=cloudtruth-${CT_CLI_VERSION}-${ARCH}-unknown-linux-musl
    fi
    PACKAGE=${PACKAGE_DIR}.tar.gz
    download "${PACKAGE}"
    tar xzf "${PACKAGE}"
    if [ ${CT_DRY_RUN} -ne 0 ]; then
        echo "[dry-run] skipping install of ${PACKAGE_DIR}/cloudtruth"
    else
        install -m 755 -o root "${PACKAGE_DIR}/cloudtruth" /usr/local/bin
    fi
fi

# debian based
if [ "${PKG}" = "deb" ]; then
    if [ "${ARCH}" = "x86_64" ]; then
        ARCH="amd64"
    fi
    # debian package names strip build information off the release version name
    # this is typical in a draft build, like 0.3.0_mytest.1 => 0.3.0
    CT_CLI_VERSION_STUB=$(echo "${CT_CLI_VERSION}" | sed 's/[^0-9.]*\([0-9.]*\).*/\1/')
    PACKAGE=cloudtruth_${CT_CLI_VERSION_STUB}_${ARCH}.deb
    download "${PACKAGE}"
    if [ ${CT_DRY_RUN} -ne 0 ]; then
        echo "[dry-run] skipping install of ${PACKAGE}"
    else
        dpkg -i "${PACKAGE}"
    fi
fi

# rpm based
if [ "${PKG}" = "rpm" ]; then
    # rpm package names strip build information off the release version name
    # this is typical in a draft build, like 0.3.0_mytest.1 => 0.3.0
    CT_CLI_VERSION_STUB=$(echo "${CT_CLI_VERSION}" | sed 's/[^0-9.]*\([0-9.]*\).*/\1/')
    PACKAGE=cloudtruth-${CT_CLI_VERSION_STUB}-1.${ARCH}.rpm
    download "${PACKAGE}"
    if [ ${CT_DRY_RUN} -ne 0 ]; then
        echo "[dry-run] skipping install of ${PACKAGE}"
    else
        rpm -i "${PACKAGE}"
    fi
fi

if [ ${CT_DRY_RUN} -eq 0 ]; then
    echo "[cloudtruth] installed: $(cloudtruth --version)"
fi
