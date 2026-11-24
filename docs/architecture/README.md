# Architecture

Sept crates Cargo réels, édition 2024, resolver 3, sans cycle de dépendances.
Le socle headless (CLI, MCP) ne dépend jamais de Tauri.

| Composant | Responsabilité | Dépend de |
| --- | --- | --- |
| `systempulse-core` | Types, modèles, unités, disponibilité, fraîcheur | `serde`, `schemars` |
| `systempulse-platforms` | Adaptateurs OS et accès matériel | `core`, `sysinfo` |
| `systempulse-collector` | Orchestration bornée, profils, cache, backoff | `core`, `platforms` |
| `systempulse-alerts` | Règles déterministes, hystérésis, diagnostics | `core` |
| `systempulse-history` | Historique SQLite local, agrégats seulement | `core`, `rusqlite` |
| `systempulse-mcp-server` | Serveur MCP local stdio, 10 outils lecture seule | `core`, `collector`, `alerts`, `rmcp`, `tokio` |
| `systempulse-cli` | Présentation terminal, sortie JSONL | `core`, `collector` |
| `apps/desktop` | Dashboard et capsule Tauri 2 / TypeScript (hors workspace), collecteur partagé | `core`, `collector`, `alerts`, `tauri` |

## Règles de dépendance

- Jamais de dépendance `core` → présentation, ni de cycle.
- La CLI et le MCP fonctionnent sans GUI ni dépendance Tauri.
- Aucune représentation ne décide des droits d'accès matériel.
- `apps/desktop/src-tauri` est un workspace Cargo séparé, pour que les
  commandes headless n'imposent pas WebKit/GTK.

## Sections spéciales

- `systempulse-core` ne doit pas masquer le crate standard `core` ; les noms
  Cargo restent préfixés `systempulse-`.
- `systempulse-history` n'est actuellement importé par aucun autre composant :
  c'est un composant testé mais non intégré.

## Frontières de confiance

| Frontière | Contrôle |
| --- | --- |
| OS → collector | validation finitude/unités, états explicites, warmup |
| Collector → UI/IA | données structurées, noms seuls, jamais d'arguments |
| Client MCP → service | 10 outils fixes, schémas stricts, lecture seule |
| Webview → Rust | Commandes IPC étroites, fenêtres main/widget vérifiées côté Rust, CSP restrictive ; aucune permission générique shell/fichiers |

Détail des menaces : [modèle de menace](../security/THREAT_MODEL.md).

Capsule, couleurs mesurées, cycle de vie et installation opt-in :
[étude de conception](WIDGET_DESIGN.md).

## Décisions

Voir [DECISIONS.md](DECISIONS.md) : Tauri isolé, TypeScript natif + Vite + uPlot,
sysinfo puis adaptateurs ciblés, SQLite local facultatif, SDK MCP officiel.

## État de validation

L'implémentation et les preuves sont suivies dans
[V1_TEST_REPORT.md](../testing/V1_TEST_REPORT.md). Un composant écrit n'est pas
un composant validé : se référer au rapport de tests.
