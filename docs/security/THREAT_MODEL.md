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

## Capsule desktop — ajout du 23 novembre 2026

Deux fenêtres locales fixes partagent le même worker et un verrou d'instance.
Les commandes vérifient le label réel de la fenêtre appelante ; aucun argument
de commande shell, URL, chemin cible ou fenêtre arbitraire n'est exposé au widget.
La fenêtre compacte n'est pas un espace de confiance séparé du même UID.

Le switch MCP ne crée pas une connexion : il prépare un exemple à copier après
confirmation. Aucun fichier d'agent lu/modifié, aucun client lancé. La configuration
externe peut permettre ensuite l'envoi de métriques au fournisseur du client ; la
révocation s'effectue aussi dans ce client. Ne pas présenter le switch comme un
coupe-circuit global d'un MCP stdio indépendant.

L'installateur utilisateur exige une décision autostart, refuse les cibles non
reconnues/symboliques et conserve les anciennes versions. Écritures atomiques par
fichier et erreurs propagées ; pas de garantie de transaction globale entre tous
les fichiers lors d'une coupure. Résultats, avis de dépendances ouverts et limites :
[rapport du widget](../testing/WIDGET_TEST_REPORT_2026-11-23.md).
