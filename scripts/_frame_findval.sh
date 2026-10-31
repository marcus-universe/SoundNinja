#!/bin/bash
set -u
bin=/tmp/.mount_SoundNBINofb/usr/bin/soundninja
python3 - << PY
p="$bin"
data=open(p,"rb").read()
for s in [b"\$value", b"then is not", b"Internal Server Error", b"b0.then"]:
    print(s, data.find(s))
PY
