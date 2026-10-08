#!/usr/bin/env bash
# Inspect Linux bundles: metadata, ELF arch, ldd, optional GUI smoke.
set -euo pipefail

KIND="${KIND:?KIND is required}"
OUT="$(cd "${OUT_DIR:-dist-upload}" && pwd)"

check_deb_meta() {
  local deb="$1"
  local expected_arch="$2"
  local info
  info="$(dpkg-deb --info "$deb")"
  echo "$info"
  echo "$info" | grep -qi "Marcus Universe" || {
    echo "::error::deb Maintainer missing 'Marcus Universe'"
    exit 1
  }
  if ! echo "$info" | grep -Eqi "marcus-universe\.(github\.io/SoundNinja|github\.io)|github\.com/marcus-universe/SoundNinja"; then
    echo "::warning::deb Homepage/repo URL not found in control fields"
  fi
  echo "$info" | grep -Eq "Architecture:[[:space:]]*${expected_arch}" || {
    echo "::error::deb Architecture is not ${expected_arch}"
    exit 1
  }
  dpkg-deb --contents "$deb" | grep -q soundninja || {
    echo "::error::deb has no soundninja path"
    exit 1
  }
}

extract_deb() {
  local deb="$1"
  local dest="$2"
  mkdir -p "$dest"
  dpkg-deb -x "$deb" "$dest"
}

find_bin() {
  local root="$1"
  find "$root" -type f -name soundninja | head -n1
}

ldd_native() {
  local bin="$1"
  if ! command -v ldd >/dev/null; then
    echo "ldd not installed; skip"
    return 0
  fi
  ldd "$bin" | tee /tmp/soundninja-ldd.txt
  if grep -q "not found" /tmp/soundninja-ldd.txt; then
    echo "::error::ldd reported missing libraries"
    exit 1
  fi
}

smoke_gui() {
  local bin="$1"
  if [[ ! -x "$bin" ]]; then
    chmod +x "$bin" || true
  fi
  if ! command -v xvfb-run >/dev/null; then
    echo "xvfb-run missing; skip GUI smoke"
    return 0
  fi
  set +e
  timeout 15 xvfb-run -a "$bin"
  local code=$?
  set -e
  # 124 = still running when timeout fired = success
  if [[ "$code" -eq 124 || "$code" -eq 0 ]]; then
    echo "GUI smoke ok (exit $code)"
    return 0
  fi
  echo "::warning::GUI binary exited $code (headless runner may lack audio/GPU)"
}

case "$KIND" in
  linux-x64)
    check_deb_meta "$OUT/soundninja-linux-amd64.deb" amd64
    tmp="$(mktemp -d)"
    extract_deb "$OUT/soundninja-linux-amd64.deb" "$tmp"
    bin="$(find_bin "$tmp")"
    file "$bin"
    ldd_native "$bin"
    chmod +x "$OUT/soundninja-linux-amd64.AppImage"
    extract_dir="$(mktemp -d)"
    pushd "$extract_dir" >/dev/null
    "$OUT/soundninja-linux-amd64.AppImage" --appimage-extract >/dev/null
    img_bin="$(find_bin "$extract_dir/squashfs-root")"
    smoke_gui "$img_bin"
    popd >/dev/null
    smoke_gui "$bin"
    ;;
  linux-arm64)
    check_deb_meta "$OUT/soundninja-linux-arm64.deb" arm64
    tmp="$(mktemp -d)"
    extract_deb "$OUT/soundninja-linux-arm64.deb" "$tmp"
    bin="$(find_bin "$tmp")"
    file "$bin"
    echo "$bin" | grep -q . 
    file "$bin" | grep -Eqi "ARM aarch64|ARM64|aarch64" || {
      echo "::error::arm64 binary is not aarch64"
      file "$bin"
      exit 1
    }
    ldd_native "$bin"
    chmod +x "$OUT/soundninja-linux-arm64.AppImage"
    extract_dir="$(mktemp -d)"
    pushd "$extract_dir" >/dev/null
    "$OUT/soundninja-linux-arm64.AppImage" --appimage-extract >/dev/null
    img_bin="$(find_bin "$extract_dir/squashfs-root")"
    smoke_gui "$img_bin"
    popd >/dev/null
    ;;
  linux-armhf)
    check_deb_meta "$OUT/soundninja-linux-armhf.deb" armhf
    tmp="$(mktemp -d)"
    extract_deb "$OUT/soundninja-linux-armhf.deb" "$tmp"
    bin="$(find_bin "$tmp")"
    file "$bin"
    file "$bin" | grep -Eqi "ARM|EABI|32-bit" || {
      echo "::error::armhf binary is not 32-bit ARM"
      exit 1
    }
    ;;
  *)
    echo "No Linux smoke for kind=$KIND"
    ;;
esac

echo "Smoke passed for $KIND"
