#!/bin/bash
set -u
echo "=== lost table ==="
grep -n "lost_and_found" /tmp/sn-recover.sql | head -5
echo "=== sample lost rows ==="
python3 - << 'PY'
from pathlib import Path
text = Path("/tmp/sn-recover.sql").read_text(errors="replace")
# print a few lost_and_found inserts that mention Soundboard
n=0
idx=0
while n<8:
    i=text.find("INSERT INTO lost_and_found", idx)
    if i<0: break
    j=text.find(";\n", i)
    row=text[i:j if j>0 else i+400]
    if "Soundboard" in row or "separator" in row.lower() or n<2:
        print(row[:500])
        print("---")
        n+=1
    idx=i+10
print("total marker", text.count("INSERT INTO lost_and_found"))
PY
