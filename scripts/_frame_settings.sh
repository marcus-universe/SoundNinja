#!/bin/bash
set -u
# Stale web process from the previous image.
if [ -d /proc/36763 ]; then
  ppid=$(ps -o ppid= -p 36763 | tr -d ' ')
  echo "stale webkit parent $ppid"
  kill 36763 2>/dev/null || true
fi
python3 - << 'PY'
import ctypes, os, time
os.environ["DISPLAY"] = ":0"
x11 = ctypes.CDLL("libX11.so.6")
xtst = ctypes.CDLL("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
xtst.XTestFakeMotionEvent.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong]
xtst.XTestFakeButtonEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
x11.XFlush.argtypes = [ctypes.c_void_p]
x11.XRaiseWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XSetInputFocus.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
dpy = x11.XOpenDisplay(None)
wid = 0x3c00003
x11.XRaiseWindow(dpy, wid)
x11.XSetInputFocus(dpy, wid, 1, 0)
x11.XFlush(dpy)
time.sleep(0.2)
# Left rail settings gear, from an earlier map of this 1920x1080 window.
x, y = 64, 666
xtst.XTestFakeMotionEvent(dpy, -1, x, y, 0)
x11.XFlush(dpy)
time.sleep(0.1)
xtst.XTestFakeButtonEvent(dpy, 1, 1, 0)
x11.XFlush(dpy)
time.sleep(0.08)
xtst.XTestFakeButtonEvent(dpy, 1, 0, 0)
x11.XFlush(dpy)
print("clicked settings", x, y)
PY
sleep 1
DISPLAY=:0 xwininfo -root -tree | awk '/"Sound Ninja"|"Theme Creator"/ {print}'
