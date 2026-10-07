#!/usr/bin/env bash
# Cross-compile sysroot for armv7-unknown-linux-gnueabihf.
# PipeWire/libspa 0.10 bindgen needs Ubuntu 24.04 (noble) headers; jammy is too old.
set -euo pipefail

sudo dpkg --add-architecture armhf
codename="$(lsb_release -cs)"

# Host sources keep the runner's mirror (azure.archive.ubuntu.com); only pin them to amd64.
# armhf comes from ports.ubuntu.com.
if [[ -f /etc/apt/sources.list.d/ubuntu.sources ]]; then
  if ! grep -q '^Architectures:' /etc/apt/sources.list.d/ubuntu.sources; then
    sudo sed -i '/^Types:/a Architectures: amd64' /etc/apt/sources.list.d/ubuntu.sources
  fi
  cat /etc/apt/sources.list.d/ubuntu.sources
fi
if [[ -f /etc/apt/sources.list ]]; then
  sudo sed -i -E 's/^deb ([^[])/deb [arch=amd64] \1/' /etc/apt/sources.list
  sudo sed -i -E 's/^deb-src /# deb-src /' /etc/apt/sources.list
fi

sudo tee /etc/apt/sources.list.d/armhf-ports.sources >/dev/null <<EOF
Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports
Suites: ${codename} ${codename}-updates ${codename}-security
Components: main universe restricted multiverse
Architectures: armhf
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
EOF
sudo rm -f /etc/apt/sources.list.d/armhf-ports.list

sudo apt-get -o Acquire::Retries=5 update
sudo apt-get -o Acquire::Retries=5 install -y \
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
  libjavascriptcoregtk-4.1-dev:armhf \
  libx11-dev:armhf \
  libxext-dev:armhf \
  libxrender-dev:armhf \
  libgdk-pixbuf-2.0-dev:armhf \
  x11proto-dev \
  shared-mime-info

{
  echo "CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc"
  echo "CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc"
  echo "CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++"
  echo "PKG_CONFIG_ALLOW_CROSS=1"
  echo "PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/share/pkgconfig"
  echo "BINDGEN_EXTRA_CLANG_ARGS=--target=arm-linux-gnueabihf -I/usr/include -I/usr/include/arm-linux-gnueabihf"
} >> "$GITHUB_ENV"
# PKG_CONFIG_LIBDIR replaces the default search path and hides /usr/share/pkgconfig
# (xproto.pc etc.). Leave it unset.

if ! PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig:/usr/share/pkgconfig pkg-config --exists gdk-3.0; then
  echo "::error::gdk-3.0.pc still missing after armhf sysroot setup"
  ls -la /usr/lib/arm-linux-gnueabihf/pkgconfig/gdk*.pc /usr/share/pkgconfig/xproto.pc || true
  exit 1
fi

echo "armhf cross toolchain ready ($(pkg-config --modversion libpipewire-0.3 2>/dev/null || echo unknown pipewire))"
