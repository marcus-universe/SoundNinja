---
title: Fernsteuerung
description: SoundNinja über das lokale Netz mit Bitfocus Companion auslösen.
order: 7
---

# Fernsteuerung

SoundNinja kann im lokalen Netz lauschen, damit Bitfocus Companion (oder jeder HTTP/WebSocket-Client) Sounds spielt und stoppt.

## Remote-Server einschalten

1. **Einstellungen → Remote** öffnen.
2. Remote-Server aktivieren. Standardport ist **7331**.
3. Optional: Token setzen. Clients senden ihn als Bearer-Token oder `?token=`.
4. `http://IP:PORT` kopieren oder die System-IP unter **Einstellungen → Über** holen.

Windows kann beim ersten Start eine Firewall-Abfrage zeigen. Zugriff im privaten Netz erlauben.

## Bitfocus Companion

1. [Bitfocus Companion](https://bitfocus.io/companion) installieren.
2. [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz) herunterladen. Es gibt noch kein Companion-Store-Paket.
3. In Companion: **Modules → Load module package** und die `.tgz` wählen.
4. Eine **Sound Ninja**-Verbindung anlegen. PC-IP, Port und Token eintragen.

## Aktionen

- **Trigger Sound** — aus dem Live-Dropdown wählen oder 8-stellige Sound-ID tippen (Companion-Variablen gehen)
- **Stop Sound** — einen spielenden Sound per ID stoppen
- **Stop All** — alle spielenden Sounds stoppen

Sound-ID in SoundNinja über das Button-Kontextmenü oder den Mehrfachauswahl-Chip kopieren.

## Feedbacks und Variablen

- **Sound Playing** — wahr, solange diese Sound-ID aktiv ist (für Button-Farbe)
- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — wie viele Sounds spielen
- `$(soundninja:last_triggered)` — zuletzt ausgelöste Sound-ID

## HTTP-API (fortgeschritten)

Basispfad: `/api/v1`

- `GET /info` — App-Name, Version, Protokoll, ob ein Token nötig ist
- `GET /sounds` — Soundliste (id, name, tabs, active)
- `GET /state` — Sounds plus gerade spielende IDs
- `POST /trigger` mit `{ "id" }` — Sound abspielen
- `GET /trigger/:id` — abspielen (einfach im Browser testen)
- `POST /stop` mit optionalem `{ "id" }` — einen Sound oder alle stoppen
- `GET /ws` — Live-Status; Trigger/Stop eingehend senden
