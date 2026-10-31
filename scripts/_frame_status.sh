#!/bin/bash
set -u
echo "=== proc ==="
ps -ef | awk '/soundninja|WebKitWebProcess/ && !/awk/ {print $2,$8,$9}'
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== files ==="
ls -la "$db" "$db.bak" "$db-wal" 2>/dev/null || true
echo "=== live col ==="
sqlite3 "$db" "SELECT name FROM pragma_table_info('sounds');" | tr '\n' ' '
echo
echo "=== bak col ==="
sqlite3 "$db.bak" "SELECT name FROM pragma_table_info('sounds');" | tr '\n' ' '
echo
echo "=== counts ==="
echo -n "live "; sqlite3 "$db" "SELECT COUNT(*) FROM sounds;"
echo -n "bak "; sqlite3 "$db.bak" "SELECT COUNT(*) FROM sounds;"
echo "=== window ==="
python3 - << 'PY'
import os, ctypes
os.environ["DISPLAY"]=":0"
x=ctypes.CDLL("libX11.so.6")
x.XOpenDisplay.restype=ctypes.c_void_p
d=x.XOpenDisplay(None)
print("display", "ok" if d else "no")
PY
