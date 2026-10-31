#!/bin/bash
set -u
db=/tmp/sn-fixed.sninja
echo "=== counts ==="
sqlite3 "$db" "SELECT 'sounds', COUNT(*) FROM sounds UNION ALL SELECT 'links', COUNT(*) FROM sound_tabs UNION ALL SELECT 'tabs', COUNT(*) FROM tabs UNION ALL SELECT 'gifs', COUNT(*) FROM gif_blobs UNION ALL SELECT 'seps', COUNT(*) FROM separators;"
echo "=== per tab ==="
sqlite3 -header -column "$db" "SELECT tab, COUNT(*) AS n FROM sound_tabs WHERE tab != 'All' GROUP BY tab ORDER BY n DESC;"
echo "=== integrity ==="
sqlite3 "$db" "PRAGMA integrity_check;" | head -3
echo "=== sample missing ==="
python3 - << 'PY'
import sqlite3
from pathlib import Path
db=sqlite3.connect("/tmp/sn-fixed.sninja")
missing=0
total=0
for (p,) in db.execute("SELECT path FROM sounds"):
    total+=1
    if not Path(p).is_file():
        missing+=1
        if missing<=8:
            print("MISS", p)
print("total", total, "missing", missing)
PY
