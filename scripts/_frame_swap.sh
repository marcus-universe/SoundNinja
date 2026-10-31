#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
sqlite3 "$db" "PRAGMA wal_checkpoint(TRUNCATE);" || true
cp -a "$db" /home/steamos/Music/Soundboard/260912_Soundboard.before-restore.sninja
# Close Sound Ninja so the file replace is not fighting an open WAL.
pkill -f SoundNinja-dev.AppImage || true
sleep 1
if pgrep -f SoundNinja-dev.AppImage >/dev/null; then
  pkill -9 -f SoundNinja-dev.AppImage || true
  sleep 1
fi
rm -f "$db-wal" "$db-shm"
cp -a /tmp/sn-fixed.sninja "$db"
chmod 755 "$db"
echo "=== installed ==="
sqlite3 "$db" "SELECT 'sounds', COUNT(*) FROM sounds UNION ALL SELECT 'tabs', COUNT(*) FROM tabs;"
sqlite3 "$db" "PRAGMA integrity_check;" | head -1
# Relaunch into the gamescope session.
set -a
. /home/steamos/.local/share/soundninja/session.env
set +a
export DISPLAY=:0
LOG=/home/steamos/.local/share/soundninja/launch-dev.log
: > "$LOG"
nohup /home/steamos/.local/bin/soundninja >>"$LOG" 2>&1 &
echo "pid $!"
disown || true
sleep 4
ps -ef | awk '/SoundNinja-dev|WebKitWebProcess/ && !/awk/{print $2,$8}'
DISPLAY=:0 xwininfo -root -tree | awk '/"Sound Ninja"/ {print; exit}'
