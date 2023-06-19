#!/usr/bin/env sh
# shellcheck shell=dash
#
# Copyright (C) 2023 CloudTruth, Inc.
#

set -u

CT_CLI_VERSION=
CT_DRAFT_AUTH_TOKEN=
CT_DRAFT_RELEASE_ID=
CT_DRY_RUN=0
CT_DEBUG=

usage() {
    cat <<EOF
Usage: install.sh [ OPTIONS ]

OPTIONS:

  -d | --debug             enable shell debug output
  -h | --help              show usage
  -v | --version <VER>     use a specific version
  -y | --dry-run           download but do not install (may fail if prerequisites are missing)

These options are only used for testing during the CloudTruth release workflow:

  -a | --auth-token <TOK>  authorization token to access draft release
  -r | --release-id <ID>   identity of draft release
EOF
}

### entry point for installer script
main() {
    # parse options
    parse_opts "$@"
    # check required privs
    check_privs
    # check required commands
    require_cmd uname
    require_cmd mktemp
    require_cmd rm
    require_download_cmd
    # only required for draft release
    if [ -n "${CT_DRAFT_RELEASE_ID}" ]; then
        require_cmd jq
    fi
    # set up cleanup handler
    ORIG_DIR=$(pwd)
    TMP_DIR=$(mktemp -d)
    trap cleanup EXIT
    cd "${TMP_DIR}" || fail "Could not enter temp directory: ${TMP_DIR}"
    
    get_target_info
    get_version
    install_cloudtruth
}

### Detect command-line options
parse_opts() {
    while true; do
        case "${1:-}" in
        (-a|--auth-token)
            CT_DRAFT_AUTH_TOKEN=$2
            shift 2;;
        (-d|--debug)
            echo "[debug] enabled"
            CT_DEBUG=1
            set -x
            shift;;
        (-y|--dry-run)
            CT_DRY_RUN=1
            echo "[dry-run] enabled"
            shift;;
        (-h|--help)
            usage
            exit 1;;
        (-r|--release-id)
            CT_DRAFT_RELEASE_ID=$2
            shift 2;;
        (-v|--version)
            CT_CLI_VERSION=$2
            shift 2;;
        (--)  shift; break;;
        (*)
            if [ -n "${1:-}" ]; then
                echo "[error] invalid parameter: ${1}"
                exit 1; # error
            fi
            break;;
        esac
    done
}

### detect target platform information
get_target_info() {
    # detect the architecture
    ARCH=$(uname -m)
    if [ -z "${ARCH}" ]; then
        fail "Cannot determine system architecture."
    fi

    # detect which package type the system wants
    OS=$(uname)
    PKG=
    if [ "${OS}" = "Linux" ]; then
        check_cmd apk && PKG=apk
        check_cmd apt-get && PKG=deb
        check_cmd yum && PKG=rpm

    elif [ "${OS}" = "Darwin" ]; then
        PKG=macos
    fi

    if [ -z "${PKG}" ]; then
        fail "[error] cannot determine system package format"
    fi
}

### Detect which version to install
get_version() {
    local latest_version_url
    if [ -z "${CT_CLI_VERSION}" ]; then
        latest_version_url="https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest"
        CT_CLI_VERSION=$(download "${latest_version_url}" | \
            grep "tag_name" | \
            sed -E 's/.*"([^"]+)".*/\1/')
        echo "[cloudtruth] found latest version: ${CT_CLI_VERSION}"
    else
        echo "[cloudtruth] using requested version: ${CT_CLI_VERSION}"
    fi
}

### Download and install cloudtruth CLI
install_cloudtruth() {
    local target_name
    local package_dir
    local package
    # alpine, macos - no package format yet, use generic binary
    if [ "${PKG}" = "apk" ] || [ "${PKG}" = "macos" ]; then
        # normalize CPU arch
        case $ARCH in
            arm64 | armv8l | armv8b)
                ARCH="aarch64"
            ;;
            armv7l)
                ARCH="armv7"
            ;;
            armv6l)
                ARCH="arm"
            ;;
        esac
        # determine taret name from OS (default to linux)
        if [ "${OS}" = "Darwin" ]; then
            target_name=apple-darwin
        elif [ "${ARCH}" = "aarch64" ]; then
            target_name=unknown-linux-musl
        elif [ "${ARCH}" = "arm" ] || [ "${ARCH}" = "armv7" ]; then
            target_name=unknown-linux-musleabihf
        else
            target_name=unknown-linux-musl
        fi
        package_dir="cloudtruth-${CT_CLI_VERSION}-${ARCH}-${target_name}"
        package="${package_dir}.tar.gz"
        download_cloudtruth "${package}"
        tar xzf "${package}"
        if [ ${CT_DRY_RUN} -ne 0 ]; then
            echo "[dry-run] skipping install of ${package_dir}/cloudtruth"
        else
            install -m 755 -o root "${package_dir}/cloudtruth" /usr/local/bin
        fi
    fi

    # debian based
    if [ "${PKG}" = "deb" ]; then
        if [ "${ARCH}" = "x86_64" ]; then
            ARCH="amd64"
        fi
        package=cloudtruth_${CT_CLI_VERSION}_${ARCH}.deb
        download "${package}"
        if [ ${CT_DRY_RUN} -ne 0 ]; then
            echo "[dry-run] skipping install of ${package}"
        else
            dpkg -i "${package}"
        fi
    fi

    # rpm based
    if [ "${PKG}" = "rpm" ]; then
        package=cloudtruth-${CT_CLI_VERSION}-1.${ARCH}.rpm
        download "${package}"
        if [ ${CT_DRY_RUN} -ne 0 ]; then
            echo "[dry-run] skipping install of ${package}"
        else
            rpm -i "${package}"
        fi
    fi

    if [ ${CT_DRY_RUN} -eq 0 ]; then
        echo "[cloudtruth] installed: $(cloudtruth --version)"
    fi
}

