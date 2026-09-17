---
title: Organizing Sounds
description: Tags, filters, tabs, groups, sort, multi-select, volume, and hotkeys.
order: 4
---

# Organizing Sounds

SoundNinja is built for large libraries. Use **tags and filters**, tabs, search, multi-select, and hotkeys so the clip you need is one click (or one key) away.

## Tabs

- Add a tab with **Add tab**. Give it a color so you can scan the bar fast.
- The **All** view shows every sound across tabs.
- A sound can belong to **one or more tabs**. Right-click a sound and use **Move to tab** to assign it.

Tab transitions live in **Settings → Main**: Slide, Fade, Buttons one by one, or None.

## Groups

Since v0.5.3, separators can become named groups:

- Add, rename, or remove a group from the context menu
- Drag the group header — children move with it
- Drop sounds in or out of a group
- Set group border color, name color, and button alignment (or use the tab default)

You can still add a plain **Separator** if you only need a visual break.

Right-click empty board space to **Add Group**.

## Tags / Filters

Tags cut across tabs. The slide-out **Filter** panel (sidebar **Filter**) is where you create them and turn them into a board filter.

### Create and edit tags

1. Open **Filter** in the sidebar.
2. Under **Tags**, use **Add tag** — give it a name and a color.
3. Rename, recolor, or **Delete tag** from the same list.
4. If you see **No tags yet**, add one here before you assign anything.

### Assign tags to sounds

Right-click a sound → **Tags** (checkboxes). A sound can have several tags. If none exist yet, the menu tells you to open the filter panel.

### Filter the board

- Toggle tags in the Filter panel, or as **chips under the search bar**.
- Matching is **OR**: a sound stays visible if it has **any** selected tag.
- The navbar **Search** field **ANDs** with the active tag filter (type + tags together).
- Turn chips off (or clear the filter) to show the full tab again.

Colored tag names can appear on buttons. Toggle them in **Settings → Behavior → Show tag badges**.

## Sort

Open the filter panel and pick a sort mode:

- **User defined** — your tab order, with groups visible
- **Name**, **Added time**, **Play time**, or **File size** — a flat sorted list; **groups are hidden** in these modes

Duration sort may warm duration metadata the first time you use it.

## Reorder

Turn on drag-and-drop in **Settings → Behavior** (**Allow reorder**). Then drag sounds and groups into place. Reorder applies in **User defined** sort.

## Multi-select

Turn on **Multi-select** in the navbar, then:

- Click a sound to toggle it
- **Shift+click** to select a range from the last anchor
- Drag a **marquee** on empty board space
- Click empty board (or outside) to clear the selection

A bulk bar appears with the count, plus **Color**, **Move to tab…**, **Delete**, and **Done**.

In multi-select, each button shows an **8-character ID chip**. Right-click the chip to copy it.

## Per-sound volume

Right-click a sound → **Volume**. Drag the slider from 0% to 100%. **Double-click** the slider to reset to 100%.

This stacks with the master output volume in Settings.

## Hotkeys and sound IDs

Every sound has an 8-character ID. Copy it from the button context menu (**Copy ID**) or the multi-select ID chip.

Open **Settings → Hotkeys** to bind a key to a sound ID. Global hotkeys are optional. **Assign hotkey** on the context menu jumps there with the ID prefilled.

The same ID is what [Bitfocus Companion](/docs/remote-control) uses to trigger a clip.

## Background images and colors

Right-click a sound to set per-button colors or a background image. That is separate from the [Theme Creator](/docs/theme-editor), which styles the whole board.

The picker is a **Background image** dialog (PNG, JPEG, GIF, WebP):

- **Local files** — add folders, pick a single file, search filenames, preview the grid, or remove a folder from the library
- **GifSnap** — search online GIFs with no API key. The first visit asks you to accept GifSnap requests; search results are cached to cut traffic. See [GifSnap](https://gifsnap.com).
- **KLIPY** — search online GIFs. Set your own API key in **Settings → Behavior**. The key is not stored in project files.

After you pick an image, pan and crop it on the button.
