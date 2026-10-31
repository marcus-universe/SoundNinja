#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== schema ==="
sqlite3 "$db" ".schema" | head -200
echo "=== pragma sounds ==="
sqlite3 "$db" "PRAGMA table_info(sounds);"
echo "=== binary snippet ==="
exe=/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja
python3 - << 'PY'
import pathlib
data = pathlib.Path("/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja").read_bytes()
idx = data.find(b"failed_sound")
print("idx", idx)
if idx < 0:
    # try utf16
    needle = "failed_sounds_path".encode("utf-16le")
    print("utf16", data.find(needle))
else:
    print(data[max(0,idx-80):idx+80])
PY
