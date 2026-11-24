# Capsule de surveillance — étude et contrat du lot

> Ce document conserve le contrat du premier lot. Le lot suivant ajoute les
> signaux de variation, le glisser sur toute la surface et une présélection visible
> Oui pour le démarrage capsule seule : [comportement actuel et tests](../testing/MEASUREMENT_SIGNALS_2026-11-23.md).

## Périmètre confirmé le 23 novembre 2026

Capsule ovale, priorités RAM et température CPU ; ouverture d'un panneau au clic,
poignée de déplacement distincte, Échap pour replier, toujours au-dessus et
masquable. Deux fenêtres fixes réutilisent **un seul collecteur desktop**. Le
serveur MCP reste un programme séparé, non lancé par la capsule. Aucun score
artificiel mélangeant CPU/RAM/GPU ni seuil thermique universel n'est introduit.

Budget autorisé : 30 minutes de compilations/tests locaux, deux jobs Cargo,
objectifs 3 Gio RAM et 2 Gio disque supplémentaires, pas de nouvelle dépendance.
La mise à jour locale et le redémarrage ont été autorisés ; pour ce PC, Kiunda a
choisi explicitement **de ne pas activer le démarrage automatique**.

## Étude des couleurs avant intégration

Sources officielles lues :
- [W3C WCAG 2.2 — contraste minimal](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html),
  page mise à jour le 4 octobre 2026, consultation le 23 novembre 2026.
- [W3C — usage de la couleur](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html),
  mise à jour le 16 septembre 2025, même date de consultation.

But : au moins 4,5:1 pour tous les textes, 3:1 pour les contours de contrôles,
indépendamment du fond d'écran. Les surfaces sont **opaques** ; seule la zone
autour de la capsule est transparente. Pas de verre flouté dont le contraste
dépendrait du bureau. Pas de clignotement, animation courte neutralisée par
`prefers-reduced-motion`, contours adaptés à `forced-colors`.

| Rôle | Sombre | Clair | Justification et signal complémentaire |
| --- | --- | --- | --- |
| Surface | `#111c2b` | `#f5f8fc` | Fond neutre opaque ; absence de surcharge de saturation |
| Carte | `#19283a` | `#e8eff6` | Séparation spatiale des métriques sans dépendre de leur teinte |
| Texte | `#edf4ff` | `#14283d` | Lecture des valeurs prioritaires, chiffres tabulaires |
| Texte secondaire | `#b1c0d2` | `#475d73` | Disponible mais subordonné aux métriques ; ne pas le rendre illisible |
| RAM/action | `#73d9ca` | `#0b655f` | Cyan/teal cohérent avec SystemPulse, toujours accompagné du label RAM et du pourcentage |
| GPU | `#a9c7ff` | `#335fa8` | Bleu distinguant une catégorie, accompagné de GPU et d'une valeur/absence |
| Charge élevée | `#ffcf80` | `#865000` | Ambre d'attention, accompagné de « RAM élevée » |
| Charge très élevée | `#ff9cab` | `#a7193c` | Rose-rouge lisible sans rouge sombre sur noir ; « RAM très élevée », pas « panne » |
| Contour | `#71849b` | `#718198` | Délimitation des contrôles et focus, mesurée sur les deux surfaces |

Ratios calculés par la formule de luminance sRGB du W3C **avant intégration** :

| Texte / surface | Sombre : minimum sur fond/carte | Clair : minimum sur fond/carte |
| --- | --- | --- |
| Principal | 13,50:1 | 12,93:1 |
| Secondaire | 8,07:1 | 5,88:1 |
| RAM/action | 8,90:1 | 5,95:1 |
| GPU | 8,75:1 | 5,42:1 |
| Attention | 10,31:1 | 5,73:1 |
| Charge très élevée | 7,54:1 | 6,36:1 |

Les tests comparent les valeurs non arrondies aux seuils. Ces calculs et les
aperçus ne constituent pas une certification WCAG complète, ni une étude avec
participants, ni une qualification de tous les types de déficience visuelle.

## Ergonomie et données

- Capsule logique 320 × 112 (surface ovale intérieure 300 × 88 environ) ; panneau
  400 × 620, dimensions bornées par la zone de travail de l'écran et son DPI.
  L'ancre bas-droite est conservée pendant l'ouverture vers la gauche/haut.
