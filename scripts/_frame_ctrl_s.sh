#!/bin/bash
set -u
db=/home/steamos/Music/Soundboard/260912_Soundboard.sninja
echo "=== before ==="
stat -c '%y %s' "$db" "$db.bak" 2>/dev/null || true
sqlite3 "$db" "SELECT name FROM pragma_table_info('sounds') WHERE name='failed_sounds_path';"
python3 - << 'PY'
import ctypes, os, time
os.environ["DISPLAY"] = ":0"
x11 = ctypes.CDLL("libX11.so.6")
xtst = ctypes.CDLL("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XStringToKeysym.restype = ctypes.c_ulong
x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XKeysymToKeycode.restype = ctypes.c_uint
xtst.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
x11.XFlush.argtypes = [ctypes.c_void_p]
x11.XRaiseWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XSetInputFocus.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
dpy = x11.XOpenDisplay(None)
wid = 0x3c00003
x11.XRaiseWindow(dpy, wid)
x11.XSetInputFocus(dpy, wid, 1, 0)
x11.XFlush(dpy)
time.sleep(0.3)

def tap(name, down):
    code = x11.XKeysymToKeycode(dpy, x11.XStringToKeysym(name.encode()))
    xtst.XTestFakeKeyEvent(dpy, code, 1 if down else 0, 0)
    x11.XFlush(dpy)

tap("Control_L", True)
time.sleep(0.08)
tap("s", True)
time.sleep(0.08)
tap("s", False)
tap("Control_L", False)
print("sent ctrl+s")
PY
sleep 4
echo "=== after ==="
stat -c '%y %s' "$db" "$db.bak" "$db-wal" 2>/dev/null || true
sqlite3 "$db" "PRAGMA integrity_check;" | head -1
echo -n "sounds "; sqlite3 "$db" "SELECT COUNT(*) FROM sounds;"
echo -n "col "; sqlite3 "$db" "SELECT name FROM pragma_table_info('sounds') WHERE name='failed_sounds_path';"
echo
# bak should now include the column too if persist copied the migrated file first
echo -n "bak col "; sqlite3 "$db.bak" "SELECT name FROM pragma_table_info('sounds') WHERE name='failed_sounds_path';"
echo
