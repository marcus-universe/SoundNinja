#!/usr/bin/env python3
import os, struct, subprocess, zlib
from pathlib import Path
os.environ["DISPLAY"] = ":0"

def u32(buf, i, be):
    return struct.unpack_from(">I" if be else "<I", buf, i)[0]

wid = subprocess.check_output(
    ["bash", "-lc", """DISPLAY=:0 xwininfo -root -tree | awk '/"Theme Creator"/ {print $1; exit}'"""],
    text=True,
).strip()
print("wid", wid)
data = subprocess.check_output(["xwd", "-silent", "-id", wid])
be = struct.unpack_from(">I", data, 0)[0] < 4096
vals = [u32(data, i * 4, be) for i in range(25)]
header_size, ncolors = vals[0], vals[19]
width, height, bpp, bpl = vals[4], vals[5], vals[11], vals[12]
red_mask, green_mask, blue_mask = vals[14], vals[15], vals[16]
offset = header_size + ncolors * 12

def shift(mask):
    s = 0
    while mask and (mask & 1) == 0:
        mask >>= 1
        s += 1
    return s
rs, gs, bs = shift(red_mask), shift(green_mask), shift(blue_mask)
step = bpp // 8
rows = []
scale = 3
for y in range(0, height, scale):
    row = data[offset + y * bpl : offset + (y + 1) * bpl]
    px = bytearray()
    for x in range(0, width, scale):
        chunk = row[x * step : x * step + step]
        v = int.from_bytes(chunk[:4].ljust(4, b"\0"), "big" if be else "little")
        px += bytes(((v & red_mask) >> rs, (v & green_mask) >> gs, (v & blue_mask) >> bs))
    rows.append(bytes(px))
w, h = width // scale, len(rows)

def chunk(tag, payload):
    return struct.pack(">I", len(payload)) + tag + payload + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
raw = b"".join(b"\x00" + r for r in rows)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b"")
Path("/tmp/sn-theme.png").write_bytes(png)
print(f"WINDOW {wid} {w}x{h}")
