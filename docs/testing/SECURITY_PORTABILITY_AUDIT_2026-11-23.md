# Audit local de sécurité et de portabilité — 23 novembre 2026

## Conclusion et périmètre

SystemPulse possède une architecture adaptée à un produit multiplateforme : sept
crates headless séparés du workspace Tauri, modèle de données explicite et
adaptateur `sysinfo`. Les tests locaux passent. Cela ne valide ni Windows/macOS,
ni les installateurs, ni l'absence de vulnérabilités connues des dépendances.
La V1 n'est pas déclarée livrée et cet audit n'est pas un pentest exhaustif.

Revue des sources CLI, plateformes, collector, historique, MCP, frontend, IPC,
CSP, capacités et CI. Tests locaux non destructifs, sans installation, publication,
scan externe, navigateur ni démarrage de fenêtre native. Aucun correctif métier
appliqué dans ce lot. Le rapport historique V1 reste inchangé.

Les fichiers du projet sont actuellement non suivis par Git sur `main` : pas
de référence commitée permettant un diff de régression complet. Aucun fichier
existant supprimé ni ajouté à l'index.

## Environnement et preuves exécutées

- Cible Rust installée : `x86_64-unknown-linux-gnu` seulement.
- rustc 1.99.1 ; Cargo 1.99.1 ; Node 24.22.2 ; npm 11.19.1.
- Cargo hors PATH initial ; utilisation explicite de `$HOME/.cargo/bin`.
- Cargo hors ligne, deux jobs maximum, limite de 120 secondes pour le script
  global ; vérifications incrémentales, pas de compilation de distribution.

| Contrôle | Résultat réel |
| --- | --- |
| `PATH="$HOME/.cargo/bin:$PATH" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 timeout 120 sh scripts/check.sh` | PASS : fmt, Clippy sans avertissement, 29 tests Rust, check ; fmt/Clippy/check Tauri inclus |
| `npm run check && npm test && npm run build` dans `apps/desktop` | PASS : TypeScript, 5 tests, build Vite (14 modules) |
| `PATH="$HOME/.cargo/bin:$PATH" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 timeout 60 cargo build --workspace --locked` | PASS |
| `timeout 45 python3 scripts/test-mcp.py` | PASS, environ 3,34 s : handshake, 10 outils, refus privés/paramètres, 8 appels en vol, ligne 70 Kio, EOF et redémarrage |
| Sondes Node sur fixtures synthétiques | Défauts de validation reproduits, voir A1 |
| `git diff --check` | Aucune sortie ; portée limitée puisque les fichiers sont non suivis |

Les tests Rust comprennent les refus de paramètres CLI/MCP, quotas, fraîcheur,
permissions et symlinks Unix, échec de base pleine et refus de schéma futur.
Le test de 8 appels en vol n'est pas un test de charge soutenue.

## Constats et améliorations priorisées

### A1 — P2 : validation frontend incomplète (reproduit)

Sources : `apps/desktop/src/model.ts:14-38`, `src/view.ts:28-31`,
`src/backend.ts:14-18`.

Sondes réellement exécutées avec `fixture(1)` :

- Remplacer `memory` par `{}` : `validateSnapshot` accepte, car
  `Object.values({}).every(isMetric)` est vrai.
- Fixer `cpu.usage.age_ms = -1` : `available` retourne une valeur affichable.
- Fixer `processes = [null]` : le snapshot est accepté ; le filtre processus
  élimine ensuite cette entrée, donc aucun crash démontré pour ce cas.
- Fixer `storage = [null]` : le snapshot est accepté ; l'accès
  `disk.name ?? disk.id` utilisé dans la vue produit un `TypeError`.

La garde de diagnostics vérifie principalement l'état et le tableau `alerts`,
pas l'ensemble des champs ensuite parcourus (`missing_evidence`, etc.).
Ce constat relève de la robustesse d'une frontière de confiance : le backend
actuel ne produit pas nécessairement ces charges. Aucune XSS ou exécution
arbitraire démontrée.

Proposition : vérifier chaque champ obligatoire, âges finis et non négatifs,
horodatages/compteurs entiers bornés, formes et tailles des collections et
diagnostics. Ajouter les quatre cas synthétiques et les erreurs de diagnostics
à la suite de non-régression. Conserver `textContent`, sans insertion HTML.

### A2 — P2 : diagnostics périmés encore accessibles (revue statique)

