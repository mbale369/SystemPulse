# Suivi des corrections, audits et préparation Git

## Autorisation et budget

Le 23 novembre 2026, Kiunda a autorisé : corrections locales, installations
isolées d'auditeurs officiels, bases publiques, `npm audit` vers le registre
vérifié, matrice standard et commit local. Pas de push, création de dépôt distant,
publication, mise à jour globale de dépendances ni exécution de workflow distant.

Budget choisi : 15 minutes d'audit, deux jobs CPU, objectifs 2 Gio RAM / 1 Gio de
disque supplémentaire. Contrôles incrémentaux limités individuellement par
`timeout`, Cargo à deux jobs et Gitleaks à deux threads ; pas de fuzzing natif
continu, benchmark matériel, serveur GUI, charge soutenue ou compilation complète.
Les limites RAM/disque sont des objectifs, pas des plafonds système mesurés pour
toute la session. Les caches préexistants ne sont pas inclus dans le supplément ;
aucun cache nettoyé. Une exécution `time -v` des tests a été tentée, sans rapport
mémoire complet exploitable dans la sortie : aucun pic global revendiqué.

## Correctifs appliqués

- **A1** : champs mémoire obligatoires vérifiés explicitement, âges/compteurs
  entiers sûrs non négatifs, qualité/unités vérifiées, collections bornées et
  entrées `null` refusées, processus et champs de stockage affichés contrôlés.
  Garde complète des champs de diagnostics utilisés par l'interface.
- **A2** : `get_health` refuse un cache vieux de plus de dix secondes ou marqué
  en erreur ; le worker marque l'échec et la récupération. Le frontend efface
  aussi le rapport en erreur. Contrat IPC conservé (`Result<HealthReport, String>`)
  plutôt qu'une rupture d'interface par nouvelle enveloppe.
- **A3** : échéance monotone indépendante des messages IPC ; un changement de
  profil ne déclenche pas une acquisition anticipée. Collector réutilisé :
  warmup, compteur et cache préservés, pas de réinitialisation sysinfo répétée.
- Scripts/CI : Tauri `--locked`, suffixe MCP `.exe`, actions épinglées par SHA,
  matrice headless et desktop explicite, workflow manuel avant budget distant.
- Git : exclusions supplémentaires des bases locales, exports et certificats ;
  icônes ICNS marquées binaires. Aucun fichier source supprimé.

## Résultats réellement obtenus après correction

| Contrôle | Résultat |
| --- | --- |
| `sh scripts/check.sh`, Cargo hors ligne, deux jobs, timeout 120 s | PASS : fmt, Clippy, 31 tests headless et compilation ; fmt/Clippy/check Tauri |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked --bin systempulse-desktop`, hors ligne, timeout 90 s | PASS : 1 test expiration/échec/récupération des diagnostics, aucune fenêtre ouverte |
| `npm run check && npm test && npm run build` | PASS : 9 tests, 14 modules, bundle JS 69,80 kB |
| Corpus de validation synthétique déterministe | 1 000 entrées mémoire invalides refusées ; ce n'est pas un fuzzing guidé par couverture |
| `cargo build --workspace --locked` puis `python3 scripts/test-mcp.py` | PASS : protocole, confidentialité, paramètres, concurrence bornée, entrée 70 Kio, EOF/redémarrage, environ 3,33 s |
| Snapshot CLI réel, profil detailed, validé par `validateSnapshot` Node | PASS ; aucun processus partagé, aucune télémétrie brute imprimée |
| Gitleaks 8.30.1 `dir --redact=100`, timeout 60 s, configuration par défaut étendue | Aucun secret détecté dans environ 404 ko de sources au premier passage ; caches/builds/.git exclus |
| Gitleaks final sur diff indexé puis sources avec configuration projet | Aucun secret détecté : environ 447 ko de diff et 419 ko de sources ; aucun historique antérieur à scanner |
| `npm audit --registry=https://registry.npmjs.org/ --ignore-scripts`, timeout 60 s | **0 vulnérabilité signalée**, dépendances dev incluses |
| cargo-audit 0.23.0, lockfile racine | **ÉCHEC sécurité : 1 vulnérabilité**, 1 avertissement |
| cargo-audit 0.23.0, lockfile Tauri | **ÉCHEC sécurité : 1 vulnérabilité**, 3 avertissements |
| cargo-deny 0.22.0, racine, advisories/sources | Avis rmcp confirmé ; sources OK ; avis non ignoré |
| cargo-deny Tauri hors ligne | **BLOQUÉ**, crates de plateformes non présents (`winapi-util`) ; aucun verdict sources/licences Tauri |
| cargo-deny list racine | Inventaire des expressions de licences obtenu ; ne constitue pas une approbation juridique |

Tentatives sans résultat utile : `npm run check` lancé une fois à la racine sans
package.json, puis relancé dans le bon dossier ; premier cargo-deny hors ligne
sans son sous-dossier de base propre, corrigé par une exécution réseau autorisée.
Ces essais ne sont pas comptés comme des contrôles réussis.

