---
title: Aufnahme-Editor
description: Mikrofon oder PC-Audio aufnehmen, den Clip bearbeiten und aufs Board legen.
order: 6
---

# Aufnahme-Editor

Nimm einen Clip in SoundNinja auf, schneide ihn zu und importiere ihn in einen Tab. Öffnen über **Aufnahme** in der Player-Leiste.

## Eingang wählen

Unter **Einstellungen → Audio**:

- **Eingabegeräte** — Mikrofon
- **Ausgabegeräte (PC-Audio)** — Loopback dessen, was der PC spielt
- **Eingangslautstärke** — Aufnahme-Gain von 0 % bis 200 %

Der Aufnahme-Button nutzt das hier gewählte Gerät.

## Take bearbeiten

- Aufnehmen und stoppen
- Pegelmesser beobachten
- **Normalisieren** oder **Rauschunterdrückung**
- Ganzen Clip trimmen, auf Auswahl trimmen oder Auswahl löschen
- Rückgängig / Wiederholen
- Rein- und rauszoomen

## Stagen und importieren

1. Auswahl oder ganzen Track zur Staging-Liste hinzufügen.
2. Die Liste startet leer: aufnehmen, bearbeiten, dann Clip hinzufügen.
3. **Auswahl importieren** oder **Alle importieren** in den aktuellen oder einen gewählten Tab.

## Stem-Trennung (optional)

Der Aufnahme-Editor kann **Gesang** oder **Musik** mit BS-RoFormer behalten. Das Modell ist etwa 158 MB und **nicht** im App-Paket.

Download unter **Einstellungen → Stem-Trennung** oder über den Erststart-Hinweis. Der Windows-Installer kann den Download beim ersten Start anbieten.

Source-Builds mit Stems brauchen extra Flags (`tauri:serve:stems` / `tauri:build:stems`).