Sources : `apps/desktop/src-tauri/src/main.rs:18-35,61-71`,
`apps/desktop/src/main.ts:59-65`.

`get_snapshot` augmente l'âge des métriques, mais `get_health` retourne le
dernier rapport sans garde de fraîcheur. Le worker conserve son cache après
échec et le frontend ne remet pas `report` à `null` dans le `catch`.
Un ancien diagnostic peut donc rester affiché même si un bandeau d'erreur
est présent. Le MCP possède déjà une garde de fraîcheur plus stricte.

Proposition : enveloppe commune de diagnostics avec observation, âge et état
du worker ; rendre Unknown/indisponible après expiration ; invalider le rapport
frontend en erreur. Tester panne, récupération et suspension/reprise avec horloge
injectable. Un échec ancien ne doit pas rester un diagnostic courant.

### A3 — P2 : changements de profil peuvent accélérer les collectes (revue statique)

Source : `apps/desktop/src-tauri/src/main.rs:61-83`.

Le canal de taille 1 borne les messages en attente, pas le débit cumulé.
Chaque réception de profil recrée le collector et ramène la boucle vers une
collecte, même pour le profil déjà actif. Des appels IPC répétés peuvent donc
réinitialiser le warmup et contourner l'intention d'une cadence de 2/3/5 secondes.
Pas de saturation réelle provoquée dans cet audit.

Proposition : ignorer les profils identiques, regrouper les changements et
préserver un délai monotone minimal de collecte, indépendamment des commandes
IPC. Tester avec un backend compteur et une horloge contrôlée.

### A4 — P2 : mémoire de collecte non bornée par la limite de sortie (revue statique)

Source : `crates/platforms/src/lib.rs:250-287`.

La collecte parcourt tous les processus, alloue leurs modèles et noms, trie
la collection entière, puis tronque à 100. Le coût est O(P log P), avec un
vecteur O(P), en plus de l'inventaire sysinfo. La limite des résultats n'est
donc pas une limite du travail d'acquisition.

Proposition : sélectionner un top-K borné avant construction des chaînes de
sortie ; examiner séparément le coût de sysinfo. Mesurer CPU, RSS et durée
médiane/p95 sur inventaires synthétiques avant de promettre un gain.

### A5 — P2 : chaîne d'approvisionnement non auditée et CI partielle

Sources : `.github/workflows/ci.yml`, `scripts/check.sh`, `rust-toolchain.toml`.

- `cargo-audit` et `cargo-deny` indisponibles ; aucun verdict CVE/licences.
- Pas d'audit npm exécuté : aucun arbre de dépendances transmis implicitement
  à un service externe.
- `stable` et les images `*-latest` évoluent ; les actions utilisent des tags,
  pas des SHA. La reproductibilité des outils n'est donc pas entièrement figée.
- Le workspace Tauri est vérifié sans `--locked` dans le script et la CI.
- CI desktop sur Ubuntu seulement ; pas de test protocole réel ni navigateur
  dans le workflow actuel.
- `scripts/test-mcp.py` cherche un binaire sans suffixe `.exe` : rendre la
  résolution dépendante de la plateforme avant son intégration à Windows.
- `tests/browser.mjs` utilise des chemins absolus machine par défaut et un
  serveur Vite préexistant ; il n'est pas autonome sur un poste neuf.

Proposition : audit des deux Cargo.lock (pas seulement le workspace racine),
audit npm autorisé, politique de licences décidée, actions épinglées, commandes
Tauri `--locked`, contrôle central frontend+MCP et matrice desktop native.
Conserver les versions d'outils documentées avec mises à jour volontaires.

### A6 — P2 : durcissement des chemins de l'historique avant intégration

Source : `crates/history/src/lib.rs:140-174,364-426`.

Les permissions 0700/0600, liens, hardlinks et NOFOLLOW sont déjà contrôlés.
Il n'y a pas de contrôle explicite du propriétaire Unix et l'inspection des
composants du chemin n'est pas une ouverture atomique ancrée sur un descripteur.
Ne pas présenter cette approche comme une défense absolue contre les courses
de renommage ; aucune exploitation démontrée. Le modèle de menace exclut déjà
une isolation forte contre le même UID.

Proposition : préciser parents de confiance, propriétaire attendu et contrat
de chemins par OS ; tester le comportement en dossiers temporaires et privés.
L'historique n'étant pas intégré, éviter toute migration implicite.

