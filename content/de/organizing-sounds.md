---
title: Sounds organisieren
description: Tags, Filter, Tabs, Gruppen, Sortierung, Mehrfachauswahl, Lautstärke und Hotkeys.
order: 4
---

# Sounds organisieren

SoundNinja ist für große Bibliotheken gebaut. **Tags und Filter**, Tabs, Suche, Mehrfachauswahl und Hotkeys — der Clip ist einen Klick (oder eine Taste) entfernt.

## Tabs

- Tab mit **Tab hinzufügen** anlegen. Farbe vergeben, damit die Leiste schnell scannbar bleibt.
- Die Ansicht **Alle** zeigt jeden Sound über alle Tabs.
- Ein Sound kann **einem oder mehreren Tabs** gehören. Rechtsklick → **Zu Tab verschieben**.

Tab-Übergänge: **Einstellungen → Allgemein** — Slide, Fade, Buttons nacheinander oder Keine.

## Gruppen

Seit v0.5.3 können Trenner zu benannten Gruppen werden:

- Gruppe über das Kontextmenü hinzufügen, umbenennen oder entfernen
- Gruppenkopf ziehen — Kinder kommen mit
- Sounds in eine Gruppe legen oder herausnehmen
- Rahmenfarbe, Namensfarbe und Button-Ausrichtung setzen (oder Tab-Standard nutzen)

Ein einfacher **Trenner** geht weiter, wenn du nur einen visuellen Bruch brauchst.

Rechtsklick auf leeren Board-Bereich: **Gruppe hinzufügen**.

## Tags / Filter

Tags gelten tabübergreifend. Das ausfahrbare **Filter**-Panel (Sidebar **Filter**) ist der Ort zum Anlegen und zum Filtern des Boards.

### Tags anlegen und bearbeiten

1. **Filter** in der Sidebar öffnen.
2. Unter **Tags** → **Tag hinzufügen** — Name und Farbe vergeben.
3. Im selben Panel umbenennen, umfärben oder **Tag löschen**.
4. Steht da **Noch keine Tags**, zuerst hier einen anlegen, bevor du zuweist.

### Tags an Sounds hängen

Rechtsklick auf einen Sound → **Tags** (Checkboxen). Ein Sound darf mehrere Tags haben. Existieren noch keine, zeigt das Menü den Weg zum Filter-Panel.

### Board filtern

- Tags im Filter-Panel oder als **Chips unter der Suchleiste** umschalten.
- Abgleich ist **ODER**: ein Sound bleibt sichtbar, wenn er **irgendeinen** gewählten Tag hat.
- Das Feld **Suche** in der Navbar kombiniert per **UND** mit dem aktiven Tag-Filter (Tippen + Tags zusammen).
- Chips aus (oder Filter leeren), um den ganzen Tab wieder zu sehen.

Farbige Tag-Namen können auf Buttons erscheinen. Schalter: **Einstellungen → Verhalten → Tag-Badges anzeigen**.

## Sortierung

Im Filter-Panel eine Sortierung wählen:

- **Benutzerdefiniert** — deine Tab-Reihenfolge, Gruppen sichtbar
- **Name**, **Hinzugefügt**, **Spielzeit** oder **Dateigröße** — flache sortierte Liste; **Gruppen sind ausgeblendet**

Sortierung nach Dauer kann beim ersten Mal die Dauer-Metadaten vorwärmen.

## Neu anordnen

Drag-and-Drop in **Einstellungen → Verhalten** einschalten (**Neu anordnen erlauben**). Dann Sounds und Gruppen ziehen. Neu anordnen gilt bei **Benutzerdefiniert**.

## Mehrfachauswahl

**Mehrfachauswahl** in der Navbar einschalten, dann:

- Klick toggelt einen Sound
- **Shift+Klick** wählt einen Bereich ab dem letzten Anker
- **Rahmen** auf leerem Board ziehen
- Klick auf leeres Board (oder außerhalb) leert die Auswahl

Eine Sammelleiste zeigt die Anzahl plus **Farbe**, **Zu Tab verschieben…**, **Löschen** und **Fertig**.

In der Mehrfachauswahl zeigt jeder Button einen **8-Zeichen-ID-Chip**. Rechtsklick auf den Chip kopiert die ID.

## Lautstärke pro Sound

Rechtsklick → **Lautstärke**. Slider von 0 % bis 100 %. **Doppelklick** setzt auf 100 % zurück.

Das stapelt sich mit der Master-Ausgangslautstärke in den Einstellungen.

## Hotkeys und Sound-IDs

Jeder Sound hat eine 8-stellige ID. Kopieren über das Button-Kontextmenü (**ID kopieren**) oder den Mehrfachauswahl-Chip.

**Einstellungen → Hotkeys**: Taste an eine Sound-ID binden. Globale Hotkeys sind optional. **Hotkey zuweisen** im Kontextmenü springt dorthin, ID vorausgefüllt.

Dieselbe ID nutzt [Bitfocus Companion](/docs/remote-control) zum Auslösen.

## Hintergrundbilder und Farben

Rechtsklick auf einen Sound: eigene Farben oder ein Hintergrundbild. Das ist getrennt vom [Theme Creator](/docs/theme-editor), der das ganze Board gestaltet.

Der Picker ist ein Dialog **Hintergrundbild** (PNG, JPEG, GIF, WebP):

- **Lokale Dateien** — Ordner hinzufügen, einzelne Datei wählen, Dateinamen suchen, Rastervorschau, Ordner aus der Bibliothek entfernen
- **KLIPY** — GIFs online suchen. Eigenen API-Key unter **Einstellungen → Verhalten**. Der Key landet nicht in Projektdateien.

Nach der Wahl Bild auf dem Button schieben und zuschneiden.