- RAM : pourcentage utilisé, jamais zéro si inconnu. Indicateur de charge
  « élevée » à 85 %, « très élevée » à 95 % : conventions visuelles de ressources,
  **pas une modification des seuils d'alertes du moteur Rust**.
- Température CPU : valeur en °C ou Non mesuré. Un seuil matériel non qualifié
  n'autorise pas à annoncer une surchauffe. GPU : maximum des capteurs valides
  disponibles, pas une attribution GPU par processus.
- Panneau : CPU/RAM/GPU, thermique, consommateurs RAM parmi les processus
  observés, noms affichés via `textContent`, aucune commande kill/optimisation.
- Le backend limite actuellement sa sortie processus à 100 ; ce classement ne
  prétend pas couvrir chaque processus du système. L'attribution mémoire aux
  modèles locaux et la mémoire GPU par processus restent à qualifier.
- Actualisation UI environ 3 s, backoff jusqu'à 30 s en erreur ; collecteur selon
  profil 2/3/5 s, capteurs lents selon leur propre cadence. Ce n'est pas du temps
  réel dur. Pas de collecte déclenchée par clic ou par animation.
- Le panneau peut défiler, les boutons ne sont pas reconstruits à chaque
  acquisition ; le focus et les choix MCP de session restent stables.
- Wayland peut ignorer le placement global bas-droite et la superposition. Ne
  pas contourner les restrictions du compositeur ni promettre un placement absolu
  partout. La poignée utilise le déplacement natif ; pas de mémorisation de
  coordonnées arbitraires hors écran.

## Frontières de confiance

- Deux fenêtres locales fixes, aucune permission générique de shell, fichiers,
  HTTP ou gestion de fenêtres côté JavaScript. CSP conservée. Chaque commande
  reçoit la vraie fenêtre appelante et vérifie son label côté Rust.
- Actions widget strictes : agrandir/replier, déplacement natif, dashboard,
  masquage et quitter. Pas de chemin, URL, commande ou label cible arbitraire.
- Un verrou d'instance garde une seule acquisition desktop par installation de
  données. Un second lancement doit être refusé avant de démarrer son worker ;
  revenir au tableau de bord via la capsule. L'isolation contre un même UID
  malveillant n'est pas une garantie.
- Masquer le widget réaffiche le tableau de bord pour conserver un moyen de
  contrôle. Fermer le dashboard garde la capsule ; Quitter arrête l'application.

## Switch Claude/Codex : périmètre honnête

Le switch **Préparer la connexion MCP** affiche seulement une configuration à
copier, après confirmation. État « prêt à configurer », jamais « connecté » sans
preuve. Aucun fichier privé d'agent lu ou modifié, aucun client lancé et aucune
métrique envoyée. Le binaire MCP installé reste local stdio et lecture seule,
sans `--share-processes`. Couper le switch masque la préparation : cela ne révoque
pas une configuration copiée ; l'interface le dit explicitement.

Sources consultées : [Codex MCP](https://developers.openai.com/codex/mcp/) et
[Claude Code MCP](https://code.claude.com/docs/en/mcp), le 23 novembre 2026.
La configuration du client peut ensuite envoyer les métriques à son fournisseur
IA : consentement et validation réels dans ce client requis, non effectués ici.

## Installation et démarrage à la connexion

`scripts/install-user.py` : installation Linux utilisateur versionnée, anciennes
versions et lanceurs conservés, refus d'écraser un fichier non reconnu ou une
version existante, chemins et permissions contrôlés, écritures de lanceurs
atomiques. Aucun sudo ni exécution des binaires par l'installateur.

Par défaut il demande l'accord d'installation puis : « Lancer la capsule à chaque
connexion au bureau ? [o/N] ». Une réponse vide est non. Le mode non interactif
exige des options explicites. `--autostart enable` crée une entrée gérée fixe ;
`disable` n'en crée pas si absente et marque `Hidden=true` si déjà gérée ; pas de
suppression implicite. Un fichier autostart non reconnu est refusé.

Source : [freedesktop Autostart Specification](https://specifications.freedesktop.org/autostart-spec/latest/),
version 0.5, consultée le 23 novembre 2026. Il s'agit de la connexion de session
graphique, pas du boot avant authentification. Windows/macOS exigent une
implémentation native distincte et ne sont pas qualifiés par cet installateur.
