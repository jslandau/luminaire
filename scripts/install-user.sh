#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
os="$(uname -s)"

case "$os" in
  Linux)
    cargo install --path "$repo_root/src-tauri"

    data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
    desktop_dir="$data_home/applications"
    icon_dir="$data_home/icons/hicolor/scalable/apps"

    cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin/luminaire"

    install -d "$desktop_dir" "$icon_dir"
    sed "s|^Exec=.*|Exec=$cargo_bin|" "$repo_root/src-tauri/luminaire.desktop" > "$desktop_dir/luminaire.desktop"
    chmod 0644 "$desktop_dir/luminaire.desktop"
    install -m 0644 "$repo_root/src-tauri/icons/icon.svg" "$icon_dir/luminaire.svg"

    if command -v update-desktop-database >/dev/null 2>&1; then
      update-desktop-database "$desktop_dir" || true
    fi

    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
      gtk-update-icon-cache -q "$data_home/icons/hicolor" || true
    fi

    echo "Installed Luminaire binary with cargo and desktop assets under $data_home."
    echo "Make sure Cargo's bin directory is on PATH: ${CARGO_HOME:-$HOME/.cargo}/bin"
    ;;
  Darwin)
    cargo tauri build --bundles app

    app_src="$repo_root/src-tauri/target/release/bundle/macos/Luminaire.app"
    app_dest_dir="$HOME/Applications"
    app_dest="$app_dest_dir/Luminaire.app"

    if [ ! -d "$app_src" ]; then
      echo "error: expected app bundle not found at $app_src" >&2
      exit 1
    fi

    mkdir -p "$app_dest_dir"
    rm -rf "$app_dest"
    cp -R "$app_src" "$app_dest_dir/"

    echo "Installed Luminaire.app to $app_dest"
    echo "If macOS Gatekeeper blocks first launch, right-click the app and choose Open."
    ;;
  *)
    echo "error: unsupported OS for install-user: $os" >&2
    exit 1
    ;;
esac
