# EIP — documents de travail de l'équipe

Ce dossier versionne les documents de pilotage et de conception de Transf-ORM produits dans le cadre de l'EIP (Epitech Innovative Project). Il ne fait pas partie du paquet npm.

Les membres de l'équipe y sont désignés **uniquement par leur pseudo GitHub** : nitrached, Timeotr, AymericJM, tuturicide, Babouye. Toute personne extérieure à l'équipe y est désignée par son rôle. Respecter cette règle pour tout nouveau document.

## Contenu

| Dossier | Document | Description |
|---|---|---|
| [`organisation/`](organisation) | [`organisation-equipe.md`](organisation/organisation-equipe.md) | Accord d'organisation du 10/08/2026 : réunions, workflow Git, règles et critères de review, répartition du code |
| [`specifications/`](specifications) | [`api-cli-specification.md`](specifications/api-cli-specification.md) | Spécification des routes de l'API (rôles, permissions) et du binaire CLI (modes, flags, commandes) |
| [`architecture/`](architecture) | [`architecture.svg`](architecture/architecture.svg) | Diagramme d'architecture v1 : composants, flux, dépendances externes, points de défaillance, alternatives écartées |
| [`database/`](database) | [`er-database.svg`](database/er-database.svg) | Modèle entité-association de la base du serveur |
| | [`transf-orm-schema.dbml`](database/transf-orm-schema.dbml) | Schéma de la base du serveur en DBML (à coller sur dbdiagram.io) |
| [`suivi-mentor/`](suivi-mentor) | `suivi-rdv-N.md` | Comptes rendus d'avancement préparés pour chaque rendez-vous mentor |
| [`plan-action/`](plan-action) | [`plan-action-eip.md`](plan-action/plan-action-eip.md) | Plan d'action EIP initial (contexte, jalons, sujets non techniques) |

## Diagrammes

Les diagrammes sont dessinés avec le plugin Excalidraw d'Obsidian :

- `*.excalidraw.md` : la source, à ouvrir dans Obsidian avec le plugin Excalidraw ;
- `*.svg` / `*.png` : les exports, à régénérer à chaque modification de la source.

Un changement structurant (nouveau composant, nouvelle table, nouvelle dépendance externe) doit mettre à jour le diagramme concerné dans la même PR. Les décisions qui en découlent sont consignées dans les [ADR du wiki](https://github.com/Transf-ORM/Transf-ORM-CLI/wiki/Architecture-Decision-Records).