### Download CloudTruth CLI
download_cloudtruth() {
    if [ -z "${CT_DRAFT_RELEASE_ID}" ]; then
      download_release "$1"
    else
      download_draft "$1"
    fi
}

### This used to download release assets
download_release() {
    local package
    local base_url
    local download_url
    package=$1
    base_url="https://github.com/cloudtruth/cloudtruth-cli/releases/download"
    download_url="${base_url}/${CT_CLI_VERSION}/${package}"
    download "${download_url}" "${package}" 
}

### This is used to download a draft release during integration testing
download_draft() {
    local package
    local assetfile
    local asset_id
    local download_url
    package=$1
    assetfile="${CT_DRAFT_RELEASE_ID}.assets.json"

    # get all the assets for the release
    download \
        "https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/${CT_DRAFT_RELEASE_ID}/assets" \
        "${assetfile}"

    # find the asset id for the given package
    asset_id=$(jq ".[] | select(.name==\"${package}\") | .id" "${assetfile}")
    rm "${assetfile}"

    download_url="https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/assets/${asset_id}"
    download "${download_url}" "${package}" 
}

### Uses either curl or wget to download something, depending on what is available
download() {
    local dl_cmd
    local out
    local status
    if check_cmd curl; then
        dl_cmd=curl
    elif check_cmd wget; then
        dl_cmd=wget
    else
        require_download_cmd
    fi
    if [ "$dl_cmd" = curl ]; then
        out=$(curl \
            --retry 3 --proto '=https' --tlsv1.2 --silent --show-error --fail \
            ${CT_DRAFT_AUTH_TOKEN:+ -H "Authorization: token ${CT_DRAFT_AUTH_TOKEN}"} \
            ${CT_DEBUG:+ --verbose} \
            --location "$1" \
            ${2:+ --output "$2"} 2>&1)
        status=$?
    elif [ "$dl_cmd" = wget ]; then
        if [ "$(wget -V 2>&1|head -2|tail -1|cut -f1 -d" ")" = "BusyBox" ]; then
            echo "Warning: using the BusyBox version of wget.  Not enforcing strong cipher suites for TLS or TLS v1.2, this is potentially less secure"
            out=$(wget \
                ${CT_DRAFT_AUTH_TOKEN:+ --header="Authorization: token ${CT_DRAFT_AUTH_TOKEN}"} \
                ${CT_DEBUG:+ --verbose} \
                "$1" \
                ${2:+ -O "$2" } \
                2>&1)
            status=$?
        else
            out=$(wget \
                --https-only --secure-protocol=TLSv1_2 \
                ${CT_DRAFT_AUTH_TOKEN:+ --header="Authorization: token ${CT_DRAFT_AUTH_TOKEN}"} \
                ${CT_DEBUG:+ --verbose} \
                "$1" \
                ${2:+ -O "$2" } \
                2>&1)
            status=$?
        fi
    else
        fail "Could not find download command '$dl_cmd'"
    fi
    if [ -n "$out" ]; then
        echo "$out"
    fi
    return $status
}

### Exit with a helpful message
fail() {
    echo "[error] $1" >&2
    exit 1
}

### Checks for a command on the system and exits with a helpful message if not found
require_cmd() {
    if ! check_cmd "$1"; then
        fail "This install script requires the '$1' command, but it was not found."
    fi
}

### Checks for a command on the system
check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

### Check for a download command on the system
require_download_cmd() {
    if ! check_cmd curl && ! check_cmd wget ; then
        fail "This install script requires either the curl or wget command, but neither were found."
    fi
}

### Installer must run as root
check_privs() {
    if [ "${CT_DRY_RUN}" -eq 0 ] && [ "$(id -u)" -ne 0 ]; then
        fail "This install script requires root privileges. Please run with su or sudo."
    fi
}

### Clean up on exit
cleanup() {
    cd "${ORIG_DIR}" || fail "Could not return to original directory: ${ORIG_DIR}"
    rm -r "${TMP_DIR}"
}

main "$@"
