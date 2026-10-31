#!/bin/bash
set -u
echo "=== launcher ==="
sed -n '1,80p' /home/steamos/.local/bin/soundninja 2>/dev/null || echo NO_LAUNCHER
echo "=== desktop ==="
sed -n '1,30p' /home/steamos/.local/share/applications/soundninja.desktop 2>/dev/null || true
echo "=== last project ==="
sqlite3 /home/steamos/.local/share/com.soundninja.dev/app-config.db "SELECT key, value FROM settings WHERE key='lastProjectPath';" 2>/dev/null || echo no-app-config
echo "=== disk ==="
df -h /home/steamos | tail -1
echo "=== procs ==="
ps -ef | awk '/soundninja|WebKitWebProcess/ && !/awk/{print}'
