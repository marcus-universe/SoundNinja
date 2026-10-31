#!/bin/bash
set -u
echo "=== parents ==="
ps -o pid,ppid,cmd -p 24631,24834,36680,36763,36675 2>/dev/null || true
# Old web processes whose parent is not the new app.
for pid in 24631 24834; do
  if [ -d /proc/$pid ]; then
    ppid=$(ps -o ppid= -p $pid | tr -d ' ')
    cmd=$(ps -o cmd= -p $ppid 2>/dev/null || true)
    echo "old $pid parent $ppid $cmd"
    case "$cmd" in
      *SoundNinja-dev*) ;;
      *) kill "$pid" 2>/dev/null || true ;;
    esac
  fi
done
sleep 1
echo "=== left ==="
ps -ef | awk '/SoundNinja-dev|WebKitWebProcess/ && !/awk/{print $2,$3,$8}'
