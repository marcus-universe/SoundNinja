#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
md5sum "$db" "$db.bak"
echo -n "live sounds "; sqlite3 "$db" "SELECT COUNT(*) FROM sounds;"
echo -n "bak sounds "; sqlite3 "$db.bak" "SELECT COUNT(*) FROM sounds;"
ps -ef | awk '/SoundNinja-dev|WebKitWebProcess/ && !/awk/{print $2,$8}'
