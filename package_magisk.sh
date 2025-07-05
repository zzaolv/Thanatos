#!/bin/bash
# 文件路径: /Thanatos/package_magisk.sh

# Stop on error
set -e

PROJECT_ROOT=$(pwd)
MODULE_ID="thanatos-core"
BUILD_DIR="$PROJECT_ROOT/daemon/build"
OUTPUT_DIR="$PROJECT_ROOT/release"
MODULE_SRC_DIR="$PROJECT_ROOT/magisk"
DAEMON_BINARY="$BUILD_DIR/thanatosd"

echo "--- Thanatos Magisk Module Packager ---"

# 1. Check if daemon binary exists
if [ ! -f "$DAEMON_BINARY" ]; then
    echo "Error: Daemon binary not found at $DAEMON_BINARY"
    echo "Please build the daemon first by running 'ninja' in '$BUILD_DIR'"
    exit 1
fi

echo "Daemon binary found."

# 2. Create temporary packaging directory
TMP_DIR=$(mktemp -d)
echo "Using temporary directory: $TMP_DIR"

# 3. Create module structure
echo "Creating Magisk module structure..."
mkdir -p "$TMP_DIR/system/bin"

# 4. Copy files
echo "Copying files..."
# Copy module metadata and scripts
cp -r "$MODULE_SRC_DIR/"* "$TMP_DIR/"
echo "Copied module scripts and metadata."

# Copy the compiled daemon binary
cp "$DAEMON_BINARY" "$TMP_DIR/system/bin/"
echo "Copied 'thanatosd' binary."

# 5. Zip the module
echo "Zipping module..."
mkdir -p "$OUTPUT_DIR"
OUTPUT_ZIP="$OUTPUT_DIR/Thanatos-Module-$(date +%Y%m%d-%H%M).zip"
cd "$TMP_DIR"
zip -r9 "$OUTPUT_ZIP" .
cd "$PROJECT_ROOT"

# 6. Clean up
echo "Cleaning up temporary directory..."
rm -rf "$TMP_DIR"

echo "---"
echo "Success! Module created at: $OUTPUT_ZIP"
echo "---"
