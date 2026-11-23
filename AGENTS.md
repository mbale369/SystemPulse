# SystemPulse — instructions locales

- Répondre en français. Phase 02 autorisée : développer la V1 progressivement, tests et preuves
  documentés pour chaque lot. Ne pas déclarer la V1 prête avant ses critères.
- Lire docs/architecture/README.md et docs/security/README.md avant développement.
- Rust stable, edition 2024 ; respecter rust-toolchain.toml et Cargo.lock.
- Conserver les noms Cargo préfixés systempulse- ; core ne doit pas masquer
  le crate standard Rust `core`. Pas de dépendance Tauri dans le socle headless.
  Le crate `apps/desktop/src-tauri` reste un workspace Cargo séparé.
- Frontend : TypeScript et npm, sans framework (Décision ADR 02).
- Vérifications locales sans effet externe : `sh scripts/check.sh`.
- Pas d'installation de dépendance sans accord ; aucun sudo pour Cargo/npm.
- MCP futur local, lecture seule et stdout réservé au protocole.
- Ne supprimer aucun fichier/configuration ; pas de commit/push/publication
  automatique. Licence non choisie : ne pas créer LICENSE sans validation.
- Conserver le rapport Phase 01 comme historique. Suivre
  docs/development/V1_IMPLEMENTATION_PLAN.md et docs/testing/V1_TEST_REPORT.md
  (résultats réellement exécutés, limites comprises).
- Ne pas activer la feature rusqlite `fallible_uint` sans décision explicite :
  les conversions u64/i64 passent par des erreurs métier explicites.
- Les dépendances locales V1 (Rust, frontend, tests/audits isolés) ont été
  autorisées le 4 novembre 2026 ; pas de sudo, cloud payant ni publication.
- Windows et macOS ne sont pas validés tant qu'aucune exécution réelle n'a eu
  lieu ; le workflow CI ne remplace pas une preuve locale.
