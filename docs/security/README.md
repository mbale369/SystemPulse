# Contraintes de sécurité

Aucune collecte ni écoute réseau en Phase 01. Le futur MCP doit être local et
en lecture seule par défaut ; privilégier stdio, sans télémétrie Internet.
Ne pas écrire de diagnostic sur stdout du transport MCP.

Minimiser les données : les arguments de processus, chemins et identifiants
peuvent contenir des secrets. Définir accès, rétention et masquage avant collecte.
Aucune commande privilégiée, action matérielle ou exécution automatique implicite.
Borner fréquence, mémoire, délais et taille des réponses avant intégration.
