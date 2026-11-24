# Préférences Système — 23 novembre 2026

## Comportement

- Langue : Système / Français / English. Système lit `navigator.languages`,
  puis `navigator.language` si la liste est vide. La première langue FR ou EN
  prise en charge est choisie, y compris les variantes régionales (`en-US`,
  `fr-CD`). Si aucune n'est traduite, repli français, explicitement indiqué.
- Apparence : Système / Sombre / Clair. Système suit
  `prefers-color-scheme: dark` via `matchMedia`.
- Les événements `languagechange` et `MediaQueryList.change` recalculent les
  valeurs effectives. Les valeurs manuelles ignorent les changements système.
  Les abonnements sont retirés sur `pagehide`.
- La préférence enregistrée reste `system` : elle n'est jamais remplacée par
  la valeur effective `fr`, `en`, `dark` ou `light` pendant la résolution.
- Sans préférence existante ou avec une valeur invalide : Système par défaut.
  Les anciennes préférences manuelles stockées dans les mêmes clés sont conservées.
  Si localStorage est interdit, l'application garde les choix en mémoire sans crash.
- Le focus de la liste reste sur le même réglage lors du rafraîchissement des
  préférences. Les contrôles natifs reçoivent aussi le `color-scheme` effectif.
- Profil de collecte : Eco / Balanced / Detailed restent explicites ; un texte
  explique qu'il n'existe pas de profil SystemPulse défini par l'OS. Pas de faux
  choix Système, de collecte batterie supplémentaire ou d'adaptation de cadence
  non définie. Les réglages futurs devront appliquer Système seulement à un
  signal OS réel et documenté.

## Vérifications réellement exécutées

- `npm run check && npm test && npm run build` : PASS, 13 tests, 15 modules Vite.
- Quatre tests nouveaux : migration/sauvegarde/valeurs invalides, priorité des
  locales et repli, résolution du thème et stabilité manuelle, stockage interdit.
- La résolution est testée avec différents signaux système synthétiques ; aucun
  changement des réglages globaux du PC n'a été effectué. La réception réelle des
  événements de changement de langue/thème par chaque WebView reste à qualifier.
  Certains environnements peuvent nécessiter une relance pour actualiser la langue
  système qu'ils exposent au navigateur.
- Build natif hors ligne, deux jobs, timeout 240 s : PASS, compilation Rust
  incrémentale environ 2,28 s. Aucun paquet ni runtime installé.
- Binaire copié dans une nouvelle version utilisateur ; SHA-256 source et copie :
  `7974eb48bce488bf45ec114541041de4241416bd99d047a5cf8e963ac712e668`.
- `sh -n` du lanceur et validation du fichier desktop : PASS.
- L'ancienne fenêtre a été fermée normalement par son bouton Close via AT-SPI,
  après accord explicite de redémarrage et de perte du graphique en mémoire.
- Nouvelle fenêtre visible, ouverture de Settings via son bouton : PASS. AT-SPI
  confirme trois combo boxes (Appearance, Language, Collection profile) et deux
  options System. Ce contrôle ne simule pas un changement OS à chaud.
- `git diff --check` : PASS. Une première sonde Python ponctuelle comportait une
  erreur d'indentation ; remplacée par un script borné ayant effectué les contrôles
  natifs ci-dessus. Pas de capture de l'écran ni de lecture des autres applications.

## Installation mise à jour

Chemin : `~/.local/share/systempulse/0.1.0-local-20261123-system-preferences/systempulse-desktop`.
Le lanceur `~/.local/bin/systempulse-desktop` pointe vers cette version. L'ancien
binaire `0.1.0-local-20261123` et son icône sont conservés, tout comme les données
de WebView et préférences. Le raccourci de menu et l'identifiant applicatif sont
inchangés. L'application a été laissée ouverte sur les paramètres.

Aucun commit, push, nettoyage ou changement de sécurité/dépendance Rust effectué.
Cette mise à jour reste une version locale de développement ; les avis Rust
précédemment documentés ne sont pas résolus par ce lot.
