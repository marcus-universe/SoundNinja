#!/usr/bin/env bash
# Cross-compile sysroot for armv7-unknown-linux-gnueabihf on Ubuntu 22.04 amd64.
set -euo pipefail

sudo dpkg --add-architecture armhf

codename="$(lsb_release -cs)"
# Keep host arch on archive.ubuntu.com; armhf comes from ports.
if [[ -f /etc/apt/sources.list ]]; then
  sudo sed -i -E 's/^deb ([^[])/deb [arch=amd64] \1/' /etc/apt/sources.list
  sudo sed -i -E 's/^deb-src /# deb-src /' /etc/apt/sources.list
fi

sudo tee /etc/apt/sources.list.d/armhf-ports.list >/dev/null <<EOF
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports ${codename} main restricted universe multiverse
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports ${codename}-updates main restricted universe multiverse
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports ${codename}-security main restricted universe multiverse
EOF

sudo apt-get update
sudo apt-get install -y \
  gcc-arm-linux-gnueabihf \
  g++-arm-linux-gnueabihf \
  pkg-config \
  clang \
  libclang-dev \
  file \
  libwebkit2gtk-4.1-dev:armhf \
  libayatana-appindicator3-dev:armhf \
  librsvg2-dev:armhf \
  libasound2-dev:armhf \
  libpipewire-0.3-dev:armhf \
  libpulse-dev:armhf \
  libgtk-3-dev:armhf \
  libglib2.0-dev:armhf \
  libsoup-3.0-dev:armhf \
  libjavascriptcoregtk-4.1-dev:armhf

{
  echo "CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc"
  echo "CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc"
  echo "CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++"
  echo "PKG_CONFIG_ALLOW_CROSS=1"
  echo "PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig"
  echo "PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf/pkgconfig"
  echo "PKG_CONFIG_SYSROOT_DIR=/"
  echo "BINDGEN_EXTRA_CLANG_ARGS=--target=arm-linux-gnueabihf -I/usr/include -I/usr/include/arm-linux-gnueabihf"
} >> "$GITHUB_ENV"

echo "armhf cross toolchain ready"
