#!/bin/bash
set -u
echo "=== stderr ==="
ls -l /proc/24543/fd/1 /proc/24543/fd/2 2>/dev/null || true
echo "=== steam log hits ==="
grep -a -n "then is not\|theme-creator\|Theme Creator\|500" /home/steamos/.local/share/Steam/logs/steam_output.log | tail -20
echo "=== lost sample ==="
python3 - << 'PY'
from pathlib import Path
p=Path("/tmp/sn-recover.sql")
text=p.read_text(errors="replace")
i=text.find("CREATE TABLE lost_and_found")
print(text[i:i+500])
print("---ROWS---")
# first insert
j=text.find("INSERT INTO lost_and_found")
print(text[j:j+800])
print("---SOUND---")
k=text.find("Soundboard\\Music_1")
if k<0: k=text.find("Soundboard/Music_1")
print("idx", k)
print(text[max(0,k-200):k+300])
PY
