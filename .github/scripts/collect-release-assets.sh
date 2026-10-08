#!/usr/bin/env bash
# Collect Tauri bundles under dist-upload/ with stable asset names.
set -euo pipefail
shopt -s nullglob

KIND="${KIND:?KIND is required}"
OUT="${OUT_DIR:-dist-upload}"
mkdir -p "$OUT"

copy_one() {
  local src="$1"
  local dest_name="$2"
  if [[ ! -f "$src" ]]; then
    echo "::error::Missing $src"
    exit 1
  fi
  cp "$src" "$OUT/$dest_name"
  echo "Collected $dest_name"
}

copy_optional_sig() {
  local src="$1"
  local dest_name="$2"
  if [[ -f "$src" ]]; then
    cp "$src" "$OUT/$dest_name"
    echo "Collected $dest_name"
  fi
}

case "$KIND" in
  windows)
    files=(src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/*x64-setup.exe)
    copy_one "${files[0]:-}" "soundninja-windows-x64-setup.exe"
    ;;
  linux-x64)
    debs=(src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/deb/*_amd64.deb)
    images=(src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/appimage/*_amd64.AppImage)
    copy_one "${debs[0]:-}" "soundninja-linux-amd64.deb"
    copy_one "${images[0]:-}" "soundninja-linux-amd64.AppImage"
    copy_optional_sig "${images[0]:-}.sig" "soundninja-linux-amd64.AppImage.sig"
    ;;
  linux-arm64)
    debs=(src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/deb/*_arm64.deb)
    images=(src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/appimage/*_aarch64.AppImage)
    if [[ ${#images[@]} -eq 0 ]]; then
      images=(src-tauri/target/aarch64-unknown-linux-gnu/release/bundle/appimage/*_arm64.AppImage)
    fi
    copy_one "${debs[0]:-}" "soundninja-linux-arm64.deb"
    copy_one "${images[0]:-}" "soundninja-linux-arm64.AppImage"
    copy_optional_sig "${images[0]:-}.sig" "soundninja-linux-arm64.AppImage.sig"
    ;;
  linux-armhf)
    debs=(src-tauri/target/armv7-unknown-linux-gnueabihf/release/bundle/deb/*_armhf.deb)
    copy_one "${debs[0]:-}" "soundninja-linux-armhf.deb"
    ;;
  macos-arm)
    files=(src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/*_aarch64.dmg)
    copy_one "${files[0]:-}" "soundninja-macos-arm64.dmg"
    ;;
  macos-intel)
    files=(src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/*_x64.dmg)
    copy_one "${files[0]:-}" "soundninja-macos-x64.dmg"
    ;;
  *)
    echo "::error::Unknown kind=$KIND"
    exit 1
    ;;
esac

ls -la "$OUT"
