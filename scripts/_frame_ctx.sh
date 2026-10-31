#!/bin/bash
set -u
echo "=== recover head ==="
head -c 1500 /tmp/sn-recover.sql
echo
echo "=== insert verbs ==="
grep -o "INSERT[^;]\{0,40\}" /tmp/sn-recover.sql | awk '{print $1,$2,$3}' | sort | uniq -c | sort -nr | head
echo "=== music_1 in recover sql ==="
grep -c "Music_1" /tmp/sn-recover.sql || true
echo "=== context in db ==="
python3 - << 'PY'
from pathlib import Path
db=Path("/home/steamos/Music/Soundboard/260912_Soundboard.sninja").read_bytes()
for n in [b"Music_1/", b"Lollis/", b"Meru/", b"MusicMeme/"]:
    i=db.find(n)
    chunk=db[max(0,i-80):i+120]
    text=''.join(chr(b) if 32<=b<127 else '.' for b in chunk)
    print("---", n, i)
    print(text)
PY
