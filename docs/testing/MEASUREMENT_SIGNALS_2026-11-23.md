# Variations perceptibles, glisser et lancement — 23 novembre 2026

## Autorisation et résultat

Kiunda a autorisé un lot borné à 20 minutes, deux jobs de compilation, objectifs
2 Gio RAM / 1 Gio disque supplémentaires, installation versionnée et relance.
L'activation de la capsule seule à la connexion sur **son PC** est explicitement
autorisée. Aucun nouveau paquet, aucune configuration d'agent, aucun sudo,
commit ou push dans ce lot.

Installé : `~/.local/share/systempulse/0.1.0-local-20261123-signals/`.
Anciennes versions et sauvegardes des lanceurs conservées. La capsule est
laissée visible, dashboard caché. SHA-256 du desktop installé :
`a402f96dc5871f278daa1237b2f6bc9aa855c8c5b83cdf91d307ead0a5fa8d82`.

## Reconnaissance des variations

Un module TypeScript pur compare les instantanés, sans nouvelle acquisition ni
modification du moteur d'alertes Rust. Il distingue observation et diagnostic.

- Flèches ↑/↓, écart chiffré et unité, en complément d'une couleur de bordure.
  Pas de clignotement. Les transitions respectent `prefers-reduced-motion`.
- Changements significatifs : RAM de 8 points, CPU/GPU de 20 points, température
  de 5 °C par rapport au dernier niveau de référence. Confirmation sur **deux
  instantanés nouveaux consécutifs** ; un pic isolé ne sonne pas. Ce sont des
  conventions de perception, pas des seuils de panne/surchauffe.
- Les nouvelles alertes évaluées du moteur, y compris un changement de gravité,
  sont signalées séparément. Aucune répétition sur chaque rafraîchissement.
- Pas de notification au premier relevé, pour un instantané en cache dupliqué,
  pour une valeur inconnue, ou après réinitialisation due à une panne/pause longue.
  Les valeurs manquantes ne deviennent pas zéro.
- Un événement par métrique est limité à une fois par minute. Messages conservés
  15 secondes, quatre maximum. Les baisses restent visuelles. Les variations
  ordinaires ont une flèche à partir de 1 point, 0,5 °C ou 16 Mio selon la métrique.
- Les cartes CPU/RAM/température/swap et cellules CPU/RAM des processus affichent
  leurs variations. Les cellules processus peuvent être actualisées pendant une
  recherche sans remplacer le champ ni perdre son focus.

## Analyse des différentes vues

| Interface | Comportement |
| --- | --- |
| Capsule compacte | Flèche RAM, contour d'attention pour événement confirmé, anneau SVG et valeur ; aucun clignotement |
| Panneau de capsule | Flèches/écarts CPU/RAM/GPU/température, messages persistants courts, commande Muet |
| Vue d'ensemble | Variations des cartes de mesures et bandeau global ; valeurs inconnues explicites |
| Processus | Écarts CPU/mémoire sur les cellules observées, recherche et focus conservés |
| Diagnostics | Règles Rust conservées ; bandeau des changements et nouvelles alertes séparé des valeurs |
| Historique | Graphique de session inchangé, bandeau du dernier changement ; aucun changement de couleur rétroactif des anciennes mesures |
| Intégration IA | Bandeau global, contrôles de préparation inchangés ; aucune transmission de notification ou mesure à un agent |
| Paramètres | Langue/thème conservés ; préférence sonore, test explicite du son et explication anti-répétition |

