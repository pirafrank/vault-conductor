#!/bin/sh

#
# vault-conductor install script
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/pirafrank/vault-conductor/main/install.sh | sh
#

set -e

OWNER="pirafrank"
REPO="vault-conductor"
BIN_NAME="vault-conductor"
INSTALL_DIR="${HOME}/.local/bin"

# Determine OS and Arch
OS="$(uname -s)"
ARCH="$(uname -m)"

# Detect system capabilities
detect_alpine() {
    [ -f "/etc/alpine-release" ]
}

detect_glibc() {
    if command -v ldd >/dev/null 2>&1; then
        ldd --version 2>&1 | grep -qi "glibc"
    else
        return 1
    fi
}

# Normalize Arch
case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)
        echo "Error: Sorry, $ARCH architecture is unsupported at this time."
        exit 1
        ;;
esac

echo "It looks like you are running $OS on $ARCH"

# Determine Target based on system detection
TARGET=""
case "$OS" in
    Linux)
        DEFAULT="gnu"
        # Use glibc build if available, otherwise use musl
        if detect_alpine || ! detect_glibc; then
            # Alpine or no glibc detected - use musl
            DEFAULT="musl"
            echo "No glibc detected, using musl build."
        fi
        TARGET="${ARCH}-unknown-linux-${DEFAULT}"
        ;;
    Darwin)
        TARGET="${ARCH}-apple-darwin"
        ;;
    *)
        echo "Error: Sorry, $OS is unsupported at this time."
        exit 1
        ;;
esac

echo "Target: $TARGET"

# Check dependencies
if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl not found. curl and tar are required. Install it and try again."
    exit 1
fi
if ! command -v tar >/dev/null 2>&1; then
    echo "Error: tar not found. curl and tar are required. Install it and try again."
    exit 1
fi

# Get latest version
echo "Fetching latest version..."
LATEST_URL="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
RELEASE_JSON=$(curl -sL "$LATEST_URL")
TAG_NAME=$(echo "$RELEASE_JSON" | grep -m 1 '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
# Remove 'v' prefix for filename construction
VERSION="${TAG_NAME#v}"

if [ -z "$TAG_NAME" ] || [ "$TAG_NAME" = "null" ]; then
    echo "Error: Could not determine latest release version."
    echo ""
    echo "Please download vault-conductor manually from https://github.com/${OWNER}/${REPO}/releases"
    echo "then move it to $INSTALL_DIR/$BIN_NAME:"
    echo "  mv vault-conductor-${VERSION}-${TARGET}.tar.gz $INSTALL_DIR/$BIN_NAME"
    echo "and make it executable:"
    echo "  chmod +x $INSTALL_DIR/$BIN_NAME"
    echo ""
    echo "then run the following command to add it to your PATH:"
    echo ""
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo "  source ~/.bashrc or ~/.zshrc"
    exit 1
fi

echo "Latest release is $TAG_NAME"

# Construct Filename
# Format matches release.yml: vault-conductor-<version>-<target>.tar.gz
FILENAME="${BIN_NAME}-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/${TAG_NAME}/${FILENAME}"

# Download and Install
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading $DOWNLOAD_URL..."
if ! curl -fL "$DOWNLOAD_URL" -o "$TMP_DIR/$FILENAME"; then
    echo "Error: Download failed. Please check your internet connection and if the asset exists for your architecture."
    exit 1
fi

echo "Extracting..."
tar -xzf "$TMP_DIR/$FILENAME" -C "$TMP_DIR"

# Verify binary exists (it should be at root of archive)
if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
    echo "Error: Binary not found in archive."
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
if [ -f "$INSTALL_DIR/$BIN_NAME" ]; then
    echo "Existing binary found. Updating..."
    rm "$INSTALL_DIR/$BIN_NAME"
fi
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo "Successfully installed $BIN_NAME to $INSTALL_DIR/$BIN_NAME"
echo ""
echo "Run $BIN_NAME --help to get started."
echo ""

echo "Cleaning up..."
rm -rf "$TMP_DIR"

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo "Warning: $INSTALL_DIR is not in your PATH."
        echo "To use $BIN_NAME, add the directory to your PATH:"
        echo ""
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "You can add this to your shell config (e.g., ~/.zshrc or ~/.bashrc)."
        ;;
esac

