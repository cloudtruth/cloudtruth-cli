#!/usr/bin/env sh
#
# Copyright (C) 2021 CloudTruth, Inc.
#

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

# We use "$@" instead of $* to preserve argument-boundary information
ARGS=$(getopt -o 'dhv' --long 'debug,help,version' -- "$@") || exit
eval "set -- $ARGS"

while true; do
    case $1 in
      (-d|--debug)
            set -x
            shift;;
      (-h|--help)
            echo "Usage: install.sh [ OPTIONS ]"
            echo ""
            echo "OPTIONS:"
            echo "  -d | --debug           enable shell debug output"
            echo "  -h | --help            show usage"
            echo "  -v | --version <VER>   use a specific version"
            exit 2;;
      (-v|--version)
            CT_CLI_VERSION=$2
            shift;;
      (--)  shift; break;;
      (*)   exit 1;;           # error
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
    CT_CLI_VERSION=$(curl --silent "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest" | grep "tag_name" | sed -E 's/.*"([^"]+)".*/\1/')
    echo "Latest version: ${CT_CLI_VERSION}"
else
    echo "Using version: ${CT_CLI_VERSION}"
fi

### Install       ############################################################

# alpine, centos, rhel, macos - no package format yet, use generic binary
if [ "$PKG" = "apk" ] || [ "$PKG" = "rpm" ] || [ "$PKG" = "macos" ]; then
    if [ "$PKG" = "macos" ]; then
        PACKAGEDIR=cloudtruth-${CT_CLI_VERSION}-${ARCH}-apple-darwin
    else
        PACKAGEDIR=cloudtruth-${CT_CLI_VERSION}-${ARCH}-unknown-linux-musl
    fi
    PACKAGE=${PACKAGEDIR}.tar.gz
    CWD=$(pwd)
    cd /tmp || exit
    curl -sLOJ "https://github.com/cloudtruth/cloudtruth-cli/releases/download/${CT_CLI_VERSION}/${PACKAGE}"
    tar xzf "${PACKAGE}" || exit
    install -m 755 -o root "${PACKAGEDIR}/cloudtruth" /usr/local/bin
    rm -rf "${PACKAGEDIR}"
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
    cd /tmp || exit
    curl -sLOJ "https://github.com/cloudtruth/cloudtruth-cli/releases/download/${CT_CLI_VERSION}/${PACKAGE}"
    dpkg -i "${PACKAGE}"
    rm "${PACKAGE}"
    cd "${CWD}" || exit
fi

cloudtruth --version