Les petites jauges non affichées comme mesures (GPU/batterie non qualifiés dans
l'overview, réseau sans vue détaillée) ne gagnent pas de seuil inventé ni de
signal de santé fictif. Les couleurs utilisent les accents déjà étudiés dans
`docs/architecture/WIDGET_DESIGN.md`, accompagnés de texte et de direction.

## Son discret et maîtrise des ressources

Son local Web Audio : sinusoïde 660 Hz, enveloppe douce d'environ 150 ms, gain
maximal 0,02. Aucun téléchargement, fichier sonore distant, accès micro ou
modification du volume système. Un contexte audio par vue, un oscillateur actif
maximum, nœuds déconnectés et contexte fermé à la fin de la vue.

**Seule la capsule** produit les sons automatiques pour éviter les doublons avec
le dashboard. Délai global minimal 30 secondes en plus de la limite par événement.
Le dashboard propose un test sonore volontaire, pas une seconde production
d'alertes. La préférence Muet est enregistrée localement et synchronisée entre
fenêtres. Par défaut les sons sont demandés selon l'accord de Kiunda, mais la
politique Web Audio peut exiger un premier clic/Entrée sur la capsule. L'état
affiche cette limite ; aucun contournement d'autoplay n'est ajouté.

Le fonctionnement logiciel de l'oscillateur est testé dans Chrome isolé ; cela ne
certifie pas le volume réellement entendu sur les haut-parleurs/casque de ce PC.
Un lancement automatique peut donc rester silencieux jusqu'au premier geste.

## Glisser et forme

Les six points ont été retirés du DOM. La capsule entière reçoit les gestes :
seuil de 6 pixels CSS, un seul appel au déplacement natif par geste, capture de
pointeur, aucun calcul de coordonnées de fenêtre en boucle JavaScript. Un glisser
n'ouvre pas les détails ; un clic ou une activation clavier les ouvre. Entrée et
Échap restent disponibles.

La silhouette est une pilule de 82 px de hauteur visible, courbes arrondies et
padding régulier. L'anneau de RAM est un cercle SVG antialiasé, progression avec
bout arrondi ; valeur et absence de capteur restent lisibles. Les aperçus
compact/clair ont été ouverts et relus.

GTK/Wayland conserve ici une surface native 320 × 200 et un panneau 400 × 620,
malgré une silhouette visible plus petite. Le placement absolu, le masque de
zone d'entrée et la fluidité ressentie d'un glisser sur ce compositeur ne sont
pas certifiés. Le test navigateur valide le geste/clic, pas un benchmark de
latence du gestionnaire de fenêtres.

## Démarrage demandé

- Sans argument, le binaire ouvre désormais uniquement la capsule.
- `--widget` l'explicite ; `--dashboard` demande l'interface principale.
  Si une instance existe déjà, utiliser Tableau de bord dans la capsule.
- L'entrée menu créée par l'installateur utilise également `--widget`.
- À l'installation interactive : confirmation d'installation puis question
  visible « Lancer la capsule seule à chaque connexion au bureau ? [O/n] ».
  Oui est présélectionné, Non désactive. EOF n'est jamais un accord ; les modes
  non interactifs doivent préciser `--autostart enable|disable|keep`.
- Sur ce PC : entrée `~/.config/autostart/io.systempulse.desktop.desktop`
  créée avec `Hidden=false` et `Exec="…/.local/bin/systempulse-desktop" --widget`.
  Elle a été relue et validée avec `desktop-file-validate`.
- Aucun redémarrage/reconnexion forcé pour tester. Le lancement sans argument a
  été exécuté réellement et le dashboard était absent/masqué au contrôle AT-SPI.

Pour revenir au choix désactivé lors d'une prochaine mise à jour, utiliser
`python3 scripts/install-user.py --update --autostart disable` puis confirmer
l'installation. Cela conserve le fichier géré avec `Hidden=true`, sans suppression
implicite. L'installateur conserve ses validations de chemins et sauvegardes.

## Tests réellement exécutés

| Contrôle | Résultat |
| --- | --- |
| TypeScript, tests et build frontend | PASS : **25 tests**, dont 7 nouveaux pour événements, cache, pics, indisponibilité, tendances, gestes et cooldown sonore |
| Tests navigateur isolés, timeout 55 s | PASS : glisser sans ouverture, clic, thèmes, arrondis/SVG, clavier/focus, MCP, Muet enregistré, scénario synthétique RAM persistante et **un seul oscillateur** sans son au démarrage ni répétition, responsive ; aucune erreur JS/console ni requête externe dans le scénario |
| `sh scripts/check.sh`, hors ligne, deux jobs, timeout 90 s | PASS : **31 tests headless**, fmt, Clippy et checks desktop |
| Clippy custom-protocol / tests Rust desktop | PASS sans avertissement ; **4 tests** |
| Tests installateur isolés | **7 tests PASS**, dont présélection visible, réponse négative/invalide et entrée menu capsule seule |
| Build natif embarqué | PASS hors ligne, deux jobs ; compilation incrémentale environ **2,38 s** |
| AT-SPI sur version installée | PASS : lancement capsule seule sans argument, agrandir/replier, masquer/réafficher, restaurer le dashboard puis fermeture le gardant caché |
| Menu, autostart, lanceur | `desktop-file-validate` et `sh -n` PASS ; journal de lancement vide au contrôle |
| Gitleaks local, secrets masqués | Aucun secret détecté dans le périmètre sources, caches et builds exclus |
| `git diff --check` | PASS |

Pas de nouveau test de charge, couverture, fuzzing guidé ou qualification
Windows/macOS. Les avis de dépendances Rust déjà documentés restent ouverts ;
aucune mise à jour de dépendance ni audit CVE vert n'est revendiqué dans ce lot.

Relevés disque : cache Tauri 3912 → 3916 Mio ; installations conservées 1131 →
1378 Mio, soit environ 251 Mio de supplément sur ces emplacements, hors petits
artefacts. Le pic RAM global n'est pas mesuré et l'objectif 2 Gio n'est pas un
plafond système imposé. Aucun cache supprimé.
