#!/usr/bin/env python3
"""Summarize sqlite .recover lost_and_found rows."""
import re
from collections import Counter
from pathlib import Path

text = Path("/tmp/sn-recover.sql").read_text(errors="replace")
rows = re.findall(r"INSERT INTO lost_and_found VALUES\((.*?)\);", text)
print("rows", len(rows))
nf = Counter()
samples = {}
for raw in rows:
    # split on commas not inside quotes — fields are simple enough: use csv
    # The SQL uses single quotes. Do a tiny parser.
    fields = []
    i = 0
    n = len(raw)
    while i < n:
        if raw[i] == "'":
            i += 1
            buf = []
            while i < n:
                if raw[i] == "'" and i + 1 < n and raw[i + 1] == "'":
                    buf.append("'")
                    i += 2
                    continue
                if raw[i] == "'":
                    i += 1
                    break
                buf.append(raw[i])
                i += 1
            fields.append("".join(buf))
            if i < n and raw[i] == ",":
                i += 1
            continue
        j = raw.find(",", i)
        if j < 0:
            fields.append(raw[i:].strip())
            break
        fields.append(raw[i:j].strip())
        i = j + 1
    # root, pg, nfield, id, c0...
    if len(fields) < 4:
        continue
    nfield = fields[2]
    nf[nfield] += 1
    samples.setdefault(nfield, fields[:12])

print("nfield counts", nf.most_common())
for k, sample in samples.items():
    print("SAMPLE nfield", k)
    for idx, val in enumerate(sample):
        s = val.replace("\n", " ")
        if len(s) > 140:
            s = s[:140] + "..."
        print(f"  {idx}: {s}")
