#!/usr/bin/env bash
set -euo pipefail

APP_HOME="${XDG_DATA_HOME:-$HOME/.local/share}/parallax"
LOCAL_BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"

rm -f "$LOCAL_BIN/parallax"
rm -f "$DESKTOP_DIR/parallax.desktop"
rm -f "$ICON_DIR/parallax.svg"
rm -rf "$APP_HOME"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
fi

echo "Parallax removed for the current user."
