#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== LIVE master ==="
sqlite3 "$db" "SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name;"
echo "=== BAK master (triggers/views only) ==="
sqlite3 "$db.bak" "SELECT type, name, tbl_name FROM sqlite_master WHERE type!='table';"
echo "=== BAK sql mentioning fail ==="
sqlite3 "$db.bak" "SELECT sql FROM sqlite_master WHERE sql LIKE '%fail%';"
echo "=== LIVE sql mentioning fail ==="
sqlite3 "$db" "SELECT sql FROM sqlite_master WHERE sql LIKE '%fail%';"