## Avis de dépendances à traiter avant release

| Avis | Dépendance | Constat et suite |
| --- | --- | --- |
| [RUSTSEC-2026-0462](https://rustsec.org/advisories/RUSTSEC-2026-0462) | rmcp 3.8.0 | Clients Streamable HTTP : corps de réponse non borné. Fonctionnalités HTTP clientes non activées dans notre manifeste stdio ; pas d'exploitation démontrée ici. Passage ciblé à >=3.9.0 à préparer et tester ; audit conservé en échec, aucune exception ajoutée. |
| [RUSTSEC-2026-0592](https://rustsec.org/advisories/RUSTSEC-2026-0592) | url 2.5.8, Tauri | Récursion d'origine des URL blob imbriquées ; correctif >=2.5.9. Atteignabilité dans la WebView non démontrée ; patch ciblé du lockfile et tests à faire, pas de banalisation du risque. |
| RUSTSEC-2026-0543 | chrono 0.4.45 | Non maintenu, présent dans les deux graphes. Évaluer migration en amont et remplacement compatible, sans réécriture implicite. |
| RUSTSEC-2024-0370 | proc-macro-error 1.0.4, Tauri | Non maintenu ; dépendance transitive à traiter via chaîne GUI compatible. |
| RUSTSEC-2024-0429 | glib 0.18.5, Tauri | Unsoundness de VariantStrIter ; version corrigée >=0.20.0 selon l'avis. Ne pas forcer une montée transitive incompatible avec GTK/WebKit. |

Les pages officielles des deux vulnérabilités ont été lues. Base RustSec :
1 561 avis chargés au contrôle. `--no-yanked` a été employé pour cargo-audit :
pas de verdict sur les versions retirées du registre. Les avertissements ne sont
pas des tests d'exploitation. Aucune mise à jour de lockfile ni suppression
d'avertissement effectuée pour obtenir artificiellement un audit vert.

## Outils isolés et reproduction

Installation temporaire conservée dans `/tmp/opencode/systempulse-audit-20261123` ;
aucun binaire global, runtime système ou configuration Codex modifié. Archives
officielles téléchargées via HTTPS, sommes SHA-256 comparées aux métadonnées de
release GitHub **avant exécution** (intégrité, pas une attestation indépendante).

| Outil | Source officielle / version | SHA-256 de l'archive Linux |
| --- | --- | --- |
| cargo-audit | rustsec/rustsec, cargo-audit/v0.23.0 | `b77993532b253fcbf550996d7716465ec3d734d4e74e7f87bfb65a19add0fb59` |
| cargo-deny | EmbarkStudios/cargo-deny, 0.22.0 | `98c56e6a408bfa5c3c60744b6015aaa6aa08f743fdeb88386d06fca035bb1786` |
| Gitleaks | gitleaks/gitleaks, v8.30.1 | `551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb` |

Les dépendances npm ont été auditées contre `https://registry.npmjs.org/` après
vérification ; ni source ni secret envoyé par l'agent. Les auditeurs Rust
consultent les bases/registries publics ; le scan de secrets reste local.
Configurations projet : `.gitleaks.toml`, `deny.toml` ; aucune politique de
licence inventée, celle du projet demeure non choisie. L'inventaire headless
inclut des expressions alternatives (ex. MIT OU LGPL), pas une obligation de
choisir chaque licence listée. Le graphe desktop complet reste à inventorier.

Avec ces outils sur PATH, commandes de reproduction (réseau autorisé requis) :

```sh
cargo audit --no-yanked --file Cargo.lock
cargo audit --no-yanked --file apps/desktop/src-tauri/Cargo.lock
cargo deny --locked check advisories sources
cargo deny --manifest-path apps/desktop/src-tauri/Cargo.toml --locked check advisories sources
gitleaks dir --config .gitleaks.toml --redact=100 --timeout 60 .
```

## Portabilité et suites ouvertes

Voir [PORTABILITY.md](../development/PORTABILITY.md) et
[préparation GitHub](../development/GITHUB_PREPARATION.md).
Les trois cibles sont préparées, **Windows/macOS ne sont toujours pas exécutés**.
Un runner Windows Server ne qualifie pas Windows 11, et cette machine Linux ne
qualifie pas automatiquement Ubuntu 24.04. ACL Windows, compléments matériels,
packaging et signature restent des lots distincts. Pas de release ou certification.

Restent aussi ouverts : A4 (top-K processus), A6 (durcissement chemins/propriétaire),
fuzzing natif, couverture, charge soutenue, comportement après vraie suspension,
test dynamique IPC/WebView et inventaire juridique complet des dépendances.
La prochaine étape sécurité est une mise à jour **ciblée et relue** de rmcp/url,
puis la résolution des transitives Tauri. Le commit local conserve les constats
au lieu de présenter ce travail comme un audit complet réussi.
