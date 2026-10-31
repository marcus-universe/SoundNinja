#!/bin/bash
set -u
echo "=== procs ==="
ps -u steamos -o pid,stat,cmd | awk '/[Ss]ound[Nn]inja|[Aa]pp[Rr]un/ && !/awk/ {print}'
echo "=== logs ==="
ls -lt /home/steamos/.local/share/soundninja/*.log /tmp/sn*.log 2>/dev/null | head
echo "=== launch-fixed tail ==="
tail -c 4000 /home/steamos/.local/share/soundninja/launch-fixed.log 2>/dev/null || true
echo
echo "=== projects ==="
find /home/steamos -name '*.sninja' -o -name 'app-config.db' 2>/dev/null | head -40
echo "=== appdata ==="
ls -la /home/steamos/.local/share/com.soundninja.dev 2>/dev/null | head
