#!/bin/bash
set -u
echo "=== gcc ==="
command -v gcc cc 2>/dev/null || true
echo "=== cache grep ==="
# WebKit / tauri may have left the JS uncompressed in cache
cache=/home/steamos/.cache/com.soundninja.dev
du -sh "$cache" 2>/dev/null || true
# don't walk forever
find "$cache" -type f -size -20M 2>/dev/null | head -50
echo "=== rg ==="
if command -v rg >/dev/null; then
  rg -a -l "failed_sounds_path" "$cache" /home/steamos/.local/share/com.soundninja.dev 2>/dev/null | head
else
  grep -R -a -l "failed_sounds_path" "$cache" /home/steamos/.local/share/com.soundninja.dev 2>/dev/null | head
fi
echo "=== asset names in binary ==="
python3 - << 'PY'
p="/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/soundninja"
data=open(p,"rb").read()
print("bin", len(data))
idx=0
n=0
while n<30:
    i=data.find(b"/_nuxt/", idx)
    if i<0: break
    end=data.find(b"\x00", i)
    s=data[i:end if 0<end-i<180 else i+80]
    print(i, s[:120])
    n+=1
    idx=i+8
print("nuxt hits shown", n)
for key in (b"index.html", b"failed_sound", b".js"):
    print(key, data.find(key))
PY
