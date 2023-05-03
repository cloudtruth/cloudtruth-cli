#!/usr/bin/env sh
# shellcheck shell=dash
#
# Copyright (C) 2023 CloudTruth, Inc.
#

main() {
    require_download_cmd
    require_cmd uname
    require_cmd mktemp
    require_cmd chmod
    require_cmd mkdir
    require_cmd rm
    require_cmd rmdir

    get_target_info

    echo "$OS $ARCH $TARGET"


    # if [ -z "${CT_CLI_VERSION}" ]; then
    #     CT_VER_FILE_URL="https://api.github.com/repos/cloudtruth/cloudtruth-cli/releases/latest"
    #     CT_CLI_VERSION=$(curl "${CURL_OPTS}" "${CT_VER_FILE_URL}" | \
    #             grep "tag_name" | \
    #             sed -E 's/.*"([^"]+)".*/\1/')
    #     echo "[cloudtruth] found latest version: ${CT_CLI_VERSION}"
    # else
    #     echo "[cloudtruth] using requested version: ${CT_CLI_VERSION}"
    # fi
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
                    TARGET="aarch64-unknown-linux-gnu"
                    ;;
                x86_64)
                    TARGET="x86_64-unknown-linux-musl"
                    ;;
            esac
            ;;

        Darwin)
            TARGET="$ARCH-apple-darwin"
            ;;

        *)
            fail "unrecognized OS: $OS"
            ;;
    esac
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
        err=$(curl --retry 3 --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$1" --output "$2" 2>&1)
        status=$?
    elif [ "$dl_cmd" = wget ]; then
        if [ "$(wget -V 2>&1|head -2|tail -1|cut -f1 -d" ")" = "BusyBox" ]; then
            echo "Warning: using the BusyBox version of wget.  Not enforcing strong cipher suites for TLS or TLS v1.2, this is potentially less secure"
            err=$(wget "$1" -O "$2" 2>&1)
            status=$?
        else
            err=$(wget --https-only --secure-protocol=TLSv1_2 "$1" -O "$2" 2>&1)
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

main "$@" || exit 1