# Installation et travail local

## Rust

Source officielle : https://rust-lang.org/tools/install/ . Vérifier le script
https://sh.rustup.rs avant exécution ; installation par utilisateur via rustup,
profil minimal, stable, rustfmt, Clippy et rust-analyzer. Aucun sudo nécessaire.
L'installation de cette phase utilise `--no-modify-path` :

```sh
. "$HOME/.cargo/env"
cd /home/kiunda/SystemPulse
rustup show
sh scripts/check.sh
cargo run -p systempulse-cli --locked
cargo run -p systempulse-mcp-server --locked
```

Les deux binaires affichent uniquement un message de fondation sur stderr et
quittent. Ils ne représentent pas une implémentation CLI/MCP.
Ouvrir ce dossier dans un éditeur existant ; configurer rust-analyzer depuis
`~/.cargo/bin/rust-analyzer` si l'éditeur le prend en charge.

## Prérequis desktop (à installer dans un terminal autorisé)

Documentation officielle vérifiée le 4 novembre 2026 :
https://v2.tauri.app/start/prerequisites/ . Sur cette machine, seuls les paquets
suivants manquaient ; apt résout aussi leurs dépendances transitives :

```sh
sudo apt-get install --no-install-recommends libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev
```

Ne pas installer une seconde distribution Node : Node 24.22.2 et npm 11.19.1
sont déjà disponibles. Tauri CLI et Vite sont des dépendances locales de
`apps/desktop` (voir `package-lock.json`), pas des installations globales.

## Application desktop

```sh
cd apps/desktop
npm install          # première fois seulement
npm run check        # tsc --noEmit
npm test             # tests du modèle TypeScript
npm run build
cd src-tauri && cargo check
```

`src-tauri` est un workspace Cargo séparé : la compilation Tauri n'impose pas
WebKit/GTK au socle headless.

### Lancer l'application pour voir le rendu

```sh
cd /home/kiunda/SystemPulse/apps/desktop
export PATH="$HOME/.cargo/bin:$PATH"   # rustup a été installé avec --no-modify-path
npm run tauri dev
```

Cette commande démarre Vite sur `127.0.0.1:1420`, compile le binaire Rust puis
ouvre la fenêtre native **SystemPulse**. Fermer la fenêtre ou `Ctrl+C` arrête
les deux processus. Un premier lancement peut prendre 1 à 2 minutes
(compilation de Tauri/WebKit) ; les suivants sont quasi instantanés.

**Important :** `cargo` n'est pas dans le `PATH` par défaut de cette machine
(rustup a été installé avec `--no-modify-path`). Sans
`export PATH="$HOME/.cargo/bin:$PATH"`, `npm run tauri dev` échoue avec
`failed to run cargo metadata ... No such file or directory`. La même variable
est nécessaire pour `cargo check`, `cargo test`, `sh scripts/check.sh`.

Alternative pour ne voir que l'interface web (données de démonstration) :

```sh
cd /home/kiunda/SystemPulse/apps/desktop
npm run dev      # puis ouvrir http://127.0.0.1:1420
```

Ce mode n'utilise ni Tauri ni le collecteur réel : il affiche des fixtures
marquées **DÉMONSTRATION**.

Pour un binaire autonome (sans serveur Vite) :

```sh
cd /home/kiunda/SystemPulse/apps/desktop
npm run build                                   # remplit dist/
cd src-tauri
cargo build --release
./target/release/systempulse-desktop
```

La fenêtre a été ouverte et vérifiée sur cette machine (Wayland/GNOME). Le
packaging (`.deb`, AppImage) et le démarrage sur Windows/macOS restent non
qualifiés.

## Pour venir de Python

- Cargo.toml décrit les packages ; Cargo.lock fixe la résolution des dépendances.
- Un workspace partage toolchain, verrouillage et compilation entre crates.
- Une bibliothèque expose lib.rs ; un exécutable démarre dans main.rs.
- `cargo check` vérifie les types sans générer l'exécutable final ; `cargo test`
  compile et exécute les tests. rustfmt formate ; Clippy relève des problèmes.
- Ownership et emprunts remplacent beaucoup de copies implicites ; commencer
  par des types simples et Result pour les erreurs. Ne pas ajouter async avant besoin.
- Aucun venv Rust : rustup gère les compilateurs, Cargo les dépendances du projet.

## Git

Le répertoire .git fourni par l'environnement était vide et en lecture seule
pour le bac à sable. L'initialisation autorisée hors bac à sable a réussi :
branche `main`, aucun commit ni remote. Configuration globale conservée.
L'identité sera à vérifier localement avant le premier commit autorisé ; aucune
adresse personnelle n'est enregistrée dans la documentation.
