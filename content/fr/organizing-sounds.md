---
title: Organiser les sons
description: Tags, filtres, onglets, groupes, tri, sélection multiple, volume et raccourcis.
order: 4
---

# Organiser les sons

SoundNinja est conçu pour les grandes bibliothèques. **Tags et filtres**, onglets, recherche, sélection multiple et raccourcis — le clip dont tu as besoin est à un clic (ou une touche) près.

## Onglets

- Ajoute un onglet avec **Ajouter un onglet**. Donne-lui une couleur pour scanner la barre vite.
- La vue **Tous** montre chaque son à travers les onglets.
- Un son peut appartenir à **un ou plusieurs onglets**. Clic droit sur un son et utilise **Déplacer vers l’onglet** pour l’assigner.

Les transitions d’onglets sont dans **Paramètres → Général** : Slide, Fade, Boutons un par un, ou Aucune.

## Groupes

Depuis la v0.5.3, les séparateurs peuvent devenir des groupes nommés :

- Ajouter, renommer ou supprimer un groupe depuis le menu contextuel
- Glisser l’en-tête du groupe — les enfants suivent
- Déposer des sons dans un groupe ou les en sortir
- Régler la couleur de bordure, la couleur du nom et l’alignement des boutons (ou utiliser le défaut de l’onglet)

Tu peux toujours ajouter un **Séparateur** simple si tu as seulement besoin d’une coupure visuelle.

Clic droit sur l’espace vide du board pour **Ajouter un groupe**.

## Tags / Filtres

Les tags traversent les onglets. Le panneau **Filtre** (barre latérale **Filtre**) sert à les créer et à filtrer le board.

### Créer et modifier des tags

1. Ouvre **Filtre** dans la barre latérale.
2. Sous **Tags**, utilise **Ajouter un tag** — nom et couleur.
3. Renomme, recolore ou **Supprimer le tag** dans la même liste.
4. Si tu vois **Aucun tag pour l’instant**, crée-en un ici avant d’assigner.

### Assigner des tags aux sons

Clic droit sur un son → **Tags** (cases à cocher). Un son peut avoir plusieurs tags. S’il n’y en a encore aucun, le menu pointe vers le panneau filtre.

### Filtrer le board

- Active les tags dans le panneau Filtre, ou comme **puces sous la barre de recherche**.
- Le match est **OU** : un son reste visible s’il a **n’importe quel** tag sélectionné.
- Le champ **Recherche** combine en **ET** avec le filtre de tags actif (saisie + tags ensemble).
- Désactive les puces (ou vide le filtre) pour revoir tout l’onglet.

Les noms de tags colorés peuvent apparaître sur les boutons. Active-les dans **Paramètres → Comportement → Afficher les badges de tags**.

## Tri

Ouvre le panneau filtre et choisis un mode :

- **Défini par l’utilisateur** — ton ordre d’onglet, groupes visibles
- **Nom**, **Date d’ajout**, **Durée de lecture** ou **Taille de fichier** — liste plate triée ; **les groupes sont masqués**

Le tri par durée peut préchauffer les métadonnées de durée la première fois.

## Réordonner

Active le glisser-déposer dans **Paramètres → Comportement** (**Autoriser le réordonnancement**). Puis glisse sons et groupes en place. Le réordonnancement s’applique en tri **Défini par l’utilisateur**.

## Sélection multiple

Active **Sélection multiple** dans la barre de navigation, puis :

- Clique pour basculer un son
- **Maj+clic** pour une plage depuis le dernier ancrage
- Trace un **rectangle** sur l’espace vide du board
- Clique le board vide (ou à l’extérieur) pour vider la sélection

Une barre groupée affiche le compte, plus **Couleur**, **Déplacer vers l’onglet…**, **Supprimer** et **Terminé**.

En sélection multiple, chaque bouton montre une **puce d’ID de 8 caractères**. Clic droit sur la puce pour la copier.

## Volume par son

Clic droit → **Volume**. Glisse le curseur de 0 % à 100 %. **Double-clic** le remet à 100 %.

Ça se cumule avec le volume de sortie principal dans les Paramètres.

## Raccourcis et IDs de son

Chaque son a un ID de 8 caractères. Copie-le depuis le menu contextuel du bouton (**Copier l’ID**) ou la puce d’ID de la sélection multiple.

Ouvre **Paramètres → Raccourcis** pour lier une touche à un ID de son. Les raccourcis globaux sont optionnels. **Assigner un raccourci** dans le menu contextuel y saute avec l’ID déjà rempli.

Le même ID est ce que [Bitfocus Companion](/docs/remote-control) utilise pour déclencher un clip.

## Images de fond et couleurs

Clic droit sur un son pour régler des couleurs par bouton ou une image de fond. C’est séparé du [Theme Creator](/docs/theme-editor), qui habille tout le board.

Le sélecteur est une boîte **Image de fond** (PNG, JPEG, GIF, WebP) :

- **Fichiers locaux** — ajouter des dossiers, choisir un fichier, chercher par nom, prévisualiser la grille, retirer un dossier de la bibliothèque
- **KLIPY** — chercher des GIF en ligne. Mets ta propre clé API dans **Paramètres → Comportement**. La clé n’est pas stockée dans les fichiers projet.

Après le choix, recadre et décale l’image sur le bouton.
