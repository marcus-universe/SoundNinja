#!/bin/bash
set -u
# Leftover process from before the restore. It no longer holds the project file.
kill 24539 2>/dev/null || true
sleep 1
ps -ef | awk '/soundninja|SoundNinja/ && !/awk/{print $2,$8,$9}'
