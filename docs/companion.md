# Remote control / Companion

Sound Ninja can expose a local HTTP + WebSocket API so [Bitfocus Companion](https://bitfocus.io/companion) (or any HTTP client) can trigger sounds by ID and stop playback.

1. Open **Settings → Remote** and enable the server (default port `7331`).
2. Copy the `http://IP:PORT` URL from that tab, or copy the system IP from **Settings → About**.
3. Download the latest [Companion module `.tgz`](https://github.com/marcus-universe/SoundNinja/releases/latest/download/SoundNinja-Companion.tgz) and load it in Companion (**Modules → Load module package**). Paste the IP + port into the connection.

Source: [`companion-module` branch](https://github.com/marcus-universe/SoundNinja/tree/companion-module). Create a SoundNinja Release **draft** first, then run **Actions → Release Companion** (or the main **Release** orchestrator). The action packages the module and uploads `SoundNinja-Companion.tgz` plus `SoundNinja-Companion-{Version}.tgz` onto that draft.

Optional token: set one in Remote settings. Clients send `Authorization: Bearer <token>` or `?token=`.

API (`/api/v1`):

| Method | Path | Auth | Description |
| ------ | ---- | ---- | ----------- |
| GET | `/info` | no | App name, version, protocol, whether a token is required |
| GET | `/sounds` | yes | Sound list (`id`, `name`, `tabs`, `active`) |
| GET | `/state` | yes | Sounds + currently playing IDs |
| POST | `/trigger` `{ "id" }` | yes | Play a sound |
| GET | `/trigger/:id` | yes | Play a sound (browser-testable) |
| POST | `/stop` `{ "id"? }` | yes | Stop one sound, or all if `id` omitted |
| GET | `/ws` | yes | Live state push; inbound `{ "cmd": "trigger"\|"stop", "id"? }` |

Windows may prompt to allow Sound Ninja through the firewall the first time the server starts.
