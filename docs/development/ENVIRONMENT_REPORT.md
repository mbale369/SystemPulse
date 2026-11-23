# SystemPulse — rapport environnement Phase 01

Date : 4 novembre 2026 (Africa/Lubumbashi).
Statut actuel : **READY pour les fondations Phase 01 sous Linux**.
Les dépendances natives ont été installées par l’utilisateur puis vérifiées.
Le relevé initial ci-dessous est conservé comme historique ; la validation
complémentaire en fin de rapport fait foi pour l’état actuel.
Tauri lui-même reste non initialisé et non qualifié fonctionnellement.

## 1. OS et matériel

Ubuntu 26.10 (Stonking Stingray), x86_64 ; AMD Ryzen 3 PRO 4450U,
4 cœurs / 8 threads. RAM : environ 14 Gio, dont 3,7 Gio disponibles au relevé.
Swap : 4 Gio. Partition du projet : 468 Gio, 363 Gio disponibles.
Ces valeurs sont un relevé ponctuel, pas une mesure de capacité en charge.

## 2. Logiciels et versions

| Composant | Version détectée | Statut | Action nécessaire |
| --- | --- | --- | --- |
| Git | 2.55.0 | Présent | Dépôt initialisé, aucun commit |
| GCC | 15.3.0 | Présent | Aucune |
| Clang | Absent du PATH | Optionnel ici | Pas d'installation nécessaire au socle |
| Make | 4.4.1 | Présent | Aucune |
| pkg-config | 2.5.1 | Présent | Aucune |
| curl | 8.20.0 | Présent | Aucune |
| wget | 1.25.0 | Présent | Aucune |
| file | 5.47 | Présent | Aucune |
| CMake | 4.3.4 | Présent | Aucune |
| gh | 2.46.0 | Présent | Authentification non sollicitée ; aucune publication |
| Rust/rustup | Absents avant intervention | Installés via rustup officiel | Charger ~/.cargo/env |
| rustc | 1.99.1, stable, 2026-10-13 | Validé | Aucune |
| Cargo | 1.99.1 | Validé | Aucune |
| rustfmt | 1.10.0-stable | Validé | Aucune |
| Clippy | 0.1.99 | Validé | Aucune |
| rust-analyzer | 1.99.1 | Installé, version vérifiée | Intégration éditeur non testée |
| Node.js | 24.22.2 | Présent via ~/.local/bin/node | Conserver |
| npm | 11.19.1 | Présent | Gestionnaire frontend retenu |
| pnpm | 11.25.0, runtime Codex | Détecté | Non utilisé par le projet |
| yarn / bun / code | Absents du PATH | Facultatifs | Aucune installation |
| Tauri CLI | Non trouvé dans les paquets npm globaux ni sur PATH | Non initialisé | Dépendance locale future |
| build-essential | 12.12ubuntu5 | Installé | Aucune |
| libssl-dev | 4.0.3-1ubuntu1 | Installé | Compatibilité Tauri à compiler ultérieurement |
| librsvg2-dev | 2.62.1+dfsg-1ubuntu1 | Installé | Aucune |
| libwebkit2gtk-4.1-dev | Absent ; candidat 2.52.6-1ubuntu1 | Manquant | Installation apt autorisée, à effectuer |
| libxdo-dev | Absent ; candidat 1:3.20160805.1-5.1build1 | Manquant | Installation apt autorisée, à effectuer |
| libayatana-appindicator3-dev | Absent ; candidat 0.5.94-1.1 | Manquant | Installation apt autorisée, à effectuer |

Aucun ~/.cargo ou ~/.rustup préexistant trouvé. Aucun second Rust ajouté.
Aucun autre framework graphique installé. L'audit porte sur le PATH, les
emplacements Rust usuels et les paquets Debian, pas tous les disques.

## 3. Installations et autorisations

Accord utilisateur reçu pour Rust stable et les trois dépendances Tauri.
Rust installé par utilisateur dans ~/.cargo et ~/.rustup, profil minimal,
avec rustfmt, Clippy et rust-analyzer ; cible x86_64-unknown-linux-gnu seulement.
Aucune nightly, modification shell ou mise à jour Node. Activer la session avec
`. "$HOME/.cargo/env"` ; aucun changement persistant de PATH effectué.

Script officiel téléchargé puis lu avant exécution, via HTTPS :
https://sh.rustup.rs ; copie d'audit temporaire /tmp/systempulse-rustup-init.sh.
SHA-256 : d890e2fef6fa991b19cde2e608199bc1dca4e37f41f884edf8849ff693048de6.
Cette empreinte trace le fichier reçu ; ce n'est pas une signature indépendante.

Commande exécutée :
`sh /tmp/systempulse-rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable --component rustfmt,clippy,rust-analyzer`
avec les overrides des serveurs rustup retirés pour utiliser les sources officielles.

