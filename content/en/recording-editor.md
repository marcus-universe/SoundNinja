---
title: Recording Editor
description: Capture mic or PC audio, edit the clip, and drop it on your board.
order: 6
---

# Recording Editor

Record a clip inside SoundNinja, trim it, then import it onto a tab. Open the editor with the **Record** button in the player bar.

## Choose an input

In **Settings → Audio**:

- **Input Devices** — microphone
- **Output Devices (PC Audio)** — loopback of what your PC is playing
- **Input volume** — capture gain from 0% to 200%

The Record button uses the device you pick here.

## Edit the take

- Record and stop
- Watch the input level meter
- **Normalize** or **Noise Cancelation**
- Trim the whole clip, trim to a selection, or delete a selection
- Undo / redo
- Zoom in and out

## Stage and import

1. Add the selection or the full track to the staging list.
2. The list starts empty: record, edit, then add a clip.
3. **Import selected** or **Import all** into the current tab or a tab you choose.

## Stem separation (optional)

The Record Editor can keep **vocals** or **music** using BS-RoFormer. The model is about 158 MB and is **not** bundled with the app.

Download it from **Settings → Stem Separation**, or accept the first-run prompt. The Windows installer can also offer to download the model on first launch.

Stem builds from source need an extra flag (`tauri:serve:stems` / `tauri:build:stems`).
