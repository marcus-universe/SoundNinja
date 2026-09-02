---
title: Remote Control
description: Trigger SoundNinja from Bitfocus Companion over the local network.
order: 7
---

# Remote Control

SoundNinja can listen on your local network so Bitfocus Companion (or any HTTP/WebSocket client) can play and stop sounds.

## Enable the remote server

1. Open **Settings → Remote**.
2. Enable the remote server. Default port is **7331**.
3. Optional: set a token. Clients must send it as a Bearer token or `?token=`.
4. Copy the `http://IP:PORT` URL, or copy the system IP from **Settings → About**.

Windows may show a firewall prompt the first time the server starts. Allow private-network access.

## Bitfocus Companion

1. Install [Bitfocus Companion](https://bitfocus.io/companion).
2. Download [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz). There is no Companion Store package yet.
3. In Companion: **Modules → Load module package** and pick the `.tgz`.
4. Add a **Sound Ninja** connection. Paste the PC IP, port, and token if you set one.

## Actions

- **Trigger Sound** — pick from the live dropdown, or type an 8-character sound ID (Companion variables work)
- **Stop Sound** — stop one playing sound by ID
- **Stop All** — stop every playing sound

Copy a sound ID in SoundNinja from the button context menu or the multi-select ID chip.

## Feedbacks and variables

- **Sound Playing** — true while that sound ID is active (use it for button color)
- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — how many sounds are playing
- `$(soundninja:last_triggered)` — last triggered sound ID

## HTTP API (advanced)

Base path: `/api/v1`

- `GET /info` — app name, version, protocol, whether a token is required
- `GET /sounds` — sound list (id, name, tabs, active)
- `GET /state` — sounds plus currently playing IDs
- `POST /trigger` with `{ "id" }` — play a sound
- `GET /trigger/:id` — play (easy to test in a browser)
- `POST /stop` with optional `{ "id" }` — stop one sound or all
- `GET /ws` — live state; send trigger/stop commands inbound
