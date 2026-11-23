# Modèle de menace V1

Actifs : métriques locales, noms de processus, chemins de volumes, historique,
préférences, intégrité de l'application et disponibilité du poste.
Acteurs : utilisateur local, processus du même utilisateur, client MCP/agent,
contenu système malveillant, frontend compromis, dépendance compromise.

| Frontière / scénario | Contrôle attendu | Risque résiduel |
| --- | --- | --- |
| OS → collector : valeurs incohérentes, capteur absent | Validation finitude/unités, états explicites, warmup | Drivers et OS hors contrôle |
| Nom de processus → UI/IA : XSS/prompt injection | textContent, données structurées, pas arguments, opt-in MCP | Un agent peut mal interpréter toute donnée |
| Client MCP → service : lecture arbitraire/exécution | Dix outils fixes lecture seule, schémas stricts | Un processus local peut lancer son propre collecteur |
| MCP : saturation par payload/appels | Taille lignes, réponses, résultats, intervalle collecte, timeout | Pas de garantie contre un utilisateur administrateur |
| Webview → Rust : abus IPC | Commandes étroites, paramètres validés, CSP restrictive | Bug WebKit/Tauri et dépendances |
| Historique → fichiers : symlink/race/corruption | Dossier privé, nom fixe, refus symlink, migrations, transactions | Attaquant du même UID hors frontière forte |
| Historique : disque plein | Bornes stockage, retour d'erreur, pas succès fictif | Filesystem peut tomber en panne |
| Logs / export : fuite privée | Aucun argument/env/contenu ; export explicite | Noms/chemins conservés par erreur à auditer |
| Supply chain | Lockfiles, audit/deny/licences, pas JS distant | Aucun audit n'offre sécurité absolue |
| Privilèges | Aucun root, shell, kill, SMART privilégié | Accès OS ordinaires du compte |

Pas de port réseau ouvert ni télémétrie Internet. Pas d'authentification cloud.
Le client stdio possède déjà la capacité de lancer le binaire ; stdio n'isole
pas deux processus du même utilisateur. Ne pas promettre de prévention absolue
de prompt injection ou de protection contre root/un même UID malveillant.

Tests : entrées inconnues/extrêmes, chaînes adversariales, mode minimal,
valeurs non finies, données anciennes, hystérésis, permissions/symlinks,
corruption, sorties protocole et charge bornée. État réel dans V1_TEST_REPORT.md.
