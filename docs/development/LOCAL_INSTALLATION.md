# Installation utilisateur Linux — 23 novembre 2026

Installation et lancement explicitement autorisés par Kiunda. Budget : dix minutes,
deux jobs Cargo, objectifs 2 Gio RAM et 1 Gio disque supplémentaire. Aucune
installation de dépendance, aucun sudo ni service au démarrage automatique.

## Version installée

- Version locale de développement 0.1.0, profil debug, frontend embarqué via
  la feature `custom-protocol`. Aucun serveur Vite nécessaire au lancement.
- Binaire : `~/.local/share/systempulse/0.1.0-local-20261123/systempulse-desktop`.
- Commande : `~/.local/bin/systempulse-desktop`.
- Menu : `~/.local/share/applications/io.systempulse.desktop.desktop`.
- Icônes d'installation : `icon.png` et `icon.svg` à côté du binaire.
- SHA-256 du binaire compilé et installé, identiques :
  `96deb08ef8aa9e70820515875ba989e84513bc2c1ba972f1aeea3dbd4040e493`.

Le binaire debug occupe environ 195 Mio. Le cache Tauri est passé d'environ
3063 à 3672 Mio au relevé après build/Clippy ; avec la copie installée, environ
804 Mio supplémentaires à ces relevés, hors variations mineures de la session.
Le pic RAM global n'a pas été mesuré. La compilation principale a pris environ
18 secondes ; aucun objectif de performance de la version release n'est revendiqué.

## Nouvelle identité graphique

Source : `apps/desktop/app-icon-power.svg`. Un processeur sombre, une impulsion
cyan/menthe/violet et un éclair évoquent la puissance surveillée et maîtrisée.
Génération locale avec le CLI Tauri existant vers `src-tauri/icons-power` :
PNG, ICO et ICNS. Ancienne source et anciennes icônes conservées.
L'aperçu PNG 256 × 256 a été lu et vérifié visuellement.

## Commandes et contrôles réellement exécutés

Depuis `apps/desktop` :

```sh
npm exec --offline -- tauri icon app-icon-power.svg --output src-tauri/icons-power
PATH="$HOME/.cargo/bin:$PATH" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 \
  timeout 420 npm exec --offline -- tauri build --debug --no-bundle --ci -- --locked
npm test
```

- Build frontend TypeScript/Vite et build Tauri : PASS, sans accès réseau Cargo.
- 9 tests frontend : PASS.
- Clippy desktop, toutes cibles, `--features custom-protocol --locked`, deux jobs,
  timeout 60 s : PASS, aucun avertissement.
- Copie utilisateur : empreintes identiques ; aucun fichier d'installation
  préexistant remplacé. Les caches `io.systempulse.desktop` sont conservés.
- `desktop-file-validate`, `sh -n` du lanceur et `git diff --check` : PASS.
- Base des applications utilisateur actualisée via `update-desktop-database`.
- Lancement via `gio launch` du fichier desktop : processus observé ; une
  application AT-SPI `systempulse-desktop`, fenêtre `SystemPulse`, rôle `frame`,
  état `SHOWING=true`. Aucun rendu pixel de la fenêtre native capturé.

La fenêtre utilise la session graphique courante. Son absence dans l'arbre X11
ne constituait pas un échec : sa visibilité a été confirmée via AT-SPI.
Le journal ponctuel de lancement dans `/tmp/opencode` était vide au contrôle.

## Utilisation et limites

Chercher **SystemPulse** dans le menu des applications, ou lancer
`~/.local/bin/systempulse-desktop`. Fermer normalement la fenêtre pour quitter.
Pas de lancement automatique à la connexion. Pas de packaging système, signature,
licence choisie ou publication implicite. Les changements du dépôt ne sont pas
commités/poussés par ce lot.

Les avis Rust documentés dans
`docs/testing/SECURITY_FOLLOWUP_2026-11-23.md` restent ouverts. Ce build local
n'est pas une release de production ni une preuve de qualification Windows/macOS.
Lancement confirmé ne signifie pas test exhaustif de toutes les vues et capteurs.
Toute mise à jour remplaçant cette installation devra préserver l'ancienne version
et les données utilisateur ; aucune suppression automatique n'a été mise en place.
