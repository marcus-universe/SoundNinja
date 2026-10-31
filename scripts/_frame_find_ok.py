#!/usr/bin/env python3
import struct, zlib
from pathlib import Path
data = Path(r"h:\Projects\Coding\SoundNinja\scripts\_frame_shot.png").read_bytes()
pos = 8
idat = b""
while pos < len(data):
    n = struct.unpack(">I", data[pos:pos+4])[0]
    tag = data[pos+4:pos+8]
    payload = data[pos+8:pos+8+n]
    pos += 12 + n
    if tag == b"IHDR":
        w, h = struct.unpack(">II", payload[:8])
    elif tag == b"IDAT":
        idat += payload
    elif tag == b"IEND":
        break
raw = zlib.decompress(idat)
stride = w * 3 + 1
rows = [raw[y*stride+1:(y+1)*stride] for y in range(h)]
# Magenta OK button: high R and B, low G, a compact blob near center.
hits = []
for y in range(h):
    xs = []
    for x in range(w):
        i = x*3
        r, g, b = rows[y][i], rows[y][i+1], rows[y][i+2]
        if r > 180 and b > 160 and g < 100 and r > g + 80:
            xs.append(x)
    if len(xs) > 15:
        hits.append((y, min(xs), max(xs), len(xs)))
print("w,h", w, h, "rows", len(hits))
for item in hits:
    print(item)
if hits:
    y0, y1 = hits[0][0], hits[-1][0]
    x0 = min(t[1] for t in hits)
    x1 = max(t[2] for t in hits)
    cx, cy = (x0+x1)/2, (y0+y1)/2
    print(f"crop {cx:.0f},{cy:.0f}")
    print(f"screen {cx*3:.0f},{cy*3:.0f}")
