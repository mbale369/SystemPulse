# SystemPulse

Outil open source de surveillance de santé système : CPU, mémoire, GPU,
stockage, réseau, batterie, processus et diagnostics explicables.

Rust pour le cœur et les interfaces headless (CLI, MCP), Tauri 2 + TypeScript
pour l'application graphique.

## État réel

La Phase 01 (fondations) est terminée. La Phase 02 (V1) est **en cours** : les
composants principaux sont écrits et testés, mais la V1 n'est pas déclarée livrée.

| Composant | État |
| --- | --- |
| Workspace Rust, 7 crates | Compile, Clippy et tests verts sous Linux |
| Collecteur CPU/RAM/disques/réseau | Fonctionnel sur Linux, validé par collecte réelle |
| Diagnostics à hystérésis | Fonctionnel, testé |
| Historique SQLite | Écrit et testé, **non branché** à la CLI/GUI/MCP |
| Serveur MCP stdio, 10 outils lecture seule | Fonctionnel, testé sur protocole réel |
| CLI JSONL | Fonctionnel, testé |
| Interface Tauri + TypeScript | Compile, 6 vues validées en navigateur |
| Windows / macOS | Non exécutés |

Preuves et limites : [rapport de tests V1](docs/testing/V1_TEST_REPORT.md).

Suivi actuel : [correctifs et audits de sécurité](docs/testing/SECURITY_FOLLOWUP_2026-11-23.md),
[matrice de portabilité](docs/development/PORTABILITY.md) et
[préparation GitHub sans push](docs/development/GITHUB_PREPARATION.md).
Les tests fonctionnels passent, mais des avis de sécurité Rust restent ouverts.

## Développement

```sh
cd /home/kiunda/SystemPulse
. "$HOME/.cargo/env"
cargo check --workspace --locked
cargo test --workspace --locked
sh scripts/check.sh
```

Frontend :

```sh
cd apps/desktop
npm install       # première fois seulement
npm run check     # tsc --noEmit
npm test          # tests du modèle TypeScript
npm run build
```

App native :

```sh
cd apps/desktop/src-tauri
cargo check
```

### Voir l'interface

```sh
cd /home/kiunda/SystemPulse/apps/desktop
export PATH="$HOME/.cargo/bin:$PATH"   # requis sur cette machine
npm run tauri dev                      # fenêtre native + collecte réelle
# ou
npm run dev                            # navigateur sur 127.0.0.1:1420 (démonstration)
```

`cargo` n'est pas dans le `PATH` par défaut ici (rustup installé avec
`--no-modify-path`) : sans cet `export`, `npm run tauri dev` échoue sur
`cargo metadata`. Détails : [installation](docs/development/README.md).

Rust stable est géré par `rust-toolchain.toml`. Conserver `Cargo.lock` et
`package-lock.json`. Voir [installation](docs/development/README.md),
[architecture](docs/architecture/README.md),
[rapport environnement](docs/development/ENVIRONMENT_REPORT.md),
[plan V1](docs/development/V1_IMPLEMENTATION_PLAN.md) et
[contribution](CONTRIBUTING.md).

Licence en attente de validation par Kiunda : aucune licence open source n'est
encore accordée. Ne pas publier ni distribuer avant ce choix.
