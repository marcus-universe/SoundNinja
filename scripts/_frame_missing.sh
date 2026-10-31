#!/bin/bash
set -u
echo "=== soundboard tree ==="
find /home/steamos/Music/Soundboard -type d | head -80
echo "=== audio counts per dir ==="
find /home/steamos/Music/Soundboard -type f \( -iname '*.wav' -o -iname '*.mp3' -o -iname '*.ogg' -o -iname '*.flac' \) | awk -F/ '{
  n=split($0,a,"/");
  c[a[n-1]]++;
  tot++;
} END {print "files", tot; for (k in c) print c[k], k}' | sort -nr
echo "=== db path count ==="
sqlite3 /home/steamos/Music/Soundboard/260912_Soundboard.sninja "SELECT COUNT(*) FROM sounds;"
echo "=== in folder not in db ==="
sqlite3 /home/steamos/Music/Soundboard/260912_Soundboard.sninja "SELECT path FROM sounds;" | sort > /tmp/sn_dbp
find /home/steamos/Music/Soundboard -type f \( -iname '*.wav' -o -iname '*.mp3' -o -iname '*.ogg' -o -iname '*.flac' \) | sort > /tmp/sn_fsp
echo "folder not db:"
comm -23 /tmp/sn_fsp /tmp/sn_dbp | head -40
echo "count missing $(comm -23 /tmp/sn_fsp /tmp/sn_dbp | wc -l)"
echo "db not folder:"
comm -13 /tmp/sn_fsp /tmp/sn_dbp | head -20
