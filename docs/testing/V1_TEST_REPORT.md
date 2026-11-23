# SystemPulse — rapport de tests V1

> Suivi du 23 novembre 2026 : [correctifs, audits de dépendances et Git](SECURITY_FOLLOWUP_2026-11-23.md).
> Le contenu ci-dessous conserve les preuves historiques du 5 novembre ; les
> nouveaux audits Rust ont détecté des avis ouverts, voir le suivi.

Date des exécutions : 5 novembre 2026 (Africa/Lubumbashi).
Périmètre : état réel du dépôt après rétablissement de la compilation.
Ce rapport ne consigne que des commandes réellement exécutées sur cette machine.

## 1. Environnement d'exécution

| Élément | Valeur relevée |
| --- | --- |
| OS | Ubuntu 26.10, x86_64 |
| CPU | 8 threads (`nproc`) |
| RAM | 14 GiB, 7,4 GiB disponibles au relevé |
| Disque `/` | 468 Gio, 354 Gio disponibles |
| rustc | 1.99.1 (58cf8f9c3, 2026-10-13) |
| cargo | 1.99.1 (5f94df478, 2026-08-27) |
| Node.js | 24.22.2 |
| npm | 11.19.1 |
| Navigateur de test | Chrome headless via Playwright existant |

Aucune installation, aucun `sudo`, aucun accès réseau de collecte n'a été effectué.

## 2. Résultat global

| Contrôle | Commande | Résultat |
| --- | --- | --- |
| Formatage | `cargo fmt --all -- --check` | **PASS** |
| Clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **PASS** (0 avertissement) |
| Tests Rust | `cargo test --workspace --locked` | **PASS** — 29 tests, 0 échec |
| Compilation | `cargo check --workspace --locked` | **PASS** |
| TypeScript | `npx tsc --noEmit` | **PASS** |
| Tests frontend | `node --experimental-strip-types tests/model.test.ts` | **PASS** — 5/5 |
| Build frontend | `npm run build` | **PASS** — 14 modules, 68,49 kB JS |
| App Tauri (Rust) | `cargo check` dans `apps/desktop/src-tauri` | **PASS** |
| App Tauri (exécution) | `npm run tauri dev` | **PASS** — fenêtre native ouverte |
| fmt/clippy Tauri | `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` | **PASS** |
| Test navigateur | `node tests/browser.mjs` | **PASS** — 11 vérifications |
| Protocole MCP réel | `python3 scripts/test-mcp.py` | **PASS** — handshake, 10 outils, refus, redémarrage |
| Collecte réelle CLI | `./target/debug/systempulse snapshot --count 1 --profile detailed` | **PASS** — JSON valide |

## 3. Répartition des tests Rust

| Crate | Tests | Objet couvert |
| --- | --- | --- |
| `systempulse-core` | 4 | valeurs non finies, fraîcheur, âge monotone, capteur absent ≠ 0 |
| `systempulse-platforms` | 1 | bornage et nettoyage des noms de processus |
| `systempulse-collector` | 5 | cache, erreurs, backoff, exclusion processus, bornes de profils |
| `systempulse-alerts` | 5 | pic ignoré, hystérésis + acquittement, recul d'horloge, données périmées, seuils invalides |
| `systempulse-history` | 6 | insertion/rétention, bornes et horloge, agrégats, schéma futur, disque plein, permissions et symlinks |
| `systempulse-mcp-server` | 6 | outils en lecture seule, refus privés/exécution, staleness, quota, garde de stdout |
| `systempulse-cli` | 2 | `--help` sans collecte, rejet d'arguments hostiles |
| **Total** | **29** | |

## 4. Test navigateur — détail

`apps/desktop/tests/browser.mjs` sur `http://127.0.0.1:1420` (Vite local, données de démonstration) :

