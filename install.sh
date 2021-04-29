#!/usr/bin/env sh
#
# Copyright (C) 2021 CloudTruth, Inc.
#

### Control     ############################################################
CT_DRY_RUN=0
CT_DOWNLOAD_URL=""
CT_CLI_VERSION=""
CT_DOWNLOAD_AUTH_TOKEN=""

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
    >&2 echo "Cannot determine system package format."
    exit 43
fi

### Arguments     ############################################################
while true; do
    case $1 in
      (-a|--auth-token)
            CT_DOWNLOAD_AUTH_TOKEN=$2
            shift 2;;
      (-d|--debug)
            set -x
            shift;;
      (--dry-run)
            CT_DRY_RUN=1
            shift;;
      (-h|--help)
            echo "Usage: install.sh [ OPTIONS ]"
            echo ""
            echo "OPTIONS:"
            echo "  -a | --auth-token <TOK>  authorization token for download"
            echo "  -d | --debug             enable shell debug output"
            echo "  -h | --help              show usage"
            echo "  -u | --url <URL>         download directory URL"
            echo "  -v | --version <VER>     use a specific version"
            echo "       --dry-run           download, but do not install"
            exit 2;;
      (-u|--url)
            CT_DOWNLOAD_URL=$2
            shift 2;;
      (-v|--version)
            CT_CLI_VERSION=$2
            shift 2;;
      (--)  shift; break;;
      (*)
            if [ -n "${1}" ]; then
                echo "Invalid parameter: $1"
                exit 1;           # error
            fi
            break;;
    esac
done
# remaining="$@"

### Prerequisites ############################################################

case "$PKG" in
    (apk)
        # alpine - no package format yet, use generic
        apk add curl || exit
        ;;
    (deb)
        # debian based
        if [ -f /.dockerenv ]; then
            apt-get update
        fi
        apt-get install --no-install-recommends --yes ca-certificates curl
        if [ -f /.dockerenv ]; then
            apt-get purge
        fi
        ;;
    (rpm)
        # centos, rhel
        yum install -y curl || exit
        ;;
esac

### Auto-Version  ############################################################

if [ -z "${CT_CLI_VERSION}" ]; then
    CT_VER_FILE_URL="https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest"
    CT_CLI_VERSION=$(curl --silent "${CT_VER_FILE_URL}" | \
              grep "tag_name" | \
              sed -E 's/.*"([^"]+)".*/\1/')
    echo "Latest version: ${CT_CLI_VERSION}"
else
    echo "Using version: ${CT_CLI_VERSION}"
fi

if [ -z "${CT_DOWNLOAD_URL}" ]; then
    CT_VER_BASE_URL="https://github.com/cloudtruth/cloudtruth-cli/releases/download"
    CT_DOWNLOAD_URL="${CT_VER_BASE_URL}/${CT_CLI_VERSION}"
fi

### Install       ############################################################
TMP_DIR="/tmp"

download() {
    url=$1
    auth_token=$2
    filename="${TMP_DIR}/$(basename "$url")"
    minsize=100 # downloads that fail often have a 9-byte file
    auth_header=""
    accept_header=""
    if [ -n "${auth_token}" ]; then
        auth_header="Authorization: token $auth_token"
        accept_header="Accept: application/octet-stream"
    fi

    set -x
    curl -sLOJ -H "$auth_header" -H "$accept_header" -o "${filename}" "$url"
    # NOTE: 'wc' is used to determine filesize, since 'stat' format args vary
    filesize=$(wc -c < "$filename")
    if [ "$filesize" -lt "$minsize" ]; then
        echo "${filename} was only ${filesize} bytes"
        stat "$filename"
        echo "Failed to download: $url"
        exit 3
    fi
    echo "Downloaded: $url"
    set +x
}

# alpine, centos, rhel, macos - no package format yet, use generic binary
if [ "$PKG" = "apk" ] || [ "$PKG" = "rpm" ] || [ "$PKG" = "macos" ]; then
    if [ "$PKG" = "macos" ]; then
        PACKAGE_DIR=cloudtruth-${CT_CLI_VERSION}-${ARCH}-apple-darwin
    else
        PACKAGE_DIR=cloudtruth-${CT_CLI_VERSION}-${ARCH}-unknown-linux-musl
    fi
    PACKAGE=${PACKAGE_DIR}.tar.gz
    CWD=$(pwd)
    cd "${TMP_DIR}" || exit
    download "${CT_DOWNLOAD_URL}/${PACKAGE}" "${CT_DOWNLOAD_AUTH_TOKEN}"
    tar xzf "${PACKAGE}" || exit
    if [ ${CT_DRY_RUN} -ne 0 ]; then
        echo "Skipping install of ${PACKAGE_DIR}/cloudtruth"
    else
        install -m 755 -o root "${PACKAGE_DIR}/cloudtruth" /usr/local/bin
    fi
    rm -rf "${PACKAGE_DIR}"
    rm "${PACKAGE}"
    cd "${CWD}" || exit
fi

# debian based
if [ "$PKG" = "deb" ]; then
    if [ "${ARCH}" = "x86_64" ]; then
        ARCH="amd64"
    fi
    PACKAGE=cloudtruth_${CT_CLI_VERSION}_${ARCH}.deb
    CWD=$(pwd)
    cd "${TMP_DIR}" || exit
    download "${CT_DOWNLOAD_URL}/${PACKAGE}" "${CT_DOWNLOAD_AUTH_TOKEN}"
    if [ ${CT_DRY_RUN} -ne 0 ]; then
        echo "Skipping install of ${PACKAGE}"
    else
        dpkg -i "${PACKAGE}"
    fi
    rm "${PACKAGE}"
    cd "${CWD}" || exit
fi

if [ ${CT_DRY_RUN} -eq 0 ]; then
    cloudtruth --version
fi