La simulation apt prévoit 23 nouveaux paquets et 3 mises à jour libsoup,
aucune suppression. L'installation réelle n'a pas commencé : `sudo -n` refuse
sans authentification interactive. Aucun paquet système installé par cette phase.
Commande à exécuter dans un terminal utilisateur :

```sh
sudo apt-get install --no-install-recommends libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev
```

Après installation, vérifier dpkg et pkg-config, puis ultérieurement compiler
et démarrer une application Tauri minimale avant de conclure à sa compatibilité.
Aucun mot de passe n'a été demandé dans le chat.

## 4. Structure finale

Emplacement : /home/kiunda/SystemPulse, dossier de travail explicitement ouvert.
~/Projects existe mais n'est pas un dossier autorisé en écriture dans cette
session ; choix conservateur de garder les fondations ici sans doublon ni déplacement.

```text
SystemPulse/
  apps/desktop/README.md
  crates/{core,collector,platforms,alerts}/Cargo.toml + src/lib.rs
  crates/{mcp-server,cli}/Cargo.toml + src/main.rs + tests/entrypoint.rs
  tests/README.md
  docs/architecture/README.md
  docs/development/{README.md,ENVIRONMENT_REPORT.md}
  docs/security/README.md
  scripts/check.sh
  .github/workflows/ci.yml
  Cargo.toml, Cargo.lock, rust-toolchain.toml
  .gitignore, .gitattributes, .editorconfig
  README.md, CONTRIBUTING.md, SECURITY.md, AGENTS.md
```

Six crates réelles, édition 2024, resolver 3, zéro dépendance externe,
zéro cycle, publication Cargo désactivée. Workspace partagé et Cargo.lock généré
hors ligne. Toolchain `stable` volontairement mobile ; version testée 1.99.1.
Une future mise à jour stable nécessitera une nouvelle validation.
LICENSE absent conformément au choix utilisateur « à décider plus tard ».
Aucun droit open source présumé avant validation de cette licence.

## 5. Vérifications exécutées

| Commande / contrôle | Résultat local Linux |
| --- | --- |
| rustc --version | 1.99.1 |
| cargo --version | 1.99.1 |
| rustup show | Stable active, cible Linux native |
| rustup component list --installed | rustfmt, Clippy, rust-analyzer et socle présents |
| cargo generate-lockfile --offline | Succès |
| cargo fmt --all -- --check | Succès |
| cargo clippy --workspace --all-targets --locked -- -D warnings | Succès, aucun avertissement |
| cargo test --workspace --locked | Succès, 2 tests d'intégration ; aucune logique métier testée |
| cargo check --workspace --locked | Succès |
| cargo metadata --offline --locked --no-deps --format-version 1 | 6 membres, aucune dépendance |
| git init -b main | Succès après autorisation hors bac à sable |
| git status --short --branch | main sans commit ; fichiers nouveaux non suivis |
| git remote -v | Aucun remote |
| git diff --check | Succès ; ne couvre pas les nouveaux fichiers non suivis |

CI minimale préparée pour ubuntu-latest, windows-latest et macos-latest.
Workflow non publié, non exécuté sur GitHub ; Windows/macOS non validés.
Aucune app Tauri initialisée, compilée ou démarrée. Pas de package-lock.json
artificiel : il sera créé avec le vrai frontend npm après accord.

## 6. Problèmes rencontrés

DNS bloqué dans le bac à sable pour rustup : téléchargement/install autorisés
hors bac à sable ensuite réussis. .git initialement vide et protégé :
initialisation hors bac à sable autorisée et réussie, sans écraser de dépôt.
Authentification sudo interactive manquante : dépendances Tauri en attente.
Aucun fichier ou cache existant supprimé, aucun commit, push ou publication.

## 7. Limites de compatibilité

Seul le socle Rust sur Ubuntu 26.10 x86_64 est validé. Aucun collecteur,
accès matériel, performance, moteur MCP ou affichage n'est qualifié.
Ubuntu 26.10 et OpenSSL 4 nécessiteront une compilation Tauri effective ; la
présence de paquets ne suffit pas. Le frontend TypeScript reste à initialiser.
Aucun éditeur supplémentaire installé ni intégration rust-analyzer qualifiée.

## 8. Ouvrir et compiler

```sh
cd /home/kiunda/SystemPulse
. "$HOME/.cargo/env"
rustup show
sh scripts/check.sh
cargo build --workspace --locked
```

Ouvrir ce dossier dans l'éditeur existant de votre choix. Les binaires de
fondation peuvent être lancés via `cargo run -p systempulse-cli --locked` et
`cargo run -p systempulse-mcp-server --locked` ; ils affichent un message sur
stderr puis quittent, sans collecte ni serveur actif.

## 9. Conseils Python → Rust

Voir docs/development/README.md : Cargo remplace la gestion de projet/build,
rustup gère les compilateurs plutôt qu'un venv. Commencer par types, ownership,
emprunts et Result. Séparer bibliothèque et exécutable ; garder les tests près
des crates. Ne pas ajouter async, cache ou dépendances avant besoin démontré.

