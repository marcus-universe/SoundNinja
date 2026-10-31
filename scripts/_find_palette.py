from pathlib import Path
import struct, zlib

data = Path(r"scripts/_frame_shot.png").read_bytes()
# decode via a tiny approach: use Pillow if present
try:
    from PIL import Image
except ImportError:
    Image = None
if Image:
    im = Image.open(r"scripts/_frame_shot.png").convert("RGB")
    w, h = im.size
    print("size", w, h)
    # look for highly saturated pixels in the upper-middle band
    hits = []
    px = im.load()
    for y in range(40, 200):
        for x in range(200, 560):
            r, g, b = px[x, y]
            if r > 180 and b > 140 and g < 120:
                hits.append((x, y))
    print("pink", len(hits))
    from collections import Counter
    ys = Counter(y for _, y in hits)
    # cluster rows that have pink
    rows = sorted(ys)
    bands = []
    start = prev = None
    for y in rows:
        if start is None:
            start = prev = y
            continue
        if y - prev > 4:
            bands.append((start, prev))
            start = y
        prev = y
    if start is not None:
        bands.append((start, prev))
    for a, b in bands:
        pts = [(x, y) for x, y in hits if a <= y <= b]
        xs = [p[0] for p in pts]
        print(f"band y {a}-{b} n={len(pts)} x {min(xs)}-{max(xs)} center {(min(xs)+max(xs))//2} {(a+b)//2} screen {((min(xs)+max(xs))//2)*3} {((a+b)//2)*3}")
else:
    print("no pillow")
