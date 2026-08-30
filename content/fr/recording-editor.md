---
title: Éditeur d’enregistrement
description: Capturer le micro ou l’audio du PC, éditer le clip et le poser sur ton board.
order: 6
---

# Éditeur d’enregistrement

Enregistre un clip dans SoundNinja, coupe-le, puis importe-le sur un onglet. Ouvre l’éditeur avec le bouton **Enregistrer** dans la barre du lecteur.

## Choisir une entrée

Dans **Paramètres → Audio** :

- **Périphériques d’entrée** — microphone
- **Périphériques de sortie (audio PC)** — loopback de ce que le PC joue
- **Volume d’entrée** — gain de capture de 0 % à 200 %

Le bouton Enregistrer utilise le périphérique choisi ici.

## Éditer la prise

- Enregistrer et arrêter
- Surveiller le vu-mètre d’entrée
- **Normaliser** ou **Réduction de bruit**
- Couper tout le clip, couper à une sélection, ou supprimer une sélection
- Annuler / rétablir
- Zoomer et dézoomer

## Préparer et importer

1. Ajoute la sélection ou la piste complète à la liste de staging.
2. La liste commence vide : enregistre, édite, puis ajoute un clip.
3. **Importer la sélection** ou **Tout importer** dans l’onglet actuel ou un onglet que tu choisis.

## Séparation de stems (optionnel)

Le Record Editor peut garder les **voix** ou la **musique** avec BS-RoFormer. Le modèle fait environ 158 MB et n’est **pas** inclus avec l’app.

Télécharge-le depuis **Paramètres → Séparation de stems**, ou accepte l’invite au premier lancement. L’installateur Windows peut aussi proposer de télécharger le modèle au premier démarrage.

Les builds source avec stems ont besoin d’un flag extra (`tauri:serve:stems` / `tauri:build:stems`).
