#!/bin/bash
set -u
pid=407424
echo "exe $(readlink /proc/$pid/exe 2>/dev/null || echo dead)"
tr '\0' '\n' < /proc/$pid/cmdline; echo
echo "=== cwd ==="
readlink /proc/$pid/cwd
echo "=== strings hit ==="
exe=$(readlink /proc/$pid/exe)
if [ -n "$exe" ]; then
  grep -a -o 'failed_sounds_path' "$exe" | head
  # also search mounted libs / resources next to exe
  root=$(dirname "$(dirname "$exe")")
  echo "root $root"
  grep -a -l 'failed_sounds_path' "$root"/share/* "$exe" 2>/dev/null | head
fi
