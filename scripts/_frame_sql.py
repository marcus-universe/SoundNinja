#!/usr/bin/env python3
"""Find failed_sounds_path SQL inside the running Sound Ninja binary."""
import os
p = "/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja"
data = open(p, "rb").read()
print("size", len(data))
needle = b"failed_sound"
idx = data.find(needle)
print("plain", idx)
# brotli streams often start near asset headers. Try import.
try:
    import brotli
except ImportError:
    brotli = None
    print("no brotli module")

if brotli is None:
    # scan for the utf-8 column name inside any zlib window
    import zlib
    hits = 0
    # Tauri assets are brotli. Look for 'SQL' nearby by walking 0x1b markers? skip.
    print("zlib scan start")
    i = 0
    n = len(data)
    while i < n - 2 and hits < 5:
        if data[i] == 0x78 and data[i + 1] in (0x01, 0x5E, 0x9C, 0xDA):
            try:
                out = zlib.decompress(data[i : i + 2_000_000])
            except Exception:
                i += 1
                continue
            j = out.find(needle)
            if j >= 0:
                print("ZLIB", i, out[max(0, j - 180) : j + 220])
                hits += 1
            i += 1000
        else:
            i += 1
    print("zlib hits", hits)
else:
    # brute a few offsets is too slow; search brotli by trying windows at asset table.
    print("brotli available")
