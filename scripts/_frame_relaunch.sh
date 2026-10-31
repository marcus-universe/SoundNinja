#!/bin/bash
set -u
APP=/home/steamos/.local/share/soundninja/SoundNinja-dev2.AppImage
chmod +x "$APP"
ls -la "$APP"
cat > /home/steamos/.local/bin/soundninja << 'EOF'
#!/bin/sh
exec /home/steamos/.local/share/soundninja/SoundNinja-dev2.AppImage "$@"
EOF
chmod +x /home/steamos/.local/bin/soundninja
# Restart onto the new image. Project file stays.
kill 36675 36680 2>/dev/null || true
sleep 1
if pgrep -f SoundNinja-dev >/dev/null; then
  pkill -9 -f SoundNinja-dev || true
  sleep 1
fi
set -a
. /home/steamos/.local/share/soundninja/session.env
set +a
export DISPLAY=:0
LOG=/home/steamos/.local/share/soundninja/launch-dev2.log
: > "$LOG"
nohup /home/steamos/.local/bin/soundninja >>"$LOG" 2>&1 &
echo "pid $!"
disown || true
sleep 5
echo "=== log ==="
cat "$LOG"
ps -ef | awk '/SoundNinja-dev2|WebKitWebProcess/ && !/awk/{print $2,$8}'
DISPLAY=:0 xwininfo -root -tree | awk '/"Sound Ninja"|"Theme Creator"/ {print}'
sqlite3 /home/steamos/Music/Soundboard/260912_Soundboard.sninja "SELECT COUNT(*) FROM sounds;"
