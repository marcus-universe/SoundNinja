#!/bin/bash
set -u
echo "=== tools ==="
command -v rustc || ls -l "$HOME/.cargo/bin/rustc" 2>/dev/null || true
command -v python3
python3 -c "import brotli" 2>/dev/null && echo has_brotli || echo no_brotli
find /home/steamos/.local/share/soundninja/extract-081/squashfs-root -iname '*brotli*' 2>/dev/null | head
ldconfig -p 2>/dev/null | grep -i brotli || true
echo "=== cache ==="
ls -d /home/steamos/.cache/*sound* /home/steamos/.cache/webkit* 2>/dev/null | head
echo "=== integrity ==="
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
sqlite3 "$db" "PRAGMA integrity_check;" | head -2
echo -n "names "; sqlite3 "$db" "SELECT COUNT(*), SUM(length(name)) FROM sounds;"
echo -n "bak names "; sqlite3 "$db.bak" "SELECT COUNT(*), SUM(length(name)) FROM sounds;"
echo "=== path diff ==="
sqlite3 "$db" "SELECT path FROM sounds ORDER BY path;" > /tmp/sn_db_paths
sqlite3 "$db.bak" "SELECT path FROM sounds ORDER BY path;" > /tmp/sn_bak_paths
diff -q /tmp/sn_db_paths /tmp/sn_bak_paths && echo PATHS_MATCH
