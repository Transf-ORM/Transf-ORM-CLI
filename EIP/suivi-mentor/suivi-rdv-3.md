# Suivi de projet – Transf-ORM (RDV 3)

État au 12/11/2026. Précédent RDV : 19/10/2026.

## Avancement depuis le dernier RDV

- **CLI** : les flags `--verbose` et `--debug <level>` sont fusionnés sur `dev` (#203, issue #54 fermée). Un **mode interactif plein écran** (ratatui) est développé sur la branche `48-…-repl-tui` : `transf-orm` sans commande ouvre une session avec détection de l'ORM du dossier courant, menu de commandes `/`, historique et repli ligne à ligne hors terminal. Il n'a pas encore de PR.
- **Mode serveur** : il démarre maintenant un vrai serveur HTTP axum + tokio avec `GET /health` (#204), complété par `--ip` / `--host`, des erreurs JSON uniformes et des logs de requêtes (#205). Les deux PR sont ouvertes depuis le 31/10 et pas encore relues.
- **Persistance du serveur** : une couche SeaORM + PostgreSQL est en place sur la branche `90-spike-persistence-layer-orm` : première entité `users`, migrations, `docker-compose.yml` et test de fumée contre une vraie base en CI. Elle n'a pas encore de PR.
- **Tests** : environ 60 tests sur `dev` (+8 depuis le dernier RDV). Environ 55 autres attendent sur les branches non fusionnées (44 pour le mode interactif, 10 pour le serveur, 1 pour la base).
- **Release** : `main` n'a pas bougé depuis le 19/10. La PR de release **0.4.0** (#39) est toujours ouverte, donc il n'y a toujours **pas de publication npm** (la dernière version publiée reste la 0.3.4, de mars).
- **Robustesse** : un bug a été trouvé et documenté (#210) : un schéma Prisma avec des valeurs d'attributs très imbriquées provoque un dépassement de pile au lieu d'une erreur de parsing.

## Bilan des actions prévues au dernier RDV

| Action prévue | Statut | Résumé |
|---|---|---|
| Performance : protocole de mesure écrit et committé | ❌ Pas fait | Aucun protocole dans les dépôts |
| Performance : schémas de tailles croissantes (10 / 100 / 1 000 modèles) | ❌ Pas fait | — |
| Performance : hyperfine ≥ 20 runs, export versionné, mémoire résidente | ❌ Pas fait | Pas de dossier `benches/` ni de résultats versionnés |
| Performance : consigner la machine de mesure | ❌ Pas fait | — |
| Architecture : versionner les schémas (`.excalidraw` + SVG) | ✅ Fait | Architecture et ER (Excalidraw + SVG + PNG) et schéma DBML versionnés dans `EIP/` sur `main` |
| Architecture : case « schémas mis à jour » dans le template de PR | ❌ Pas fait | Template de PR inchangé |
| Architecture : ADR-0005 (licence MIT) | ✅ Fait | Écrit sous le numéro ADR-0007, le 0005 ayant été pris par le framework HTTP |
| Problème dur : mesure sur `test.prisma`, critères chiffrés | ❌ Pas fait | Reporté pour la 2ᵉ fois |
| Veille : rythme, répartition, premières entrées | ❌ Pas fait | Reportée pour la 2ᵉ fois ; aucune trace dans les dépôts |
| Revue de code : accord d'organisation dans `CONTRIBUTING.md` | 🟡 Partiel | Accord du 10/08 versionné dans `EIP/organisation/` ; le lien depuis `CONTRIBUTING.md` attend sa relecture |
| Revue de code : plus de fusion `dev` → `main` sans relecture | ✅ Respecté | Aucune fusion vers `main` sur la période (mais aucune release non plus) |
| Ouverture : 3 issues good first issue | ❌ Pas fait | Toujours 0 |

## Objectif obligatoire 1 : suivre l'existant et s'en servir pour résoudre le problème dur

- **Critères de succès chiffrés** : pas fait, pour la 2ᵉ fois. L'importeur Prisma n'a pas été lancé sur `test.prisma`, donc les seuils envisagés (préservation ≥ 95 %, validité 100 %, 0 % de perte silencieuse) ne sont toujours pas confrontés au réel.
- **Signal utile malgré tout** : le bug #210 (dépassement de pile sur attributs imbriqués) montre que l'importeur peut planter sur une entrée que la grammaire accepte. C'est un cas concret à intégrer aux critères du problème dur : la validité doit inclure « jamais de crash, toujours une erreur exploitable ».
- **Veille** : le rythme et la répartition ne sont toujours pas définis, et le journal est toujours à 0/20 entrées.
- **Mise en pratique de l'état de l'art côté serveur** : les deux ADR de la période (voir objectif 2) s'appuient sur une vraie comparaison d'alternatives : 4 frameworks HTTP, puis 3 couches de persistance avec un tableau de 9 critères. C'est la démarche attendue pour le problème dur, mais elle n'a pas encore été appliquée au cœur du sujet, la conversion.
- **Partage interne** : toujours 0/6 présentations et 0/3 éléments de veille réutilisés.
- **État de l'art, prototypes, décision** : pas d'avancement. La première échéance (décembre) approche.

## Objectif obligatoire 2 : structurer, documenter et fiabiliser l'architecture

- **Structure** : le découpage en workspace tient. Le serveur reste isolé dans `crates/server`, avec seulement `transf_orm_server::run` qui construit un runtime tokio, et le CLI reste synchrone. Le mode interactif a ajouté un module `ui` réutilisable (`Panel`, `SelectList`) et `registry::detect_in_dir` dans `core`.
- **ADR : 7 au total (+3)**, tous dans le wiki avec un index :
  - **ADR-0005 (axum + tokio)** : axum a été comparé à actix-web, poem et Rocket. Il a été retenu pour ses handlers qui sont de simples futures tokio, son écosystème de middleware tower et son routeur testable sans ouvrir de port. Conséquence acceptée : axum est en pré-1.0 et chaque montée de version demandera une relecture du changelog. Déclencheur de révision : si axum stagne avant la 1.0, reconsidérer actix-web.
  - **ADR-0006 (SeaORM + PostgreSQL)** : SeaORM a été comparé à sqlx et Diesel. Il a été retenu parce que le modèle (une vingtaine de tables liées) est surtout du CRUD relationnel, qu'il est async natif et qu'aucune base n'est nécessaire au build. Conséquences acceptées : les requêtes ne sont pas vérifiées à la compilation (d'où un test insertion + lecture par entité), et la version minimale de Rust passe à 1.94.
  - **Point d'attention** : l'ADR-0006 indique que le binaire release passe de **2,1 Mo à 3,0 Mo** avec SeaORM, et qu'il grossira encore. C'est exactement le déclencheur de révision de l'ADR-0004 (placer `server` derrière une feature Cargo). Ce déclencheur doit être évalué avant de fusionner la branche de persistance.
  - **ADR-0007 (licence MIT)** : Apache-2.0 a été comparée à MIT. MIT a été retenue parce que c'est la licence par défaut de l'écosystème npm, qu'elle est courte et compatible GPLv2, et qu'elle n'impose pas de fichier `NOTICE` à ceux qui redistribuent le binaire. Conséquence acceptée : pas de clause de brevets explicite. Déclencheur de révision : une contribution d'entreprise qui exige cette clause, ou un besoin de double licence pour le SaaS.
- **Documents d'architecture versionnés** : un dossier `EIP/` sur `main` regroupe maintenant les documents de travail de l'équipe, avec les prénoms remplacés par les pseudos GitHub :
  - le diagramme d'architecture et le modèle entité-association, en sources Excalidraw avec leurs exports SVG et PNG ;
  - le schéma de la base du serveur en DBML ;
  - la spécification des routes API et du binaire CLI (rôles et permissions, environ 50 routes, flags, commandes) ;
  - l'accord d'organisation de l'équipe ;
  - le plan d'action et les suivis mentor.

  Ils sont donc dans le dépôt, dans un format qui se compare d'une version à l'autre. Il reste à mettre le diagramme d'architecture à jour avec les choix de la période (axum, SeaORM, docker-compose), puisqu'il date du 19/10.
- **Documentation** : le wiki a avancé : une page par flag global (21/10), la documentation du serveur (`--ip`, `--host`, erreurs JSON) et celle du mode interactif (31/10). Les README des branches sont mis à jour avec chaque fonctionnalité.
- **Revue de code** : la règle tient sur ce qui est fusionné. #203 a été approuvée par un autre membre, et #207 (Dependabot) aussi. En revanche, **#204 et #205 attendent une relecture depuis 12 jours**, et le mode interactif et la persistance (plus de 2 500 et 3 000 lignes) n'ont pas de PR. Le volume de code en attente de relecture est maintenant supérieur à celui fusionné sur la période. L'accord d'organisation n'est toujours pas dans `CONTRIBUTING.md`.
- **CI** : le correctif Dependabot fonctionne. Les PR ciblent `dev` et passent la règle (#207 fusionnée, #208 fermée). La branche de persistance ajoute un service PostgreSQL au job `tests`. Manquent toujours `cargo doc` (#118), la vérification MSRV (#117), devenue plus importante avec `rust-version = 1.94`, et le seuil de couverture (#116).
- **Tests** : ~60 sur `dev` ; le pourcentage Codecov n'est toujours pas relevé (échéance fin décembre).

## Objectif complémentaire 1 (B) : ouvrir le projet aux contributions externes

- **Licence** : MIT, cohérente partout, et maintenant justifiée par l'ADR-0007 du wiki.
- **Issues pour débutants** : toujours 0 good first issue sur 128 issues ouvertes. #113 (code mort `bin/transform.js`) et #120 (wiki « Doxygen » → rustdoc) restent candidates. La nouvelle issue #209 (logo) pourrait servir de troisième, pour un contributeur non développeur.
- **CONTRIBUTING.md** : sans changement.
- **Recherche et accueil de contributeurs** : non commencés. Toujours 2 stargazers, 0 fork.
- **Frein à l'ouverture** : sans release depuis mars, un contributeur externe qui installe le paquet npm récupère une version qui n'a ni le workspace, ni les nouveaux flags, ni le mode serveur.

## Objectif complémentaire 2 (D) : mesurer, tester et optimiser la performance technique

C'était la priorité annoncée au dernier RDV, et **rien n'a été fait**.

- Aucune des 4 actions prévues n'a été faite : pas de protocole, pas de schémas générés, pas de mesure versionnée, pas de relevé de la machine.
- La seule mesure reste celle du RDV précédent (3,4 ms sur `test.prisma`), non versionnée et non représentative.
- **Données de performance apparues indirectement** :
  - la taille du binaire (2,1 → 3,0 Mo), relevée dans l'ADR-0006 ;
  - le bug #210, qui montre un comportement pathologique sur des entrées profondes.

  Ces deux points pourraient entrer dans le protocole (taille du binaire suivie à chaque release, schémas profonds en plus des schémas larges).
- L'axe n'est toujours pas validé avec vous.

## Questions pour vous

1. Les seuils du problème dur (préservation ≥ 95 %, validité 100 %, 0 % de perte silencieuse) vous semblent-ils réalistes ? Faut-il y ajouter « 0 crash, toujours une erreur de parsing » au vu de #210 ? (question ouverte depuis 2 RDV)
2. Êtes-vous d'accord avec l'axe de performance proposé : temps d'exécution de `convert` selon la taille du schéma, complété par la taille du binaire ? (question ouverte depuis 2 RDV)
3. Les schémas sont versionnés en Excalidraw (source) avec leurs exports SVG et PNG, dans `EIP/`. Ce format vous convient-il, ou attendez-vous du Mermaid rendu directement par GitHub ?
4. Le binaire a grossi de 43 % avec la couche de persistance. Faut-il déjà placer le serveur derrière une feature Cargo (déclencheur de l'ADR-0004), ou attendre d'avoir une mesure plus complète ?

## Actions prévues pour dans 3 semaines

- **Performance (priorité, reportée)** :
  - écrire et committer le protocole de mesure ;
  - générer des schémas de tailles croissantes (10, 100, 1 000 modèles), plus un schéma profond (cas #210) ;
  - mesurer avec hyperfine (≥ 20 runs, export versionné), relever la mémoire résidente maximale et la taille du binaire ;
  - consigner la machine de mesure.
- **Revue et intégration** :
  - relire et fusionner #204 puis #205 ;
  - ouvrir les PR du mode interactif (#48) et de la persistance (#90) ;
  - fusionner la release 0.4.0 (#39) pour publier sur npm ;
  - relire et fusionner le lien de `CONTRIBUTING.md` vers l'accord d'organisation.
- **Architecture** :
  - évaluer le déclencheur de l'ADR-0004 (feature Cargo `server`) avant de fusionner #90 ;
  - mettre à jour le diagramme d'architecture avec axum, SeaORM et PostgreSQL ;
  - ajouter la case « schémas mis à jour » au template de PR.
- **Problème dur** (reporté 2 fois) : faire tourner l'importeur Prisma sur `test.prisma`, compter ce qu'une approche naïve gérerait mal, fixer les critères chiffrés, et corriger #210.
- **Veille** (reportée 2 fois) : définir le rythme et la répartition, puis saisir les premières entrées du journal.
- **Ouverture** : étiqueter 3 issues good first issue (#113, #120, #209).
