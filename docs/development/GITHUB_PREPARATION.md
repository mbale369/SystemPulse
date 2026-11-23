# Préparation GitHub — sans publication

Le 23 novembre 2026, Kiunda a autorisé le suivi Git et un commit **local**.
Le compte GitHub `mbale369` a été vérifié en lecture seule. Aucun dépôt distant
créé, aucun remote ajouté, aucun push ni workflow distant exécuté.

## Avant tout push

- Confirmer le nom et l'URL du dépôt sous `mbale369` (aucune cible présumée).
- Confirmer sa visibilité. La licence du projet n'est pas choisie : pas de
  LICENSE ajoutée, pas de publication open source implicite.
- Lire le rapport de suivi sécurité : des avis Rust restent ouverts ; ce commit
  n'est ni une release qualifiée ni une déclaration de sécurité absolue.
- Les workflows sont manuels pour éviter la consommation Actions au premier push.
- Lockfiles suivis ; `target`, `node_modules`, bundles, bases SQLite, exports de
  télémétrie, secrets et rapports locaux exclus. Icônes source suivies.
- Rapports historiques conservés : ils contiennent des chemins de développement
  et informations machine. Les relire avant une éventuelle publication publique.

## Commandes futures, à exécuter seulement après accord de destination

Si un dépôt vide existe déjà, vérifier son propriétaire et sa visibilité avec
`gh repo view mbale369/NOM_CONFIRME --json nameWithOwner,visibility,url`.
Après confirmation explicite de cette destination et du push :

```sh
git remote add origin https://github.com/mbale369/NOM_CONFIRME.git
git remote -v
git status --short
git log --oneline -10
git push -u origin main
```

Ne pas utiliser `--force`. Si le dépôt distant contient déjà des commits,
interrompre cette procédure et comparer les historiques avant toute intégration.
Ne pas lancer de `gh repo create` ou de workflow Actions implicitement.