## Portabilité : état distinct par composant

| Composant | Ubuntu / Linux testé ici | Windows | macOS |
| --- | --- | --- | --- |
| Core, collector, alertes, CLI | Build et tests PASS | Architecture portable, exécution non validée | Architecture portable, exécution non validée |
| MCP stdio | Protocole réel PASS | Serveur à qualifier ; script de test à adapter | Serveur à qualifier |
| Frontend | Types, tests, build PASS | Build/exécution non testés | Build/exécution non testés |
| Tauri | fmt/Clippy/check PASS, pas de lancement dans cet audit | Aucune preuve native | Aucune preuve native |
| Historique SQLite | Tests Unix et mémoire PASS, non intégré | Persistance explicitement refusée faute de contrôle ACL natif | Branche Unix, non exécutée ; chemins avec composants symlink à tester |
| GPU et batterie supplémentaires | Lectures sysfs Linux spécifiques | Adaptateur équivalent manquant | Adaptateur équivalent manquant |
| Installateurs et signature | Non validés | Non validés | Non validés |

`bundle.active` est `false` dans `tauri.conf.json`. La présence d'icônes Windows
et macOS ne prouve pas qu'un paquet est produit ou installable. La CI trois OS
headless est une configuration, pas une preuve d'exécution dans ce lot.

Plan recommandé, sans changer l'architecture :

1. Définir OS minimums et architectures supportées ; choisir explicitement
   Ubuntu LTS et/ou la version de développement, Windows x64/ARM64 et macOS
   Intel/Apple Silicon selon les postes cibles.
2. Publier une matrice de capacités : CPU/RAM, réseau, stockage, processus,
   GPU, batterie, températures ; conserver Unsupported/PermissionDenied et
   ne jamais remplacer un capteur absent par zéro.
3. Qualifier les trois workspaces/interfaces sur vrais runners puis vrais
   postes, sans privilèges : collecte, IPC/CSP, suspension, permissions,
   redémarrage, EOF, chemins Unicode/espaces et comportement hors ligne.
4. Implémenter le contrat ACL Windows et les chemins de données natifs ; tester
   macOS avec chemins résolus, sans retirer globalement le refus de symlinks.
5. Ajouter packaging natif et essais installation/désinstallation sur données
   fictives. Signature Windows, signature/notarisation macOS et toute publication
   nécessitent une autorisation et, si applicable, un budget séparés.

## Points positifs conservés

- Aucun couplage Tauri imposé au socle headless ; `unsafe_code = forbid` pour
  le code applicatif concerné (ne constitue pas une garantie sur les dépendances).
- MCP stdio, dix outils fixes, partage processus opt-in, schémas stricts,
  entrées 64 Kio, arguments 4 Kio, résultats et historique bornés, quota d'appels.
- Collecte indépendante des demandes MCP et refus des snapshots périmés.
- Pas d'appel shell identifié dans les sources runtime Rust examinées ; les
  `Command` repérés sont dans les tests d'entrypoint.
- Rendu textuel, CSP restrictive et capacités de plugins vides ; ne pas assimiler
  cela à une interdiction de toutes les commandes IPC applicatives.
- SQL paramétré, transactions, `synchronous=FULL`, limites et erreurs propagées.

## Limites et suite requise pour un audit approfondi

Non exécutés : audit CVE Rust/npm, scan de secrets dédié, couverture, fuzzing,
sanitizers, charge soutenue, longue durée, captures réseau, test dynamique XSS/IPC
dans la webview native, analyse des binaires de distribution et tests Windows/macOS.
Une tentative de scan par motifs avec `rg --count` n'a pas abouti : binaire absent ;
aucune conclusion sur l'absence de secrets. Aucun fichier d'authentification lu.

Prochaine étape proposée : corriger A1–A3 avec tests de non-régression, puis
autoriser une installation locale isolée des auditeurs et les consultations de
bases de vulnérabilités. Pour npm, préciser l'accord de transmission des métadonnées
de dépendances. Définir séparément durée, RAM, CPU et disque pour fuzzing/charge,
ainsi que les machines Windows/macOS disponibles. Aucun travail lourd lancé ici.

Sources de ce rapport : fichiers et exécutions locaux cités. Aucune recherche
d'avis de sécurité à jour n'a été effectuée ; aucune affirmation « sans CVE »,
« sécurisé à 100 % » ou « prêt production » n'est justifiée.
