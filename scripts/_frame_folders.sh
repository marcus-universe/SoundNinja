#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== folders ==="
sqlite3 "$db" "SELECT path FROM sounds;" | awk -F/ '{
  n=split($0,a,"/");
  if(n>=2) c[a[n-1]]++; else c["(none)"]++;
} END {for (k in c) print c[k], k}' | sort -nr
echo "=== sample paths ==="
sqlite3 "$db" "SELECT path FROM sounds LIMIT 8;"
echo "=== tabs with zero links ==="
sqlite3 "$db" "SELECT t.name FROM tabs t LEFT JOIN sound_tabs s ON s.tab=t.name WHERE s.tab IS NULL;"
echo "=== other sninja ==="
find /home/steamos -name '*.sninja*' -o -name '*.bak' 2>/dev/null | head -80
echo "=== snapshots ==="
ls /home/steamos/.snapshots 2>/dev/null | head
ls /.snapshots 2>/dev/null | head
btrfs subvolume list / 2>/dev/null | head
echo "=== log theme ==="
tail -c 4000 /home/steamos/.local/share/soundninja/launch-dev.log 2>/dev/null || true
echo "=== procs ==="
ps -ef | awk '/SoundNinja-dev|WebKit/ && !/awk/{print $2,$8}'
