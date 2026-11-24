# Rétroportage local de sécurité — 23 novembre 2026

Autorisé explicitement par Kiunda. Ce composant conserve **glib 0.18.5**, son API
et sa licence MIT d'origine (`LICENSE`). Il ne s'agit pas d'une version amont
0.20, ni d'une nouvelle version publiée par SystemPulse.

## Provenance

- Archive officielle : https://crates.io/api/v1/crates/glib/0.18.5/download
- SHA-256 vérifié avant extraction :
  `233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5`.
- Avis : https://rustsec.org/advisories/RUSTSEC-2024-0429.html
- Correctif officiel : https://github.com/gtk-rs/gtk-rs-core/pull/1343
- Commit amont fusionné : `05dff0ee696f9bcd8617cd48c4b812d046d440cb`.
- Diff officiel : https://github.com/gtk-rs/gtk-rs-core/pull/1343.diff

Seules deux lignes du code tiers sont modifiées, dans `src/variant_iter.rs` :

```diff
- let p: *mut libc::c_char = std::ptr::null_mut();
+ let mut p: *mut libc::c_char = std::ptr::null_mut();
- &p,
+ &mut p,
```

La fonction C variadique écrit dans le pointeur de sortie : un `&mut p` est
requis. La référence immutable permettait un comportement indéfini, notamment
avec optimisations. Aucun cache Cargo global ni SDK installé n'est modifié.

## Vérification et maintenance

Le workspace desktop applique `[patch.crates-io]` à ce chemin ; le workspace
headless exclut ce crate tiers. Un test desktop Linux exerce `next`, `next_back`,
`nth`, `nth_back` et `last`. Le profil dev optimise glib à niveau 2 pour couvrir
la condition de reproduction connue. La preuve réelle de compilation/test est
consignée dans le rapport de remédiation, pas déduite de ce document.

L'avis peut continuer à être signalé sur le numéro 0.18.5. Ne pas modifier ce
numéro ni ignorer l'avis dans le seul but de rendre un outil vert. Les scanners
peuvent traiter différemment un crate local : ce rétroportage doit donc toujours
être contrôlé explicitement, indépendamment de la sortie cargo-audit.

Retrait : dès qu'une version compatible de la pile Tauri/GTK utilise une glib
amont corrigée, supprimer le patch par un lot explicitement relu, conserver le
test et auditer le nouveau graphe. Les avis de maintenance `chrono` et
`proc-macro-error` ne sont pas résolus par ce correctif ponctuel.
