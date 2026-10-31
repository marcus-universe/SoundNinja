#!/bin/bash
set -u
log=/home/steamos/.local/share/Steam/logs/steam_output.log
echo "log bytes $(wc -c < "$log")"
# last occurrences
grep -n -a "failed_sound\|Failed to persist\|no such column" "$log" | tail -40
echo "=== tail context ==="
# show 2 lines around the last match via awk
awk 'BEGIN{n=0} /failed_sound|Failed to persist|no such column/{n=NR} {lines[NR]=$0} END{s=(n>3?n-3:1); for(i=s;i<=n+2 && i<=NR;i++) print i": "lines[i]}' "$log" | tail -20
