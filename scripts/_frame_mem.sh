#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== hash ==="
md5sum "$db" "$db.bak"
echo "=== schema cookie ==="
sqlite3 "$db" "PRAGMA schema_version; PRAGMA user_version;"
echo "=== cols ==="
sqlite3 "$db" "SELECT name FROM pragma_table_info('sounds') WHERE name LIKE '%fail%' OR name LIKE '%path%';"
echo "=== procs ==="
ps -ef | awk '/soundninja|WebKit/ && !/awk/ {print}'
echo "=== memscan ==="
python3 - << 'PY'
import os, re
needle = b"failed_sounds_path"
# scan readable maps of soundninja + webkit children
pids = []
for pid in os.listdir("/proc"):
    if not pid.isdigit():
        continue
    try:
        cmd = open(f"/proc/{pid}/cmdline", "rb").read().replace(b"\x00", b" ")
    except Exception:
        continue
    if b"soundninja" in cmd or b"WebKit" in cmd:
        pids.append((pid, cmd[:120]))
print("targets", pids)
pat = re.compile(rb".{0,120}failed_sounds_path.{0,160}")
for pid, cmd in pids:
    try:
        maps = open(f"/proc/{pid}/maps").read().splitlines()
    except Exception as e:
        print("maps fail", pid, e)
        continue
    hits = 0
    try:
        mem = open(f"/proc/{pid}/mem", "rb", buffering=0)
    except Exception as e:
        print("mem fail", pid, e)
        continue
    for line in maps:
        parts = line.split()
        if "r" not in parts[1]:
            continue
        start_s, end_s = parts[0].split("-")
        start, end = int(start_s, 16), int(end_s, 16)
        size = end - start
        if size <= 0 or size > 64 * 1024 * 1024:
            continue
        try:
            mem.seek(start)
            blob = mem.read(size)
        except Exception:
            continue
        if needle not in blob:
            continue
        for m in pat.finditer(blob):
            s = m.group().decode("utf-8", "replace")
            print(f"HIT pid={pid} {s}")
            hits += 1
            if hits >= 8:
                break
        if hits >= 8:
            break
    print(f"pid {pid} hits {hits}")
    mem.close()
PY
