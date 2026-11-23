# Tests

Les tests exécutables résident dans `crates/*/tests` pour être découverts par
`cargo test --workspace`. Le dossier racine accueillera seulement les scénarios
transversaux lorsqu'un harness Cargo explicite sera nécessaire.

Phase 01 : deux tests de démarrage des binaires vérifient leur sortie et la
préservation de stdout pour le futur transport MCP. Aucun test matériel ou réseau.
