#!/usr/bin/env bash
set -euo pipefail

os="$(uname -s)"

case "$os" in
  Linux)
    cargo uninstall luminaire || true

    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
    rm -f "$data_home/applications/luminaire.desktop"
    rm -f "$data_home/icons/hicolor/scalable/apps/luminaire.svg"

    if command -v update-desktop-database >/dev/null 2>&1; then
      update-desktop-database "$data_home/applications" || true
    fi

    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
      gtk-update-icon-cache -q "$data_home/icons/hicolor" || true
    fi

    echo "Removed Luminaire user install assets."
    ;;
  Darwin)
    rm -rf "$HOME/Applications/Luminaire.app"
    echo "Removed $HOME/Applications/Luminaire.app"
    ;;
  *)
    echo "error: unsupported OS for uninstall-user: $os" >&2
    exit 1
    ;;
esac
