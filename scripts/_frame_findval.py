#!/usr/bin/env python3
p = "/tmp/.mount_SoundNBINofb/usr/bin/soundninja"
data = open(p, "rb").read()
for s in [b"$value", b"Internal Server Error", b".then(e", b"theme-creator"]:
    print(s, data.find(s))
