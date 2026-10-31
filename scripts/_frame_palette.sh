#!/bin/bash
set -u
python3 - << 'PY'
import ctypes, os, time
os.environ["DISPLAY"] = ":0"
x11 = ctypes.CDLL("libX11.so.6")
xtst = ctypes.CDLL("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
xtst.XTestFakeMotionEvent.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong]
xtst.XTestFakeButtonEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong]
x11.XFlush.argtypes = [ctypes.c_void_p]
dpy = x11.XOpenDisplay(None)
x, y = 1590, 399
xtst.XTestFakeMotionEvent(dpy, -1, x, y, 0)
x11.XFlush(dpy)
time.sleep(0.15)
xtst.XTestFakeButtonEvent(dpy, 1, 1, 0)
x11.XFlush(dpy)
time.sleep(0.08)
xtst.XTestFakeButtonEvent(dpy, 1, 0, 0)
x11.XFlush(dpy)
print("clicked", x, y)
PY
sleep 2
DISPLAY=:0 xwininfo -root -tree | awk '/"Sound Ninja"|"Theme Creator"/ {print}'
