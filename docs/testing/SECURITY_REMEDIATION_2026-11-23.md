# Remédiation de sécurité — 23 novembre 2026

## Conclusion actuelle

Quatre défauts connus traités : rmcp, url, glib et le nouvel avis cookie publié
pendant ce lot. **Aucun avis de vulnérabilité détecté sur les crates du registre**
dans les deux lockfiles à l'issue de l'audit actualisé. Deux avis de maintenance
distincts restent ouverts : chrono et proc-macro-error. Ce n'est pas une promesse
d'absence de vulnérabilité inconnue, ni une déclaration prêt production.

L'audit des versions du registre ne suffit pas pour glib, devenu une dépendance
locale : sa correction est attestée séparément par la provenance exacte et un
test ciblé avec optimisations. Aucun numéro de version falsifié, aucun avis
ignoré, aucun SDK/cache global modifié pour faire disparaître une alerte.

## Correctifs

| Avis | Avant | Correction et preuve |
| --- | --- | --- |
| [RUSTSEC-2026-0462](https://rustsec.org/advisories/RUSTSEC-2026-0462) | rmcp 3.8.0 | Version officielle **3.9.0**, options server/stdio conservées, fonctionnalités clientes HTTP toujours désactivées ; protocole réel testé |
| [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592) | url 2.5.8 | Version officielle **2.5.9**, test de `Url::origin()` sur 10 000 préfixes `blob:`, origine opaque sans récursion |
| [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) | glib 0.18.5 | Rétroportage local des **deux lignes officielles** gtk-rs PR 1343, API/version/licence conservées ; `VariantStrIter` testé avec glib optimisé niveau 2 |
| [RUSTSEC-2026-0594](https://rustsec.org/advisories/RUSTSEC-2026-0594.html) | cookie 0.18.2 | Avis découvert lors de l'actualisation RustSec du lot ; version officielle **0.18.3**. Aucun test par fork/nonce exécuté ; correction fondée sur version officielle et compilation/régression de l'application |

Sources officielles lues : avis RustSec ci-dessus, métadonnées crates.io rmcp
3.9.0/url 2.5.9/cookie 0.18.3/glib 0.18.5, PR gtk-rs
https://github.com/gtk-rs/gtk-rs-core/pull/1343 et son diff.

Les mises à jour Cargo sont ciblées. Elles ajoutent les dépendances cryptographiques
requises par rmcp 3.9.0 ; `form_urlencoded` passe à 1.2.3 avec url et une liaison
de résolution cssparser-macros/syn est ajustée par Cargo. Pas de `cargo update`
global ni de migration majeure de Tauri. Les changements de lockfile ont été relus.

## glib : provenance et maintien du rétroportage

L'API GTK 3 attend glib 0.18 : imposer 0.20 seul aurait créé une incompatibilité.
Kiunda a donc autorisé explicitement le rétroportage officiel local.

- Archive crates.io glib 0.18.5, SHA-256 vérifié avant extraction :
  `233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5`.
- **121 fichiers d'origine** comparés à l'archive : tous identiques sauf les
  deux changements attendus de `src/variant_iter.rs`. Licence MIT d'origine
  préservée ; la licence du projet SystemPulse n'est pas modifiée.
- Empreinte du fichier corrigé :
  `a0f5ee8acb8faa089bcdfbc9a57372609fce7654026ccef7d9a224d05a654ccc`.
- `scripts/verify-glib-backport.py` reproduit cette comparaison, vérifie le
  câblage `[patch.crates-io]` et le profil d'optimisation. Il ne télécharge rien.
- Le test Linux exerce next/next_back/nth/nth_back/last sur une chaîne GVariant.
- Aucun `unsafe` ajouté au code SystemPulse ; les deux lignes corrigées sont
  dans le code tiers déjà unsafe, conservé sous `vendor/glib-0.18.5`.

Voir `vendor/glib-0.18.5/SYSTEMPULSE_PATCH.md`. Ce rétroportage doit être maintenu
jusqu'à une pile Tauri/GTK compatible utilisant une version amont corrigée.
Ne pas supprimer le contrôle de provenance sous prétexte que cargo-audit ne
signale plus ce package local.

## Avis restant ouverts

- **RUSTSEC-2026-0543, chrono 0.4.45 non maintenu** : dépendance de rmcp et de
  l'intégration chrono04 de schemars activée par rmcp. Le projet n'appelle pas
  directement chrono. Aucune version corrigée annoncée ; retirer cette dépendance
  demande une évolution compatible du SDK, pas un simple changement de version.
- **RUSTSEC-2024-0370, proc-macro-error 1.0.4 non maintenu** : introduit par
  glib-macros/gtk3-macros. Pas de version corrigée annoncée ; changement à effectuer
  avec la chaîne de macros/GTK compatible. Avertissement de maintenance, pas une
  preuve d'exploitation runtime.

Les avertissements restent visibles. Un audit configuré avec `--deny warnings`
ne doit donc pas être présenté comme réussi. Pas de remplacement par un fork
inconnu, de migration GTK 4 ou de réécriture temporelle implicite dans ce lot.

La dépendance locale glib révèle aussi **34 avertissements de compilation**
anciens (parenthèses et syntaxe de lifetimes) avec le Rust actuel ; ils ne sont
pas masqués ni corrigés arbitrairement pour préserver le diff de deux lignes.
Clippy du code applicatif passe avec `-D warnings`, mais la sortie globale n'est
pas « zéro avertissement tiers ».

## Vérifications réellement exécutées

| Contrôle | Résultat |
| --- | --- |
| cargo-audit 0.23.0 avec actualisation RustSec | **1 562 avis** chargés ; nouvel avis cookie découvert puis corrigé |
| Audit final racine, `--no-fetch --no-yanked` | 0 vulnérabilité signalée sur le périmètre registre, 1 avertissement chrono |
| Audit final desktop, mêmes options | 0 vulnérabilité signalée sur le périmètre registre, 2 avertissements chrono/proc-macro-error ; glib local contrôlé séparément |
| `sh scripts/check.sh`, hors ligne, deux jobs, timeout 150 s | PASS : fmt, Clippy/check, **31 tests Rust headless** ; avertissements tiers glib visibles |
| Test desktop final avec custom-protocol et `--locked` | **6 tests PASS**, dont URL blob imbriquée et itérateur glib optimisé |
| Build headless puis `python3 scripts/test-mcp.py` | PASS : handshake, dix outils, paramètres privés/hostiles, concurrence bornée, entrée 70 Kio, EOF et redémarrage ; environ 3,35 s |
| TypeScript / tests frontend | PASS : **25 tests** |
| Tests navigateur widget | PASS : thèmes, gestes, son borné, consentement MCP, préférences ; contexte isolé, aucune installation de navigateur |
| Tests Python installateur et gardes de sécurité | **11 tests PASS**, dont versions minimales verrouillées, options stdio, empreinte glib, absence d'ignore d'avis |
| Vérification exhaustive du rétroportage | PASS : archive, 121 fichiers et différence exacte de deux lignes |
| Build natif final | PASS avec cookie 0.18.3, frontend embarqué, deux jobs ; compilation finale environ 28 s |
| Validation menu/autostart, `git diff --check` | PASS |
| Contrôle AT-SPI de la nouvelle version installée | PASS : capsule seule, agrandir/replier, masquer/réafficher et restauration dashboard ; journal de lancement vide au contrôle |
| Gitleaks avec masquage complet, sources et vendor inclus | Aucun secret détecté au contrôle |

`--no-yanked` signifie que ces audits cargo-audit ne constituent pas un contrôle
des versions retirées du registre. Pas de nouvelle couverture, test longue durée,
fuzzing ou qualification Windows/macOS revendiqués.

## Installation et ressources

Installation finale : `~/.local/share/systempulse/0.1.0-local-20261123-security/`.
L'ancien binaire est conservé. Installation effectuée avec `--autostart keep` :
l'entrée existante reste active et lance le même wrapper en mode `--widget`,
qui pointe désormais vers la version corrigée. Préférences, caches et historique
en mémoire ne sont pas confondus : les fichiers sont conservés, le graphique de
session a été perdu lors de la fermeture normale autorisée.

Empreintes installées :
- desktop : `7313379e945c6dcbad1d7a4a6d2a6d09b3bbf0b30c482b29e6fbaf2b95137935` ;
- MCP : `4e1ade95d6de6f5ebc58cfec800850a0f0efe8ff704b1ca47b61864020f60815`.

La recompilation GTK a dépassé l'objectif initial de 1 Gio puis la prévision de
2 Gio ; ces dépassements ont été signalés. Kiunda a autorisé une marge finale de
3 Gio. Derniers relevés : cache desktop 6200 Mio, cache headless 1970 Mio,
installations 1630 Mio, sources tierces environ 2 Mio. Comparaison aux derniers
relevés avant ce lot (3916/1660/1378 Mio) : supplément d'environ **2848 Mio**, hors
petits artefacts. Aucun cache ou ancienne installation supprimé. Pic RAM global
non mesuré ; deux jobs CPU, opérations bornées en temps, pas de build release lourd.

## Reproduction

```sh
python3 -m unittest discover -s tests -p 'test*.py' -v
python3 scripts/verify-glib-backport.py --archive /chemin/vers/glib-0.18.5.crate
sh scripts/check.sh
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --features custom-protocol --locked
cargo audit --file Cargo.lock
cargo audit --file apps/desktop/src-tauri/Cargo.lock
```

L'archive est celle de crates.io ou du cache Cargo, jamais un téléchargement de
fork non vérifié. Une installation publique/release doit aussi revoir les deux
avis de maintenance, les plateformes cibles et la politique de licence du projet.
