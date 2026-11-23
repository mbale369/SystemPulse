# Plan V1 — Phase 02

Le 4 novembre 2026. La Phase 02 est autorisée ; aucune publication ni modification
globale des clients IA. Conserver les fondations existantes.

## Audit initial

| Domaine | Présent | Manquant au départ |
| --- | --- | --- |
| Cargo | 7 crates, edition 2024, aucun cycle | Modèles et implémentation |
| Qualité | fmt, Clippy, check, 29 tests, CI 3 OS | Tests de charge et couverture |
| GUI | Tauri 2 + TypeScript, 6 vues, icônes générées | Packaging, tray, notifications |
| Collecte | CPU/RAM/disques/réseau Linux, GPU/batterie sysfs | SMART, températures GPU |
| Diagnostic | Règles temporelles explicables, hystérésis | Étendre aux GPU et E/S |
| Historique | SQLite complet et testé | **Intégration CLI/GUI/MCP** |
| MCP | 10 outils, test protocole réel | Clients IA non testés |
| Sécurité | Modèle de menace, contrôles testés | Audit cargo-audit/deny non exécuté |
| Git | main, aucun commit/remote ; fichiers non suivis | Licence encore à choisir |

## Lots, dépendances et validation

| Lot | Dépendances | Critères de validation | Risques / état |
| --- | --- | --- | --- |
| 1 Audit / décisions / menaces | Phase 01 | Sources, alternatives, contrat données | Fait (documenté) |
| 2 Design / fixtures | 1 | Six vues, FR/EN, clavier, clair/sombre, petits écrans | **Validé** en navigateur |
| 3 Modèles typés | 1 | Valeurs finies, unités, disponibilité, fraîcheur testées | **Validé** (4 tests) |
| 4 Collecteur + CLI Linux | 3 | Mesures réelles cohérentes, warmup, fakes, limites | **Validé** Linux (6 tests + collecte réelle) |
| 5 IPC / GUI réelle | 2,4 | Compilation Tauri, démarrage, renouvellement sans chevauchement | Compile ; **démarrage fenêtre non testé** |
| 6 Diagnostics | 3,4 | Pic isolé ignoré, durée, hystérésis, stale/Unknown | **Validé** (5 tests) |
| 7 SQLite | 3,6 | Transactions, permissions, limites, migrations, erreurs | Crate **validé isolément** (6 tests) ; **non branché** |
| 8 Alertes natives | 5,6,7 | Anti-spam, résolution, notifications désactivables | À faire |
| 9 MCP stdio | 4,6,7 | SDK officiel, dix outils, paramètres stricts, moindre partage | **Validé** sur protocole réel |
| 10 Clients IA | 9 | Vrai appel client disponible sans modifier sa config globale | À valider |
| 11 Plateformes | 4,5,9 | CI et qualification séparées | Windows/macOS non exécutés ici |
| 12 Audits / performances | Tous | Audits, tests, mesures et limites documentées | `cargo audit`/`deny` non exécutés |
| 13 Packaging | 5,12 | Artefact local vérifié ; aucune publication | À faire |

Chaque lot est testé avant intégration dépendante. Aucun résultat non exécuté
ne sera annoncé comme validé. Les fonctions inachevées seront listées dans
V1_FINAL_REPORT.md ; ce plan ne constitue pas une preuve de livraison.
Les preuves exécutées sont consignées dans `docs/testing/V1_TEST_REPORT.md`.

Retour arrière : fichiers actuels non commités conservés ; modifications locales
relisibles, pas de nettoyage, migration réelle ou suppression automatique.