## 10. Préparation Phase 02

Le développement Rust headless peut commencer après une nouvelle demande.
Aucune fonctionnalité V1 commencée. Pour le desktop, finir l'installation apt
puis initialiser et valider Tauri séparément. Avant publication, choisir la
licence et le canal privé de signalement sécurité ; aucun compte GitHub
ni remote n'a été sollicité pour cette phase locale.

Sources officielles consultées le 4 novembre 2026 :
- [Installation Rust](https://rust-lang.org/tools/install/)
- [Prérequis Tauri 2](https://v2.tauri.app/start/prerequisites/)
- [Workspaces Cargo](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Action checkout](https://github.com/actions/checkout)

**État initial : PARTIALLY_READY** — avant installation des dépendances natives.


## Validation complémentaire après installation utilisateur — 4 novembre 2026

Tous les paquets requis sont maintenant `install ok installed` selon dpkg :
libwebkit2gtk-4.1-dev 2.52.6-1ubuntu1, libxdo-dev
1:3.20160805.1-5.1build1 et libayatana-appindicator3-dev 0.5.94-1.1.
Les prérequis déjà présents ont aussi été revérifiés.

Versions résolues par pkg-config : WebKit et JavaScriptCore 2.52.6,
GTK 3.24.52, Ayatana AppIndicator 0.5.94, OpenSSL 4.0.3, librsvg 2.62.1.
`libxdo` n’a pas de module pkg-config sous ce nom ; sa disponibilité a été
vérifiée directement avec xdo.h et l’édition de liens `-lxdo`.

Un programme C temporaire `/tmp/systempulse-native-smoke.c` a été compilé avec
`cc -Wall -Wextra -Werror`, les cflags/libs pkg-config et `-lxdo`.
Il inclut les headers WebKit, GTK, AppIndicator, OpenSSL, librsvg et xdo,
appelle leurs fonctions/types de version puis quitte. Compilation, édition de
liens et exécution réussies, sans initialiser de fenêtre ni accéder au matériel.
Versions runtime cohérentes ; xdo annonce 3.20160805.1.
Ce test ne constitue pas une compilation Rust/Tauri ou un test de rendu GUI.

Contrôles relancés avec succès :
- rustc 1.99.1, Cargo 1.99.1, rustup stable actif ;
- cargo fmt --all -- --check ;
- cargo clippy --workspace --all-targets --locked -- -D warnings ;
- cargo test --workspace --locked : deux tests d’intégration réussis ;
- cargo check --workspace --locked ;
- cargo build --workspace --locked --offline ;
- Node v24.22.2 et npm 11.19.1 disponibles.

Aucune dépendance ajoutée au projet, aucune fonctionnalité métier créée.
CI distante, Windows, macOS, intégration éditeur et application Tauri non testés.
Licence toujours à décider ; aucun commit ni push.

**READY — environnement des fondations Phase 01 validé sous Linux.**
La qualification de l’application Tauri nécessitera son initialisation puis
sa compilation et son démarrage, dans un périmètre ultérieur autorisé.

## Complément — 5 novembre 2026 : application Tauri démarrée

L'application native a depuis été initialisée, compilée et **démarrée** sur
cette machine (Wayland/GNOME, XWayland disponible). Le crate de fondation
Phase 01 compilait déjà ; cette section consigne la qualification qui manquait.

| Contrôle | Résultat |
| --- | --- |
| `cargo check` dans `apps/desktop/src-tauri` | Succès |
| `cargo clippy --all-targets -- -D warnings` (desktop) | Succès |
| `cargo build` (desktop) | Succès, ~1 min 32 s au premier build |
| `npm run tauri dev` | Succès : Vite + binaire Rust + fenêtre native |
| Fenêtre ouverte | Oui — titre « SystemPulse », 1299 × 699, état `IsViewable` |
| Webview → Vite | Connexion TCP établie (WebKitNetworkProcess) |
| Icônes Tauri | Jeu complet généré dans `src-tauri/icons/` |

**Piège rencontré :** `cargo` n'est pas dans le `PATH` par défaut, car rustup a
été installé avec `--no-modify-path` (choix de la Phase 01). Sans
`export PATH="$HOME/.cargo/bin:$PATH"`, `npm run tauri dev` échoue sur
`failed to run cargo metadata --no-deps --format-version 1: No such file or directory`.
C'est le seul blocage de lancement ; aucune modification système n'a été faite.

**Non qualifié :** le rendu visuel n'a pas pu être capturé automatiquement
(le compositeur Wayland ne compose pas la surface XWayland vers X11, et le
portail de capture GNOME exige une confirmation interactive). L'ouverture de la
fenêtre, la connexion de la webview et l'absence d'erreur sont en revanche
vérifiées. Le packaging (`.deb`, AppImage) et les plateformes Windows/macOS
restent non testés.
