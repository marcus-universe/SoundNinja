#!/usr/bin/env bash
# Remove bundled libwayland-* from Tauri AppImages and repack.
#
# linuxdeploy copies WebKitGTK's libwayland-* from the build runner. Host Mesa
# newer than that copy (Steam Frame, Arch, Fedora) needs newer libwayland
# symbols, so libEGL_mesa fails to load, WebKitWebProcess aborts with
# "Could not create default EGL display: EGL_BAD_PARAMETER" and the window
# stays white. libwayland is on the AppImage excludelist; the host copy wins.
#
# Usage: appimage-unbundle-wayland.sh <AppImage>...
# Env: SIGN=1 + SIGN_KEY (+ APPIMAGETOOL_SIGN_PASSPHRASE) re-embed the GPG
#      signature. TAURI_SIGNING_PRIVATE_KEY re-signs an existing updater .sig.
set -euo pipefail

APPIMAGETOOL_VERSION="${APPIMAGETOOL_VERSION:-1.9.1}"

if [[ $# -eq 0 ]]; then
  echo "::error::No AppImage given"
  exit 1
fi

case "$(uname -m)" in
  x86_64) tool_arch=x86_64 ;;
  aarch64 | arm64) tool_arch=aarch64 ;;
  armv7l | armhf) tool_arch=armhf ;;
  *) echo "::error::Unsupported host arch $(uname -m)"; exit 1 ;;
esac

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

tool="$work/appimagetool.AppImage"
curl -fsSL --retry 3 -o "$tool" \
  "https://github.com/AppImage/appimagetool/releases/download/${APPIMAGETOOL_VERSION}/appimagetool-${tool_arch}.AppImage"
chmod +x "$tool"

for image in "$@"; do
  image="$(readlink -f "$image")"
  [[ -f "$image" ]] || { echo "::error::Missing $image"; exit 1; }
  echo "Unbundle libwayland from $(basename "$image")"

  rm -rf "$work/squashfs-root"
  chmod +x "$image"
  (cd "$work" && "$image" --appimage-extract >/dev/null)

  removed="$(find "$work/squashfs-root" -name 'libwayland-*.so*' -print -delete)"
  if [[ -z "$removed" ]]; then
    echo "No bundled libwayland; keep as is"
    continue
  fi
  echo "$removed" | sed "s|$work/squashfs-root||"

  sign_args=()
  if [[ "${SIGN:-0}" == "1" && -n "${SIGN_KEY:-}" ]]; then
    sign_args=(--sign --sign-key "$SIGN_KEY")
  fi

  repacked="$work/repacked.AppImage"
  ARCH="$tool_arch" "$tool" --appimage-extract-and-run --no-appstream \
    "${sign_args[@]}" "$work/squashfs-root" "$repacked"
  mv -f "$repacked" "$image"
  chmod +x "$image"

  if [[ -f "$image.sig" ]]; then
    if [[ -n "${TAURI_SIGNING_PRIVATE_KEY:-}" ]]; then
      rm -f "$image.sig"
      npm run --silent tauri -- signer sign "$image"
    else
      echo "::warning::Removed stale updater signature $image.sig (no TAURI_SIGNING_PRIVATE_KEY)"
      rm -f "$image.sig"
    fi
  fi
done
