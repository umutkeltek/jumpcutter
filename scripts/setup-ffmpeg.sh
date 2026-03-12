#!/bin/bash
# Sets up FFmpeg sidecar binaries for development
# Creates symlinks from src-tauri/binaries/ to system FFmpeg

set -e

BINARIES_DIR="src-tauri/binaries"
TARGET=$(rustc -vV | grep host | awk '{print $2}')

echo "Setting up FFmpeg for target: $TARGET"

FFMPEG_PATH=$(which ffmpeg 2>/dev/null || true)
FFPROBE_PATH=$(which ffprobe 2>/dev/null || true)

if [ -z "$FFMPEG_PATH" ]; then
    echo "ERROR: ffmpeg not found in PATH"
    echo "Install it with: brew install ffmpeg (macOS) or choco install ffmpeg (Windows)"
    exit 1
fi

if [ -z "$FFPROBE_PATH" ]; then
    echo "ERROR: ffprobe not found in PATH"
    exit 1
fi

mkdir -p "$BINARIES_DIR"

# Create symlinks with target triple suffix (required by Tauri externalBin)
SUFFIX=""
if [[ "$TARGET" == *"windows"* ]]; then
    SUFFIX=".exe"
fi

ln -sf "$FFMPEG_PATH" "$BINARIES_DIR/ffmpeg-${TARGET}${SUFFIX}"
ln -sf "$FFPROBE_PATH" "$BINARIES_DIR/ffprobe-${TARGET}${SUFFIX}"

echo "Linked:"
echo "  ffmpeg  -> $BINARIES_DIR/ffmpeg-${TARGET}${SUFFIX}"
echo "  ffprobe -> $BINARIES_DIR/ffprobe-${TARGET}${SUFFIX}"
echo "Done!"
