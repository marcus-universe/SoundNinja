#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== checkpoint ==="
sqlite3 "$db" "PRAGMA wal_checkpoint(TRUNCATE);"
stat -c '%y %s' "$db" "$db-wal" "$db.bak"
echo -n "sounds "; sqlite3 "$db" "SELECT COUNT(*) FROM sounds;"
echo -n "col "; sqlite3 "$db" "SELECT COUNT(*) FROM pragma_table_info('sounds') WHERE name='failed_sounds_path';"
echo
sqlite3 "$db" "PRAGMA integrity_check;" | head -1