1. Bandeau « DÉMONSTRATION — données fictives » visible.
2. Un seul `h1` par écran.
3. Mise à jour réelle des valeurs après collecte des fixtures.
4. Graphique uPlot présent après deux points.
5. Navigation des six vues (Vue d'ensemble, Processus, Diagnostics, Historique, Intégration IA, Paramètres).
6. Bascule FR → EN effective sur `html[lang]`.
7. Bascule thème sombre → clair sur `html[data-theme]`.
8. Filtre de processus (1 résultat attendu) et absence de résultat sur charge `<script>`.
9. Absence de débordement horizontal à 320 / 768 / 1024 / 1440 px.
10. Focus clavier actif après `Tab`.
11. Aucune erreur console, aucune requête externe (hors `127.0.0.1` et `data:`).

Captures : `apps/desktop/test-artifacts/dashboard-demo.png`, `mobile-demo.png`
(dossier exclu de Git, régénérable).

## 5. Test MCP stdio réel

`python3 scripts/test-mcp.py` contre `target/debug/systempulse-mcp` :

- `initialize` négocié avec `2025-11-25` (handshake de compatibilité).
- `tools/list` renvoie exactement 10 outils, tous `readOnlyHint = true`,
  `inputSchema.additionalProperties = false`, et dotés d'un `outputSchema`.
- `get_top_processes` refusé sans `--share-processes`.
- Paramètres hostiles refusés : `limit: 51`, `path: /etc/passwd`,
  `command: touch /tmp/unauthorized`, `limit: 0`, période inversée.
- 8 appels concurrents servis sans corruption de flux.
- Ligne d'entrée de 70 Kio rejetée sans empoisonner la requête suivante.
- Arrêt propre sur EOF et redémarrage vérifié.

## 6. Défauts trouvés et corrigés pendant cette session

| ID | Défaut | Correction |
| --- | --- | --- |
| B1 | `Content::text` inexistant dans rmcp 3.8.0 | remplacé par `ContentBlock::text` |
| B2 | 8 erreurs `u64: FromSql/ToSql` dans `history` | conversions `i64` explicites avec erreur métier `InvalidInput` |
| B3 | `cargo fmt --check` en échec sur `mcp-server` | `cargo fmt --all` |
| B4 | `collapsible_if` dans `alerts` | chaîne `if let ... && ...` |
| B5 | icône Tauri absente, app desktop non compilable | jeu d'icônes généré + `bundle.icon` renseigné |
| B6 | deux `collapsible_if` supplémentaires dans `history` | mêmes corrections |
| B7 | `<select>` sans nom accessible (label englobant) | `aria-label` explicite |
| B8 | `content.focus()` faisait sortir le focus du document | focus conservé sur le bouton de navigation actif |
| B9 | icônes absentes → `npm run tauri dev` non qualifié | icônes générées, fenêtre démarrée et vérifiée |
| B10 | `cargo` hors `PATH` → `tauri dev` échoue sur `cargo metadata` | documenté : `export PATH="$HOME/.cargo/bin:$PATH"` |

## 7. Limites de ces preuves

- **Linux uniquement.** Windows et macOS ne sont pas exécutés ; le workflow CI
  les déclare mais aucune exécution distante n'a eu lieu.
- **Pas de test de charge ni de longue durée.** Les quotas MCP sont testés
  fonctionnellement, pas en concurrence réelle soutenue.
- **Fenêtre Tauri ouverte et vérifiée, mais rendu non capturé.** Le compositeur
  Wayland ne compose pas la surface XWayland vers une capture X11, et le portail
  de capture GNOME exige une confirmation interactive. Sont vérifiés : titre de
  fenêtre, dimensions, état `IsViewable`, connexion de la webview au serveur
  Vite, absence d'erreur. Le rendu pixel n'est pas une preuve automatisée ; le
  rendu de l'interface est validé séparément dans le navigateur (section 4).
- **`cargo` absent du `PATH`** (rustup installé avec `--no-modify-path`) :
  `npm run tauri dev` échoue sans `export PATH="$HOME/.cargo/bin:$PATH"`.
- **`crates/history` n'est branché par aucun composant** : ses tests couvrent le
  crate isolé, pas une intégration GUI/CLI/MCP.
- **Capteurs matériels non qualifiés** : GPU, SMART et certaines températures
  restent explicitement indisponibles sur cette machine.
- **Couverture de code non mesurée** : aucun outil de couverture exécuté.
- L'audit de dépendances (`cargo audit` / `cargo deny`) **n'a pas été exécuté**.

## 8. Reproduction

```sh
cd /home/kiunda/SystemPulse
. "$HOME/.cargo/env"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --workspace --locked

cd apps/desktop
npx tsc --noEmit
node --experimental-strip-types tests/model.test.ts
npm run build

# App Tauri (compilation seule)
cd src-tauri && cargo check

# MCP réel
cd /home/kiunda/SystemPulse
python3 scripts/test-mcp.py
```

## 9. Statut

**READY sous Linux pour la suite du développement headless.**
Les limitations des sections 7 restent ouvertes et ne doivent pas être
présentées comme validées.
