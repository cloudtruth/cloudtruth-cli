#!/usr/bin/env sh
# shellcheck shell=dash
#
# Copyright (C) 2023 CloudTruth, Inc.
#

usage() {
    cat <<EOF
Usage: install.sh [ OPTIONS ] -- [ INSTALLER OPTIONS ]

OPTIONS:

  -v | --verbose           enable shell debug output
  -h | --help              show usage
  -v | --version <VER>     use a specific version

These options are only used for testing during the CloudTruth release workflow:

  -a | --auth-token <TOK>  authorization token to access draft release
  -r | --release-id <ID>   identity of draft release

EOF
}

parse_opts() {
    while true; do
        case $1 in
            (-v|--verbose)
                shift;;
            (-h|--help)
                usage
                exit 1
                ;;
            (--)  
                shift;
                break
                ;;
            (*)
                if [ -n "$1" ]; then
                    fail "invalid parameter: ${1}"
                fi
                break
                ;;
        esac
    done
}

main() {
    require_download_cmd
    require_cmd uname
    require_cmd mktemp
    # require_cmd chmod
    # require_cmd mkdir
    require_cmd rm

    parse_opts "$@"

    ORIG_DIR=$(pwd)
    TMP_DIR=$(mktemp -d)
    trap cleanup EXIT
    cd "${TMP_DIR}" || fail "Could not enter temp directory: ${TMP_DIR}"
    
    get_target_info
    download_latest_installer
}

get_target_info() {
    ARCH=$(uname -m)
    if [ -z "${ARCH}" ]; then
        fail "Cannot determine system architecture."
    fi
    case "$ARCH" in
        aarch64 | arm64)
            ARCH=aarch64
            ;;
        x86_64 | x86-64 | x64 | amd64)
            ARCH=x86_64
            ;;
        *)
            fail "unknown architecture: $ARCH"
            ;;
    esac

    OS=$(uname -s)
    if [ -z "${OS}" ]; then
        fail "Cannot determine operating system."
    fi
    case "$OS" in
        Linux)
            case "$ARCH" in
                aarch64)
                    TARGET="unknown-linux-gnu"
                    ;;
                x86_64)
                    TARGET="unknown-linux-musl"
                    ;;
            esac
            ;;

        Darwin)
            TARGET="apple-darwin"
            ;;

        *)
            fail "unrecognized OS: $OS"
            ;;
    esac
}

download_latest_installer() {
    local latest_installer_version
    local base_url
    local package_dir
    local package
    local download_url
    latest_installer_version=$(\
        download https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest | \
        grep "tag_name" | \
        sed -E 's/.*"([^"]+)".*/\1/'\
    )
    echo "[cloudtruth] found latest installer version: ${latest_installer_version}"
    base_url="https://github.com/cloudtruth/cloudtruth-cli/releases/download"
    package_dir="cloudtruth-${latest_installer_version}-${ARCH}-${TARGET}"
    package="${package_dir}.tar.gz"
    download_url="${base_url}/${latest_installer_version}/${package}"
    download "$download_url" "$package"
}

download() {
    local dl_cmd
    local err
    local status

    if check_cmd curl; then
        dl_cmd=curl
    elif check_cmd wget; then
        dl_cmd=wget
    fi
    if [ "$dl_cmd" = curl ]; then
        err=$(curl --retry 3 --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$1" ${2:+ --output "$2"} 2>&1)
        status=$?
    elif [ "$dl_cmd" = wget ]; then
        if [ "$(wget -V 2>&1|head -2|tail -1|cut -f1 -d" ")" = "BusyBox" ]; then
            echo "Warning: using the BusyBox version of wget.  Not enforcing strong cipher suites for TLS or TLS v1.2, this is potentially less secure"
            err=$(wget "$1" -O "$2" 2>&1)
            status=$?
        else
            err=$(wget --https-only --secure-protocol=TLSv1_2 "$1" ${2:+ -O "$2" } 2>&1)
            status=$?
        fi
    else
        fail "Could not find download command '$dl_cmd'"
    fi
    if [ -n "$err" ]; then
        echo "$err"
    fi
    return $status
}

fail() {
    echo "$1" >&2
    exit 1
}

require_cmd() {
    if ! check_cmd "$1"; then
        fail "This install script requires the '$1' command, but it was not found."
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}


require_download_cmd() {
    if ! check_cmd curl && ! check_cmd wget ; then
        fail "This install script requires either the curl or wget command, but beither were found."
    fi
}

cleanup() {
    cd "${ORIG_DIR}" || fail "Could not return to original directory: ${ORIG_DIR}"
    rm -r "${TMP_DIR}"
}

main "$@" || exit 1