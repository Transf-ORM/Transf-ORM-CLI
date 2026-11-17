# Compte rendu de réunion du [12/08/2026]

## Début de la réunion

14h (UTC +2)

**Présents :** @everyone
**Absents :** N/A

## Points abordés

- Discussion sur le livrable finale (binaire, librairies, SaaS, ...)
- Discussion sur le travail déjà présent sur le repo => architecture du Core pratiquement complète (manque la partie modification du code)
- Rédaction d'un document sur les flag/commandes de notre binaires / CLI / API

## Décisions prises

- Choix du livrable :
    - Lib/Crate pour la logique métier (Core)
    - Lib/Crate pour la CLI => Call les fonctions du Core
    - Lib/Crate pour le SaaS => Ouvre les accès aux routes API
    => Résultat : Un seul binaire qui utilise des flags pour donner l'accès aux différentes parties

## Note importantes

- Le Core doit être designer de façon à pouvoir créer une instance du Core

## Fin de la réunion

17h (UTC +2)

[Transf-ORM_API_CLI_Specification.pdf](Compte%20rendu%20de%20r%C3%A9union%20du%20%5B12%2008%202026%5D/Transf-ORM_API_CLI_Specification.pdf)

[transf-orm-schema.dbml](Compte%20rendu%20de%20r%C3%A9union%20du%20%5B12%2008%202026%5D/transf-orm-schema.dbml)