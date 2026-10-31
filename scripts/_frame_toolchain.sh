#!/bin/bash
set -u
echo "node $(command -v node || echo no) $(node -v 2>/dev/null || true)"
echo "npm $(command -v npm || echo no)"
echo "cargo $(command -v cargo || echo no)"
echo "rustc $(rustc --version 2>/dev/null || echo no)"
echo "pkg-config $(pkg-config --exists webkit2gtk-4.1 && echo webkit-yes || echo webkit-no)"
echo "pnpm $(command -v pnpm || echo no)"
ls /home/steamos/.cargo/bin 2>/dev/null | head
