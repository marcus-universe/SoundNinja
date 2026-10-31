#!/bin/bash
set -u
APP=/home/steamos/.local/share/soundninja/SoundNinja-dev.AppImage
ls -la "$APP"
chmod +x "$APP"
# Keep the working 0.8.1 extract launcher.
if [ ! -f /home/steamos/.local/bin/soundninja.bak-extract-081 ]; then
  cp -a /home/steamos/.local/bin/soundninja /home/steamos/.local/bin/soundninja.bak-extract-081
fi
cat > /home/steamos/.local/bin/soundninja << 'EOF'
#!/bin/sh
exec /home/steamos/.local/share/soundninja/SoundNinja-dev.AppImage "$@"
EOF
chmod +x /home/steamos/.local/bin/soundninja
# Shell session has no desktop bus. Reuse the saved gamescope session.
set -a
# shellcheck disable=SC1091
. /home/steamos/.local/share/soundninja/session.env
set +a
export DISPLAY=:0
LOG=/home/steamos/.local/share/soundninja/launch-dev.log
: > "$LOG"
nohup /home/steamos/.local/bin/soundninja >>"$LOG" 2>&1 &
echo "pid $!"
disown || true
# Give WebKit time to paint or abort.
for i in 1 2 3 4 5 6 7 8 9 10 11 12; do
  sleep 1
  if grep -q EGL_BAD_PARAMETER "$LOG" 2>/dev/null; then
    echo "EGL_ABORT"
    break
  fi
done
echo "=== log ==="
wc -c "$LOG"
cat "$LOG"
echo "=== procs ==="
ps -ef | awk '/SoundNinja-dev|soundninja|WebKitWebProcess/ && !/awk/ {print $2,$8,$9,$10}'
echo "=== window ==="
DISPLAY=:0 xwininfo -root -tree | awk '/"Sound Ninja"/ {print; exit}'
