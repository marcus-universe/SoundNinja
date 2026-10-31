#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== db sounds cols ==="
sqlite3 "$db" "PRAGMA table_info(sounds);"
echo "=== bak sounds cols ==="
sqlite3 "$db.bak" "PRAGMA table_info(sounds);"
echo "=== db tables ==="
sqlite3 "$db" ".tables"
echo "=== count ==="
echo -n "db "; sqlite3 "$db" "SELECT COUNT(*) FROM sounds;"
echo -n "bak "; sqlite3 "$db.bak" "SELECT COUNT(*) FROM sounds;"
echo "=== log tails ==="
ls -lt /tmp/*sound* /home/steamos/.local/share/soundninja/*.log /home/steamos/.local/share/com.soundninja.dev/*.log 2>/dev/null | head
# recent stderr of the process if any
ls -l /proc/407424/fd 2>/dev/null | head -5
tr '\0' '\n' < /proc/407424/environ 2>/dev/null | awk 'BEGIN{c=0} {c++} END{print "envlines",c}'
