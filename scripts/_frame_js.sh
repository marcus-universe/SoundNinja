#!/bin/bash
set -u
root=/home/steamos/.local/share/soundninja/extract-081/squashfs-root
find "$root" -name '*.js' -o -name 'index.html' -o -name '*.css' | head -40
echo "=== share ==="
ls "$root/usr/share" 2>/dev/null | head
echo "=== bin size ==="
ls -lh "$root/usr/bin/soundninja"
