#!/bin/bash
set -u
echo "=== files ==="
ls -la /home/steamos/Music/Soundboard/ /home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/projects/New_Project/ /home/steamos/.local/share/com.soundninja.dev/projects/Default/
echo "=== config keys ==="
sqlite3 /home/steamos/.local/share/com.soundninja.dev/app-config.db "SELECT key, substr(value,1,200) FROM settings;"
echo "=== recent ==="
sqlite3 /home/steamos/.local/share/com.soundninja.dev/app-config.db ".tables"
echo "=== journal ==="
journalctl --user -n 30 --no-pager -q 2>/dev/null | grep -i -E 'soundninja|sqlite|save|error' | tail -20 || true
echo "=== proc fd ==="
pid=$(pgrep -n -f 'squashfs-root/AppRun|soundninja' || true)
echo pid=$pid
if [ -n "$pid" ]; then
  ls -l /proc/$pid/fd 2>/dev/null | awk '/sninja|sqlite|project/ {print}' | head
  tr '\0' '\n' < /proc/$pid/cmdline; echo
fi
