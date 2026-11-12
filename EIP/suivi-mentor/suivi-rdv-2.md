# Suivi de projet – Transf-ORM (RDV 2)

État au 19/10/2026. Précédent RDV : 02/10/2026.

## Avancement depuis le dernier RDV

- Le dépôt est passé en **workspace Cargo à 4 crates** (issue #42 fermée) : `core` (moteur), `cli`, `server` et `transf-orm`, le binaire unique qui choisit le mode avec `--cli` (par défaut) ou `--serveur` (#198). Le mode serveur ne fait encore que démarrer et afficher son port.
- Import Prisma : détection des behaviors `createdAt`, `softDelete` et `versioning` (#195), et correction des guillemets échappés dans les chaînes.
- **52 tests** passent (39 au dernier RDV), dont 6 tests d'intégration sur le binaire lui-même : le chemin CLI de `main.rs` est maintenant testé.
- La PR de release **0.4.0** (#39, release-please) est prête et attend d'être fusionnée sur `main` ; la publication npm suivra automatiquement.

## Bilan des actions prévues au dernier RDV

| Action prévue | Statut | Résumé |
|---|---|---|
| Diagramme d'architecture v1 | ✅ Fait | Produit, mais en Excalidraw et pas encore versionné dans le dépôt |
| ADR-0004 | ✅ Fait | Workspace de librairies derrière un binaire unique |
| Problème dur : mesure sur test.prisma, critères chiffrés | ❌ Pas fait | — |
| Veille : rythme, répartition, premières entrées | ❌ Pas fait | Journal toujours à 0/20 |
| Revue de code : 1 approbation obligatoire | ✅ Fait | Accord d'organisation pas encore committé |
| README réécrit (#112) | ✅ Fait | Issue #112 fermée |
| Licence : trancher Apache-2.0 / MIT | ✅ Fait | MIT ; justification à consigner |
| 3 issues good first issue | ❌ Pas fait | Toujours 0 |
| Performance : hyperfine + première mesure | 🟡 Partiel | Une seule mesure, non versionnée, peu exploitable |

## Objectif obligatoire 1 : suivre l'existant et s'en servir pour résoudre le problème dur

- **Critères de succès chiffrés** : pas fait. La mesure sur `test.prisma` n'a pas été lancée, donc les seuils envisagés (préservation ≥ 95 %, validité 100 %, 0 % de perte silencieuse) ne sont toujours pas confrontés au réel. Reporté au prochain RDV.
- **Veille** : le rythme et la répartition entre membres ne sont pas définis, et le journal est toujours à 0/20 entrées. Les canaux Discord restent en place.
- **Partage interne** : toujours 0/6 présentations et 0/3 éléments de veille réutilisés.
- **État de l'art, prototypes, décision** : pas d'avancement. Les échéances (décembre à avril) ne sont pas encore en jeu.

## Objectif obligatoire 2 : structurer, documenter et fiabiliser l'architecture

- **Structure** : le split en workspace, annoncé comme « prévu mais pas commencé » au dernier RDV, est fait. Chaque crate a un rôle délimité, et le CLI comme le serveur passent par les mêmes fonctions de `core`.
- **Diagramme d'architecture : fait.** La v1 couvre :
  - les composants actuels et le mode serveur prévu (API, worker de jobs, PostgreSQL, front Next.js) ;
  - les flux de données et les dépendances externes : aujourd'hui des crates uniquement ; plus tard GitHub/GitLab (OAuth) et un fournisseur de paiement ;
  - 12 points de défaillance identifiés et numérotés sur le schéma ;
  - un tableau des alternatives envisagées et écartées.

  Un modèle entité-association de la future base du serveur a aussi été produit (12 entités, 23 relations). **Écart avec ce qui était annoncé** : les schémas sont en Excalidraw, pas en `.mmd`, et ils ne sont pas encore dans le dépôt. Ils doivent y être versionnés, à côté de la documentation, et mis à jour à chaque changement structurant.
- **ADR : ADR-0004 écrit**, ce qui atteint le minimum de 4 ADR sur l'année.
  - **Contexte :** le déclencheur de révision de l'ADR-0003 (« package unique pour le MVP ») a été atteint, puisque le mode serveur devient un second consommateur du `core`.
  - **Options :** deux binaires séparés, ou un workspace de librairies (`core`, `cli`, `server`) derrière un binaire unique qui choisit le mode à l'exécution.
  - **Choix :** le binaire unique, pour n'avoir qu'un paquet npm et qu'une version, avec une logique métier en un seul endroit.
  - **Conséquence acceptée :** les dépendances du serveur seront compilées dans le binaire installé par tous les utilisateurs du CLI.
  - **Déclencheur de révision :** si le binaire (1,8 Mo aujourd'hui) ou son temps de build grossit nettement, placer `server` derrière une feature Cargo plutôt que de séparer les binaires.

  Le choix de la licence MIT reste à consigner dans un ADR-0005.
- **Documentation : README réécrit.** Il donne l'état réel de chaque commande (disponible ou prévue), les modes d'exécution, les flags globaux et la structure du workspace. Les commandes inexistantes ne sont plus présentées comme disponibles. L'issue #112 est fermée.
- **Revue de code : en place.** Une règle de dépôt (ruleset `protect-dev-main`) impose 1 approbation sur `main` et `dev`. Les 4 PR de fonctionnalité depuis le dernier RDV ont été relues et approuvées par un autre membre (#193, #194, #195, #198), contre 0/30 au relevé précédent. Deux réserves :
  - la PR #202 (fusion `dev` → `main`) est passée sans relecture, en contournant la règle ;
  - l'accord d'organisation du 10/08 n'est toujours pas committé dans `CONTRIBUTING.md`.
- **CI** : toujours 5 workflows. **Le blocage des PR Dependabot est réglé** : Dependabot cible maintenant `dev` (`target-branch: dev`), ce qui passe la règle de `enforce-main-source.yml`. Le correctif est arrivé sur `main` le 19/10 avec #202. Les PR ouvertes avant cette date vers `main` (#196, #197, #199 à #201) ont été fermées, et les mises à jour de pest ont été appliquées à la main. Manquent toujours : `cargo doc` (#118), MSRV (#117), seuil de couverture (#116).
- **Tests** : 52 tests ; le test du chemin CLI est fait. Le pourcentage Codecov n'est pas encore relevé (échéance fin décembre).
## Objectif complémentaire 1 (B) : ouvrir le projet aux contributions externes

- **Licence : tranchée, MIT.** `LICENSE`, `README` et `package.json` sont maintenant cohérents, et GitHub détecte la licence MIT (contre « other » avant). La justification du choix n'est écrite nulle part pour l'instant : elle doit faire l'objet d'un ADR.
- **README** : fait (voir objectif 2).
- **Issues pour débutants** : toujours 0 good first issue sur 127 issues ouvertes. #112 n'est plus candidate puisqu'elle est réglée ; restent #113 (code mort `bin/transform.js`) et #120 (wiki), et il en faut une troisième.
- **CONTRIBUTING.md** : sans changement (délai de réponse cible, périmètre de l'ouverture et correction du wiki manquent toujours).
- **Recherche et accueil de contributeurs** : non commencés. Toujours 2 stargazers.

## Objectif complémentaire 2 (D) : mesurer, tester et optimiser la performance technique

C'est l'action la moins aboutie de la période.

- **Fait** : hyperfine est installé, et une première mesure de `transf-orm convert test.prisma` en build release donne **3,4 ms ± 0,3** (min 2,5, max 4,2).
- **Limites de cette mesure** :
  - le résultat est dans un fichier local non versionné (`bench-convert.md`) : il n'est ni reproductible ni comparable dans le temps ;
  - le nombre de runs, la machine et les conditions ne sont pas consignés ;
  - la mémoire résidente maximale n'a pas été mesurée ;
  - surtout, `test.prisma` (9,4 Ko) est trop petit pour être utile. À 3,4 ms, on mesure surtout le temps de démarrage du processus, pas le parseur : cette mesure ne dit rien de l'évolution du temps quand le schéma grossit, alors que c'est l'axe proposé.
- **Pas fait** : protocole de mesure écrit, dossier `benches/`, workflow `benchmark.yml`. L'axe n'est pas encore validé avec vous.

## Questions pour vous

1. Les seuils du problème dur (préservation ≥ 95 %, validité 100 %, 0 % de perte silencieuse) vous semblent-ils réalistes ? (question du RDV précédent, toujours ouverte)
2. Êtes-vous d'accord avec l'axe de performance proposé : temps d'exécution de `convert` selon la taille du schéma ? (question du RDV précédent, toujours ouverte)
3. Le diagramme d'architecture est en Excalidraw plutôt qu'en Mermaid. Des fichiers `.excalidraw` versionnés dans le dépôt, avec leur export SVG, vous conviennent-ils ?

## Actions prévues pour dans 3 semaines

- **Performance (priorité)** :
  - écrire et committer le protocole de mesure ;
  - générer des schémas de tailles croissantes (10, 100, 1 000 modèles) ;
  - mesurer avec hyperfine (≥ 20 runs, export versionné) et relever la mémoire résidente maximale ;
  - consigner la machine de mesure.
- **Architecture** : versionner les sources des schémas (`.excalidraw` et export SVG) dans le dépôt, ajouter dans le template de PR une case « schémas mis à jour » pour les changements structurants, et écrire ADR-0005 (licence MIT).
- **Problème dur** (reporté) : faire tourner l'importeur Prisma sur `test.prisma`, compter ce qu'une approche naïve gérerait mal, puis fixer les critères chiffrés.
- **Veille** (reportée) : définir le rythme et la répartition, puis saisir les premières entrées du journal.
- **Revue de code** : committer l'accord d'organisation dans `CONTRIBUTING.md`, et ne plus fusionner `dev` → `main` sans relecture.
- **Ouverture** : étiqueter 3 issues good first issue (#113, #120 et une troisième à choisir).
