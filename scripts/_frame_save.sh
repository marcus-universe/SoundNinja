#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== alter ==="
sqlite3 "$db" "ALTER TABLE sounds ADD COLUMN failed_sounds_path TEXT;" && echo ADDED || echo "alter_failed $?"
sqlite3 "$db" "PRAGMA table_info(sounds);" | awk -F'|' '$2=="failed_sounds_path"{print "has",$2}'
# Dismiss the error, then save.
python3 - << 'PY'
import ctypes, os, time
os.environ["DISPLAY"] = ":0"
x11 = ctypes.CDLL("libX11.so.6")
xtst = ctypes.CDLL("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XStringToKeysym.restype = ctypes.c_ulong
x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
xtst.XTestFakeMotionEvent.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong]
xtst.XTestFakeButtonEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
xtst.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
x11.XFlush.argtypes = [ctypes.c_void_p]
dpy = x11.XOpenDisplay(None)

def click(x, y):
    xtst.XTestFakeMotionEvent(dpy, -1, x, y, 0)
    x11.XFlush(dpy)
    time.sleep(0.1)
    xtst.XTestFakeButtonEvent(dpy, 1, 1, 0)
    x11.XFlush(dpy)
    time.sleep(0.08)
    xtst.XTestFakeButtonEvent(dpy, 1, 0, 0)
    x11.XFlush(dpy)
    time.sleep(0.3)

def tap(name, down=True):
    code = x11.XKeysymToKeycode(dpy, x11.XStringToKeysym(name.encode()))
    xtst.XTestFakeKeyEvent(dpy, code, 1 if down else 0, 0)
    x11.XFlush(dpy)

# OK sits in the lower middle of the error card.
click(960, 690)
time.sleep(0.4)
tap("Control_L", True)
time.sleep(0.05)
tap("s", True)
time.sleep(0.05)
tap("s", False)
tap("Control_L", False)
print("sent ctrl+s")
PY
sleep 2
echo "=== after ==="
ls -la "$db" "$db-wal" "$db.bak" 2>/dev/null
sqlite3 "$db" "PRAGMA integrity_check;" | head -3
sqlite3 "$db" "SELECT COUNT(*) FROM sounds;"
