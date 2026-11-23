# Décisions techniques V1

Sources consultées le 4 novembre 2026. Les versions exactes résolues et dépendances
transitives sont dans les lockfiles ; l'audit de sécurité effectif est distinct
d'une consultation documentaire.

## ADR 01 — Tauri séparé du workspace headless

Décision : conserver les six crates métier et isoler le projet Tauri dans
apps/desktop/src-tauri. Alternative : membre obligatoire du workspace, rejetée
car les commandes CLI/MCP imposeraient WebKit/GTK aux environnements headless.
Tauri 2 est imposé par le besoin ; Electron ajouterait un navigateur embarqué
et n'apporte pas de besoin démontré. Pas de plugin shell ni filesystem générique.
[Tauri security](https://v2.tauri.app/security/).

## ADR 02 — TypeScript natif, Vite, uPlot

TypeScript DOM + CSS limite les dépendances runtime ; comparé à Svelte, il
nécessite de gérer explicitement cycle de vie, état et nettoyage, mais les six
vues ne justifient pas encore de framework. Vite construit les modules locaux.
uPlot, spécialisé séries temporelles, est retenu face à Chart.js plus généraliste.
Les graphes auront aussi résumé textuel accessible et nombre de points borné.
Licences attendues MIT : vérification des manifests/lockfiles au lot audit.
Sources : [Vite](https://vite.dev/guide/), [Svelte](https://svelte.dev/docs/svelte/overview),
[uPlot](https://github.com/leeoniya/uPlot), [Chart.js](https://www.chartjs.org/docs/latest/).

## ADR 03 — sysinfo puis adaptateurs ciblés

sysinfo 0.40.1 (MSRV 1.95 ; Rust local 1.99.1) fournit le socle multiplateforme.
Comparé à une réécriture procfs/Win32/sysctl, il évite plusieurs parsers OS et
permet de réutiliser l'instance de collecte. Rafraîchir avant lecture et mesurer
le CPU par différence avec warmup. Les API natives ne complètent que les trous
prouvés ; GPU, SMART et capteurs manquants restent explicitement indisponibles.
Aucun sous-processus par échantillon ni privilège supplémentaire.
[API officielle](https://docs.rs/sysinfo/0.40.1/sysinfo/).

## ADR 04 — SQLite local facultatif

SQLite via rusqlite 0.41.1 avec version embarquée, comparé à JSONL : transactions,
requêtes bornées et migration versionnée simplifient cohérence et reprise ; JSONL
réduit le code SQL mais impose index, récupération d'écritures partielles et
rotation atomique. Historique agrégé uniquement, aucun argument de processus
ni contenu de fichier. Permissions restrictives, rétention bornée et erreurs
remontées. Pas de base cloud. Le chiffrement disque demeure responsabilité OS.
[SQLite](https://www.sqlite.org/whentouse.html),
[rusqlite](https://docs.rs/rusqlite/0.41.1/rusqlite/).

## ADR 05 — SDK officiel MCP, stdio et partage minimal

rmcp 3.8.0, comparé à un parseur JSON-RPC artisanal : le SDK gère compatibilité,
transport et messages standards, réduit les écarts de protocole. Aucun HTTP.
La spécification latest consultée est 2026-07-28 et conserve des règles de
compatibilité avec les anciennes sessions initialize. La version réellement
négociée sera consignée par les tests ; ne pas l'inférer de latest.
Tout nom de processus est non fiable ; partage désactivé sauf option explicite.
Le SDK ne constitue pas un contrôle d'autorisation : validation, quotas et
bornes de réponse sont à appliquer dans SystemPulse.
[SDK](https://github.com/modelcontextprotocol/rust-sdk),
[compatibilité MCP](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning).
