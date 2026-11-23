# Qualification multiplateforme

## Matrice cible autorisée le 23 novembre 2026

| Cible produit | Runner préparé | Qualification restante |
| --- | --- | --- |
| Ubuntu 24.04 LTS x86_64 | `ubuntu-24.04` | Exécution sur cette version ; la machine de développement est distincte |
| Windows 11 x86_64 | `windows-2025` (Windows Server, pas Windows 11) | Poste Windows 11 réel, WebView2, permissions et installation |
| macOS Apple Silicon | `macos-15` arm64 | Exécution native, WebKit, chemins et permissions ; version minimale produit à confirmer |

Source des labels : [documentation GitHub des runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners),
consultée le 23 novembre 2026. L'architecture réelle doit être relevée dans chaque
preuve d'exécution. Aucun runner distant n'a été lancé par ce lot.

Le workflow CI est volontairement **manuel** (`workflow_dispatch`) jusqu'à
validation de la visibilité du dépôt et du budget Actions. Un dépôt privé peut
consommer des minutes facturables. Les timeouts CI ne constituent pas le budget
local d'audit de 15 minutes. Pas d'installation privilégiée sur la machine de
développement ; les prérequis Linux `sudo apt-get` du workflow concernent seulement
une VM éphémère, lors d'une future exécution explicitement autorisée.

## Capacités et invariants

- Socle headless indépendant de Tauri ; pas de nouvelle dépendance pour ce lot.
- CPU/RAM/réseau/disques/processus : backend sysinfo commun, à qualifier par OS.
- GPU et batterie supplémentaires : sysfs Linux seulement ; ne pas promettre
  de parité Windows/macOS et ne pas transformer Unsupported en zéro.
- Historique : SQLite isolé non intégré ; persistance Windows refusée en sécurité
  tant que le contrat ACL natif n'existe pas. Pas de suppression de ce garde-fou.
- macOS : tester les chemins natifs et les composants symlink sans retirer le
  contrôle global des liens ; aucune validation supposée à partir de `cfg(unix)`.
- Script MCP : suffixe `.exe` pris en charge ; aucune collecte de noms de processus
  pendant ses tests sans option explicite.

## Protocole de qualification sur chaque poste

1. Relever OS, architecture, rustc/Cargo, Node/npm et WebView ; conserver lockfiles.
2. Exécuter fmt/Clippy/tests/build headless et `python scripts/test-mcp.py`.
3. Dans `apps/desktop` : `npm ci`, `npm run check`, `npm test`, `npm run build`.
4. Dans `src-tauri` : `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings`.
5. Avec accord de lancement GUI : vérifier navigation, IPC, mode hors ligne,
   capteurs absents, échec/reprise, suspension, diagnostics expirés, Unicode et
   espaces dans les chemins, arrêt propre et compte sans privilèges.
6. Packaging/signature/installateur : lot distinct. `bundle.active` reste false.
   Utiliser seulement des données fictives pour les tests de désinstallation.

Le script navigateur existant reste dépendant d'un Playwright/Chrome local fourni
par variables `SYSTEMPULSE_PLAYWRIGHT` et `SYSTEMPULSE_CHROME` et d'un serveur
Vite local préexistant. Il n'est pas une qualification de WebView native.
