#!/usr/bin/env bash
# Fill PKGBUILD checksums, locally makepkg, namcap, and emit AUR source tarball.
# Expects VERSION and collected/soundninja-linux-{amd64,arm64,armhf}.deb
# Optional: GPG_FINGERPRINT, SIGN=1, CARCH_BUILD=x86_64|aarch64
set -euo pipefail

ROOT="${GITHUB_WORKSPACE:-$(pwd)}"
AUR_DIR="${AUR_DIR:-$ROOT/aur}"
ART="${ARTIFACT_DIR:-$ROOT/collected}"
VERSION="${VERSION:?VERSION is required}"
SIGN="${SIGN:-0}"
CARCH_BUILD="${CARCH_BUILD:-x86_64}"
OUT_DIR="${OUT_DIR:-$ROOT/dist-upload}"

if [[ "$(id -u)" -eq 0 ]]; then
  pacman -Syu --noconfirm --needed base-devel namcap git python sudo
  id builder >/dev/null 2>&1 || useradd -m builder
  echo "builder ALL=(ALL) NOPASSWD: ALL" >/etc/sudoers.d/builder
  chmod 440 /etc/sudoers.d/builder
  mkdir -p "$AUR_DIR" "$ART" "$OUT_DIR"
  chown -R builder:builder "$ROOT"
  exec su builder -s /bin/bash -c "env \
    GITHUB_WORKSPACE=$(printf %q "$ROOT") \
    AUR_DIR=$(printf %q "$AUR_DIR") \
    ARTIFACT_DIR=$(printf %q "$ART") \
    VERSION=$(printf %q "$VERSION") \
    SIGN=$(printf %q "$SIGN") \
    CARCH_BUILD=$(printf %q "$CARCH_BUILD") \
    OUT_DIR=$(printf %q "$OUT_DIR") \
    GPG_FINGERPRINT=$(printf %q "${GPG_FINGERPRINT:-}") \
    bash $(printf %q "$0")"
fi

mkdir -p "$AUR_DIR" "$ART" "$OUT_DIR"
cd "$AUR_DIR"

deb_name() {
  case "$1" in
    x86_64) echo "soundninja-linux-amd64.deb" ;;
    aarch64) echo "soundninja-linux-arm64.deb" ;;
    armv7h) echo "soundninja-linux-armhf.deb" ;;
    *) echo "::error::unknown arch $1"; return 1 ;;
  esac
}

copy_deb() {
  local arch="$1"
  local src="$ART/$(deb_name "$arch")"
  local dest="$AUR_DIR/soundninja-bin-${VERSION}-${arch}.deb"
  if [[ -f "$src" ]]; then
    cp "$src" "$dest"
    echo "Using $src"
  else
    echo "::warning::Missing $src for $arch"
  fi
}

copy_deb x86_64
copy_deb aarch64
copy_deb armv7h
cp "$ROOT/LICENSE" "$AUR_DIR/GPL-3.0-only.txt"

hash_or_skip() {
  if [[ -f "$1" ]]; then
    sha256sum "$1" | awk '{print $1}'
  else
    echo "SKIP"
  fi
}

sum_x64="$(hash_or_skip "$AUR_DIR/soundninja-bin-${VERSION}-x86_64.deb")"
sum_arm64="$(hash_or_skip "$AUR_DIR/soundninja-bin-${VERSION}-aarch64.deb")"
sum_armhf="$(hash_or_skip "$AUR_DIR/soundninja-bin-${VERSION}-armv7h.deb")"
if [[ "$sum_x64" == "SKIP" || "$sum_arm64" == "SKIP" || "$sum_armhf" == "SKIP" ]]; then
  echo "::error::Incomplete AUR package: need amd64, arm64, and armhf debs (got x64=$sum_x64 arm64=$sum_arm64 armhf=$sum_armhf)"
  ls -la "$ART" "$AUR_DIR" || true
  exit 1
fi

apply_metadata() {
  sed -i "s/^pkgver=.*/pkgver=${VERSION}/" PKGBUILD
  sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
  if [[ -n "${GPG_FINGERPRINT:-}" && "$GPG_FINGERPRINT" != "0000000000000000000000000000000000000000" ]]; then
    sed -i "s/validpgpkeys=.*/validpgpkeys=('${GPG_FINGERPRINT}')/" PKGBUILD
  fi
  python3 - "$sum_x64" "$sum_arm64" "$sum_armhf" <<'PY'
import sys
from pathlib import Path
x64, arm64, armhf = sys.argv[1:]
p = Path("PKGBUILD")
text = p.read_text()
repls = [
    ("sha256sums_x86_64=('SKIP')", f"sha256sums_x86_64=('{x64}')"),
    ("sha256sums_aarch64=('SKIP')", f"sha256sums_aarch64=('{arm64}')"),
    ("sha256sums_armv7h=('SKIP')", f"sha256sums_armv7h=('{armhf}')"),
]
for old, new in repls:
    if old not in text:
        sys.exit(f"missing {old} in PKGBUILD")
    text = text.replace(old, new, 1)
p.write_text(text)
PY
}

apply_metadata

build_deb="soundninja-bin-${VERSION}-${CARCH_BUILD}.deb"
if [[ ! -f "$build_deb" ]]; then
  echo "::error::Cannot makepkg: missing $build_deb"
  exit 1
fi

python3 - "$CARCH_BUILD" "$VERSION" <<'PY'
import re
import sys
from pathlib import Path
arch, ver = sys.argv[1], sys.argv[2]
p = Path("PKGBUILD")
text = p.read_text()
for other in ("x86_64", "aarch64", "armv7h"):
    if other == arch:
        continue
    text = text.replace(f"source_{other}=", f"_disabled_source_{other}=")
    text = text.replace(f"sha256sums_{other}=", f"_disabled_sha256sums_{other}=")
text = re.sub(
    rf"source_{arch}=\([\s\S]*?\)",
    f"source_{arch}=('soundninja-bin-{ver}-{arch}.deb')",
    text,
    count=1,
)
p.write_text(text)
PY

makepkg_args=(-s --noconfirm --skippgpcheck)
if [[ "$SIGN" == "1" ]]; then
  makepkg_args+=(--sign)
fi
makepkg "${makepkg_args[@]}"

if command -v namcap >/dev/null; then
  namcap PKGBUILD || true
  namcap soundninja-bin-*.pkg.tar.* || true
fi

shopt -s nullglob
pkgs=(soundninja-bin-*.pkg.tar.zst soundninja-bin-*.pkg.tar.xz)
if [[ ${#pkgs[@]} -gt 0 ]] && command -v pacman >/dev/null; then
  sudo pacman -U --noconfirm "${pkgs[@]}"
fi

# Restore URL-based PKGBUILD with real checksums for the AUR tarball.
if git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  git -C "$ROOT" checkout -- aur/PKGBUILD
fi
apply_metadata
makepkg --printsrcinfo > .SRCINFO

tar czf "$OUT_DIR/soundninja-aur-${VERSION}.tar.gz" \
  PKGBUILD .SRCINFO LICENSE REUSE.toml GPL-3.0-only.txt

for pkg in soundninja-bin-*.pkg.tar.zst soundninja-bin-*.pkg.tar.xz; do
  [[ -e "$pkg" ]] || continue
  cp "$pkg" "$OUT_DIR/"
done
for sig in soundninja-bin-*.pkg.tar.*.sig; do
  [[ -e "$sig" ]] || continue
  cp "$sig" "$OUT_DIR/"
done

echo "AUR artifacts:"
ls -la "$OUT_DIR"
