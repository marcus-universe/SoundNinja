#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== settings keys ==="
sqlite3 "$db" "SELECT key, substr(value,1,200) FROM settings ORDER BY key;"
echo "=== sound_tabs detail ==="
sqlite3 -header -column "$db" "SELECT tab, tab_index, substr(sound_path, length(sound_path)-40) p FROM sound_tabs WHERE tab!='All' ORDER BY tab, tab_index;"
echo "=== windows ==="
DISPLAY=:0 xwininfo -root -tree | awk '/Sound|Theme|soundninja|Error|500/' 
echo "=== web procs ==="
ps -ef | awk '/WebKitWebProcess/ && !/awk/{print}'
echo "=== default project counts ==="
d=/home/steamos/.local/share/com.soundninja.dev/projects/Default/project.sninja
sqlite3 "$d" "SELECT 'tabs', COUNT(*) FROM tabs UNION ALL SELECT 'sounds', COUNT(*) FROM sounds UNION ALL SELECT 'st', COUNT(*) FROM sound_tabs UNION ALL SELECT 'sep', COUNT(*) FROM separators;" 2>/dev/null || echo no-default
echo "=== new project ==="
n=/home/steamos/.local/share/soundninja/extract-081/squashfs-root/usr/bin/projects/New_Project/project.sninja
sqlite3 "$n" "SELECT 'tabs', COUNT(*) FROM tabs UNION ALL SELECT 'sounds', COUNT(*) FROM sounds UNION ALL SELECT 'st', COUNT(*) FROM sound_tabs UNION ALL SELECT 'sep', COUNT(*) FROM separators;" 2>/dev/null || echo no-new
