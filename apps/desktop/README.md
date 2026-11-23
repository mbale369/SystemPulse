# Desktop — Tauri 2 + TypeScript

Application graphique locale. Backend Rust dans `src-tauri/`, interface
TypeScript dans `src/`.

## État

- Tauri 2.12.5 (CLI 2.12.4), interface TypeScript native (sans framework).
- Six vues : Vue d'ensemble, Processus, Diagnostics, Historique,
  Intégration IA, Paramètres.
- FR/EN, thèmes clair/sombre, responsive 320 → 1440 px, navigation clavier.
- Vite 8.3.5, TypeScript 7.0.2, uPlot 1.6.32.
- Icônes générées dans `src-tauri/icons/` depuis `app-icon.svg`.

## Commandes

```sh
cd apps/desktop
npm install                 # première fois
npm run dev                 # Vite sur 127.0.0.1:1420
npm run check               # tsc --noEmit
npm test                    # tests du modèle TypeScript
npm run build               # build de production dans dist/

cd src-tauri
cargo check                 # compilation de l'app native
cargo clippy --all-targets -- -D warnings
```

## Architecture

- Le crate `src-tauri` est un **workspace Cargo séparé** : il n'impose pas
  WebKit/GTK au socle headless (CLI, MCP).
- Trois commandes IPC seulement : `get_snapshot`, `get_health`, `set_profile`.
- Un unique thread `systempulse-sampler` possède le collecteur ; le frontend
  ne déclenche jamais de collecte, il lit un cache et en fait vieillir l'âge.
- Aucune permission plugin (`capabilities/main.json` → `permissions: []`).
- CSP restrictive, `bundle.active = false` (pas de packaging pour l'instant).

## Mode démonstration

Hors Tauri (`npm run dev`), le frontend détecte l'absence d'IPC via `isTauri()`
et affiche des fixtures marquées **DÉMONSTRATION — données fictives**.
Ce mode ne prouve pas la collecte réelle ; il valide la présentation.

## Test navigateur

`tests/browser.mjs` utilise un Playwright et Chrome déjà présents sur la
machine, sans téléchargement. Il exige `npm run dev` actif sur le port 1420.
Chemins paramétrables par `SYSTEMPULSE_PLAYWRIGHT` et `SYSTEMPULSE_CHROME`.
