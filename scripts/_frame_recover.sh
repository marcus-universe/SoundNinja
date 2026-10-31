#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== pages ==="
sqlite3 "$db" "PRAGMA page_count; PRAGMA freelist_count; PRAGMA page_size;"
echo "=== gifs ==="
sqlite3 "$db" "SELECT COUNT(*), COALESCE(SUM(byte_len),0) FROM gif_blobs;"
echo "=== string hits in live ==="
for s in Music_1 Meru MusicMeme "/Sounds/" Lollis German; do
  printf '%s ' "$s"
  grep -a -o -F "$s" "$db" | wc -l
done
echo "=== string hits in bak ==="
for s in Music_1 Meru MusicMeme "/Sounds/" Lollis; do
  printf '%s ' "$s"
  grep -a -o -F "$s" "$db.bak" | wc -l
done
echo "=== recover copy ==="
cp -a "$db" /tmp/sn-recover.sninja
rm -f /tmp/sn-recover.sninja-wal /tmp/sn-recover.sninja-shm
sqlite3 /tmp/sn-recover.sninja ".recover" > /tmp/sn-recover.sql
wc -c /tmp/sn-recover.sql
echo "=== recovered insert counts ==="
grep -c "INSERT INTO sounds" /tmp/sn-recover.sql || true
grep -c "INSERT INTO sound_tabs" /tmp/sn-recover.sql || true
grep -c "INSERT INTO separators" /tmp/sn-recover.sql || true
echo "=== sample missing path still in file? ==="
# pick one missing file name
python3 - << 'PY'
import pathlib
db=pathlib.Path("/home/steamos/Music/Soundboard/260912_Soundboard.sninja").read_bytes()
bak=pathlib.Path("/home/steamos/Music/Soundboard/260912_Soundboard.sninja.bak").read_bytes()
needles=[b"Music_1/", b"Meru/", b"MusicMeme/", b"Soundboard/Sounds/", b"Lollis/"]
for n in needles:
    print(n, "live", db.find(n), "bak", bak.find(n))
PY
