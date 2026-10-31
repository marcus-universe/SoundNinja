#!/bin/bash
set -u
ps -ef | awk '/soundninja|SoundNinja/ && !/awk/{print $2,$3,$8,$9}'
echo "=== fds on db ==="
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
for pid in $(ps -eo pid,cmd | awk '/soundninja|SoundNinja|WebKitWebProcess/ && !/awk/{print $1}'); do
  if ls -l /proc/$pid/fd 2>/dev/null | grep -q "260912_Soundboard"; then
    echo "pid $pid holds db"
    ps -o pid,lstart,cmd -p $pid
  fi
done
