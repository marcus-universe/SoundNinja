# Sound Ninja

Trigger sounds on a running [Sound Ninja](https://github.com/marcus-universe/SoundNinja) instance over the local network.

## Setup

1. Download [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz) and load it in Companion (**Modules → Load module package**).
2. In Sound Ninja open **Settings → Remote**.
3. Enable the remote server. Default port is `7331`.
4. Optional: set a token so only clients that know it can control playback.
5. Copy the `http://IP:PORT` URL (or copy the system IP from **Settings → About**).
6. In Companion add a **Sound Ninja** connection and paste the IP + port (and token if you set one).

Windows may show a firewall prompt the first time the server starts. Allow private-network access.

## Actions

- **Trigger Sound** — play a sound by the live dropdown, or a custom 8-character sound ID (supports Companion variables).
- **Stop Sound** — stop one playing sound by ID.
- **Stop All** — stop every playing sound.

Copy a sound ID in Sound Ninja via the button context menu or the multi-select ID chip.

## Feedbacks

- **Sound Playing** — true while that sound ID is active. Use it to change button color.

## Variables

- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — number of currently playing sounds
- `$(soundninja:last_triggered)` — last triggered sound ID
