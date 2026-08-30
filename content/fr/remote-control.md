---
title: Contrôle à distance
description: Déclencher SoundNinja depuis Bitfocus Companion sur le réseau local.
order: 7
---

# Contrôle à distance

SoundNinja peut écouter sur ton réseau local pour que Bitfocus Companion (ou n’importe quel client HTTP/WebSocket) joue et arrête des sons.

## Activer le serveur distant

1. Ouvre **Paramètres → Distant**.
2. Active le serveur distant. Le port par défaut est **7331**.
3. Optionnel : définis un token. Les clients doivent l’envoyer comme Bearer token ou `?token=`.
4. Copie l’URL `http://IP:PORT`, ou copie l’IP système depuis **Paramètres → À propos**.

Windows peut afficher une invite pare-feu la première fois que le serveur démarre. Autorise l’accès au réseau privé.

## Bitfocus Companion

1. Installe [Bitfocus Companion](https://bitfocus.io/companion).
2. Télécharge le module SoundNinja depuis le dossier officiel : [companion-module-soundninja](https://github.com/marcus-universe/SoundNinja/tree/main/companion-module-soundninja). Il n’y a pas encore de paquet Companion Store — ce dossier est le téléchargement.
3. Dans Companion, ouvre **Developer → Modules**, ou dépose le dossier dans le chemin des modules custom de Companion.
4. Dans le dossier du module, lance `npm install`.
5. Ajoute une connexion **Sound Ninja**. Colle l’IP du PC, le port, et le token si tu en as défini un.

## Actions

- **Trigger Sound** — choisis dans le menu déroulant live, ou tape un ID de son de 8 caractères (les variables Companion marchent)
- **Stop Sound** — arrête un son en lecture par ID
- **Stop All** — arrête tous les sons en lecture

Copie un ID de son dans SoundNinja depuis le menu contextuel du bouton ou la puce d’ID de la sélection multiple.

## Feedbacks et variables

- **Sound Playing** — vrai tant que cet ID de son est actif (utilise-le pour la couleur du bouton)
- `$(soundninja:connected)` — `true` / `false`
- `$(soundninja:playing_count)` — combien de sons jouent
- `$(soundninja:last_triggered)` — dernier ID de son déclenché

## API HTTP (avancé)

Chemin de base : `/api/v1`

- `GET /info` — nom de l’app, version, protocole, si un token est requis
- `GET /sounds` — liste des sons (id, name, tabs, active)
- `GET /state` — sons plus IDs actuellement en lecture
- `POST /trigger` avec `{ "id" }` — jouer un son
- `GET /trigger/:id` — jouer (facile à tester dans un navigateur)
- `POST /stop` avec `{ "id" }` optionnel — arrêter un son ou tous
- `GET /ws` — état en direct ; envoyer des commandes trigger/stop en entrée
