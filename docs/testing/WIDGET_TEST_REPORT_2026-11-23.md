# Capsule — rapport du lot du 23 novembre 2026

## État livré

Prototype fonctionnel Linux installé, capsule laissée visible et dashboard masqué.
Anciennes versions et sauvegardes des lanceurs conservées. Aucune connexion réelle
Claude/Codex, aucune lecture de leur configuration privée, aucun démarrage
automatique activé, aucun commit/push réalisé pour ce lot.

Étude et décisions : [WIDGET_DESIGN.md](../architecture/WIDGET_DESIGN.md).
Les modifications d'icône et préférences déjà présentes avant ce lot sont conservées.
Pas de nouvelle dépendance ni changement de version de dépendance.

## Contrôles exécutés

| Contrôle | Résultat réel |
| --- | --- |
| `sh scripts/check.sh`, Cargo hors ligne, deux jobs, timeout 120 s | PASS : fmt/Clippy/check headless et desktop ; **31 tests headless** |
| Clippy desktop toutes cibles, feature `custom-protocol`, `--locked -D warnings` | PASS après corrections, dernier passage 0,85 s |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --features custom-protocol --locked` | **4 tests PASS** : diagnostics périmés, labels IPC, géométrie DPI/multi-écran, verrou exclusif/libération/refus symlink |
| `npm run check`, `npm test`, build Vite | **18 tests PASS**, dont 24 paires de contraste texte et vérification des contours ; 19 modules frontend |
| `python3 -m unittest discover -s tests -p test_install_user.py -v` | **6 tests PASS** : défaut sans autostart, activation/désactivation isolée, sauvegardes, dry-run, fichiers non reconnus, liens, permissions, chemins, configuration d'agent intacte |
| `node tests/widget-browser.mjs`, timeout 45 s | PASS : navigateur headless isolé, compacte/déployée, clair/sombre, 320 px, Échap/focus stable après acquisition, GPU absent, confirmation MCP, JSON sans processus, désactivation, régression préférences dashboard |
| Réseau pendant le scénario navigateur | Aucune requête externe observée dans le scénario ; Vite lié à 127.0.0.1 puis arrêté dans `finally` |
| JavaScript pendant le scénario navigateur | Aucune erreur `pageerror` ni console error observée sur le scénario final, widget et dashboard |
| Build natif embarqué | PASS, hors ligne et deux jobs ; dernière compilation incrémentale environ 1,81 s |
| `python3 scripts/test-mcp.py` après build headless | PASS : dix outils, handshake, paramètres/confidentialité, huit appels en vol, entrée 70 Kio, EOF et redémarrage ; environ 3,37 s |
| `python3 scripts/test-widget-native.py --confirm-ui-test` | PASS sur l'installation finale : `--widget` n'affiche pas le dashboard, agrandir/replier, masquer/réafficher, restaurer le dashboard, fermeture du dashboard gardant la capsule |
| Second lancement natif pendant une instance active | Refus contrôlé avec code 1 sur la version finale ; le worker n'est démarré qu'après acquisition du verrou |
| Lanceur et entrée menu | `sh -n`, `desktop-file-validate` PASS ; base utilisateur actualisée |
| `git diff --check` | PASS |
| Gitleaks 8.30.1 sur les sources | Aucun secret détecté ; scan avec masquage complet, caches/builds exclus |
| `npm audit` registre npm officiel, timeout 60 s | **0 vulnérabilité signalée** |
| cargo-audit des deux lockfiles, base locale RustSec du même jour | **ÉCHECS sécurité conservés**, voir section suivante |

Le script navigateur utilise le Playwright déjà installé, sans téléchargement,
profil utilisateur, cookies personnels ou `--no-sandbox`. Captures synthétiques
générées dans `apps/desktop/test-artifacts/` (exclu de Git) :
`widget-capsule.png`, `widget-expanded-dark.png`, `widget-expanded-light.png`.
Les aperçus compact et sombre ont été ouverts pour revue visuelle. Ces captures
de démonstration ne sont pas une capture de la fenêtre avec les vraies mesures.

## Avis de sécurité non résolus

La base RustSec locale contient 1 561 avis ; contrôles exécutés avec `--no-fetch
--no-yanked`, donc sans nouvelle actualisation distante ni vérification des crates
retirés du registre dans ces commandes.

- Racine : `rmcp 3.8.0`, **RUSTSEC-2026-0462** ; concerne les clients HTTP,
  fonctionnalités non activées dans notre serveur stdio. L'audit demeure en échec,
  aucune exception ajoutée. Avertissement de maintenance `chrono`.
- Tauri : `url 2.5.8`, **RUSTSEC-2026-0592** (récursion `blob:`), plus avertissements
  `chrono`, `proc-macro-error` et unsoundness `glib 0.18.5`.

Le lot n'a pas silencieusement mis à jour les dépendances pour rendre l'audit vert.
Ce prototype local n'est pas déclaré sûr à 100 %, prêt production ou prêt release.
Les mises à jour ciblées et compatibles des dépendances restent prioritaires.

## Défauts rencontrés et traitement

- Clippy a refusé deux `if` imbriqués : corrigés et contrôle repassé.
- Échap tentait de rendre le focus au bouton encore désactivé pendant l'IPC :
  corrigé, puis testé après 3,2 s supplémentaires pour vérifier sa stabilité.
- L'installateur créait des parents avec un mode dépendant de l'umask, puis les
  refusait à la mise à jour : création explicite 0755, tests de réinstallation PASS.
- L'installation réelle a révélé `~/.local/bin` group-writable sous un ancêtre
  privé 0700 : le contrôle tient compte de cette protection effective, sans
  modifier les permissions existantes ; test dédié ajouté.
- Le premier refus de double instance passait par une erreur du hook setup Tauri
  et affichait un panic de debug : remplacé par arrêt contrôlé, version finale
  vérifiée avec code 1 et instance initiale toujours active.
- Le premier script AT-SPI supposait le dashboard caché présent dans l'arbre
  d'accessibilité : corrigé pour accepter son absence avant affichage.
- Un passage fmt a été lancé depuis le mauvais répertoire : relancé avec le bon
  manifest, sans modifier l'environnement ou les dépendances.
- Le contrôle console ajouté en fin de test a révélé une requête d'icône 404 :
  favicon SVG explicite ajouté, build et installation r3 refaits. Console et
  régression responsive du dashboard à 320/768/1440 px repassées sans erreur.

## Limites natives observées, pas masquées

- GTK/compositeur maintient ici la fenêtre compacte à **320 × 200**, même après
  configuration d'un minimum de 280 × 96 et demande logique de 320 × 112.
  L'ovale opaque demeure petit, aligné au bas de cette zone transparente. La
  surface d'entrée peut dépasser l'ovale visible ; forme d'entrée native à affiner.
  Déploiement/repli final mesuré : **320 × 200 ↔ 400 × 620**.
- Sous Wayland, les positions globales exposées par AT-SPI ne sont pas une preuve
  de placement absolu. La demande bas-droite est bornée par l'écran, mais son
  respect et le comportement multi-écran réels ne sont pas qualifiés. Pas de
  contournement du compositeur. La géométrie est testée sur données synthétiques.
- Déplacement souris câblé sur `start_dragging`, poignée distincte ; pas de
  qualification physique d'un glisser sur ce bureau. Pas de preuve Windows/macOS.
- Aucun reboot ni reconnexion de session effectué. Le mode `--widget` a été
  lancé réellement pour tester le démarrage capsule seule. L'entrée autostart
  réelle est **absente**, conformément au choix explicite de Kiunda ; ses variantes
  enable/disable sont testées dans des dossiers temporaires isolés.
- Charge GPU par processus, diagnostic thermique matériel, efficacité de
  refroidissement et attribution d'un modèle local précis : non qualifiés. Les
  données inconnues sont indiquées comme telles. Pas de seuil thermique universel.
- Pas de couverture mesurée, fuzzing guidé, test de charge soutenu ni essai longue
  durée. L'étude des couleurs inclut normes/calculs/aperçus, pas de test utilisateurs
  ni certification d'accessibilité complète.

## Ressources et installation finale

Compilation incrémentale bornée, deux jobs ; pas de build release lourd. Le cache
Tauri était relevé à 3708 Mio au départ du lot et 3912 Mio en fin de vérification ;
les installations utilisateur de 389 à 1131 Mio (versions intermédiaires conservées).
Supplément observé sur ces deux emplacements : environ **946 Mio**, hors petits
artefacts de tests et variations des caches. Aucun nettoyage automatique.
Le pic RAM cumulé de tous les processus n'est pas mesuré : objectif 3 Gio, pas
une limite système imposée.

Un échantillon `pidstat` de six secondes de l'instance debug intermédiaire a
observé 5,67 % d'un CPU en moyenne et environ 183 Mio de RSS pour le processus
principal. Il exclut les sous-processus WebKit et n'est pas un benchmark isolé,
ni une preuve de légèreté de production. Pas de gain de performance revendiqué.

Version finale : `~/.local/share/systempulse/0.1.0-local-20261123-widget-r3/`.
Le binaire MCP est installé à côté du desktop pour les configurations préparées,
mais aucun agent n'a été lancé ni configuré. Le lanceur utilisateur pointe vers
cette version ; sauvegardes `previous-launcher-*.txt` dans chaque nouvelle version.
Anciennes versions conservées, aucune base/cache de WebView supprimée.

SHA-256 desktop installé :
`0201c9a92cb1324cc4797084425476cadf1516c3de3d4e86291527f95fa4de53`.

Commandes utiles :

```sh
# Ouvrir normalement dashboard et capsule :
~/.local/bin/systempulse-desktop
# Ouvrir seulement la capsule (une seule instance à la fois) :
~/.local/bin/systempulse-desktop --widget
# Tester l'installateur sans écrire :
python3 scripts/install-user.py --dry-run --update --autostart disable
# Installation interactive : accord puis question autostart, non par défaut :
python3 scripts/install-user.py --update
```
