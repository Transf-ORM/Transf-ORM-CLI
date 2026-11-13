# Cod3x – Plan d’action EIP

## 1. Contexte

Cod3x est un outil d’ingénierie logicielle conçu pour résoudre un problème ponctuel mais complexe dans le développement backend moderne : la fragmentation des représentations de schémas de bases de données entre différents outils et ORM. Les développeurs utilisent fréquemment des technologies comme Prisma, Drizzle, TypeORM ou encore des outils de modélisation comme DBML. Chacun de ces outils possède son propre format et ses propres conventions pour décrire les schémas de base de données, ce qui rend les migrations ou l’interopérabilité complexes.

Aujourd’hui, la plupart des outils existants ne proposent que des conversions unidirectionnelles ou partielles. Cela entraîne souvent une perte d’information, des réécritures manuelles de schémas et un risque élevé d’incohérences entre la structure de la base de données, le code applicatif et la documentation.

Cod3x a pour objectif de fournir un système universel de conversion et de gestion de schémas. Le projet introduit une représentation intermédiaire neutre (pivot) capable de décrire un schéma de base de données indépendamment d’un ORM ou d’un langage de modélisation spécifique. Grâce à ce pivot, Cod3x permet des conversions réversibles entre formats, la génération automatique de migrations et la création de types et de schémas de validation.

L’objectif global du projet est de créer un outil fiable permettant aux développeurs de :

* Convertir des schémas entre différents ORM sans perte d’information
* Maintenir une source unique de vérité pour la structure de la base de données
* Générer automatiquement des types et des couches de validation
* Suivre l’évolution d’un schéma grâce à un système de versionnement sémantique

Le projet est développé dans le cadre de la track technique de l’EIP.

## 2. Spécifications techniques

### Stack technologique

Le coeur de Cod3x sera développé en **Rust** afin de bénéficier de hautes performances, d’un typage fort et d’une sécurité mémoire élevée. Rust est également adapté à la création d’outils en ligne de commande et à l’implémentation de parseurs efficaces.

L’architecture du projet comportera plusieurs composants.

**Moteur principal (Rust)**

* Définition de la représentation pivot du schéma
* Parseurs pour les différents formats supportés
* Générateurs pour exporter vers différents formats
* Algorithmes de diff et de merge

**Interface CLI**

* Commandes de conversion
* Validation de schémas
* Génération de migrations
* Gestion de l’historique et rollback

**Plateforme SaaS (évolution possible)**

* Interface web développée avec Next.js
* Éditeur visuel de schémas
* Collaboration en temps réel
* Synchronisation avec des dépôts Git

### Fonctionnalités principales

Cod3x proposera plusieurs fonctionnalités principales.

**Conversion de schémas**

Les utilisateurs pourront importer un schéma écrit en DBML ou dans un langage d’ORM et le convertir vers un autre format.

Exemples de conversions :

* DBML → Prisma
* Prisma → Drizzle
* Drizzle → DBML

Toutes les conversions passeront par la représentation pivot interne.

**Migration entre ORM**

Cod3x permettra de migrer automatiquement entre différents ORM. L’outil traduira les modèles, relations, contraintes et types tout en respectant les conventions de l’ORM cible.

En plus de la conversion du schéma de base de données, l’outil analysera également le **code applicatif utilisant l’ORM** afin d’adapter les appels correspondants. Les requêtes, modèles importés et structures utilisées dans le code devront être ajustés pour correspondre à l’API du nouvel ORM.

Cod3x vise donc à assister la migration non seulement au niveau du schéma mais aussi au niveau du **code applicatif**, en générant des transformations ou des indications permettant de modifier les appels à l’ORM dans l’application.

Un rapport de conversion sera généré afin d’indiquer les éventuelles différences, transformations appliquées et adaptations nécessaires dans le code.

**Génération de types et validations**

À partir du schéma pivot, Cod3x pourra générer :

* des types TypeScript
* des schémas de validation Zod

Cela garantit la cohérence entre la structure de la base de données et la logique applicative.

**Versionnement des schémas**

Chaque état d’un schéma sera sérialisé dans le format pivot et stocké. Le système permettra :

* la comparaison sémantique de deux versions
* la détection de conflits de fusion
* le retour à une version précédente
* la génération automatique de scripts de migration SQL

### Jalons du projet

**Jalon 1 – Proof of Concept**

Objectifs :

* Implémenter une représentation pivot minimale
* Implémenter un parseur Prisma
* Implémenter un générateur Prisma
* Valider une conversion complète (Prisma → Pivot → Prisma)

Critères de réussite :

* aucune perte d’information sur des schémas simples
* performance de conversion inférieure à 100 ms pour de petits schémas

**Jalon 2 – Outil CLI principal**

Objectifs :

* Ajouter le support du format DBML
* Implémenter les commandes de conversion
* Implémenter la fonctionnalité de diff de schéma

**Jalon 3 – Support multi‑ORM**

Objectifs :

* Ajouter le support de Drizzle et TypeORM
* Étendre le pivot pour supporter des fonctionnalités avancées
* Améliorer les systèmes d’erreurs et d’avertissements
* Changer le code applicatif de l'orm utilisé

**Jalon 4 – Versionnement et migrations**

Objectifs :

* Implémenter l’historique des schémas
* Implémenter le diff et le merge sémantiques
* Générer automatiquement les scripts de migration SQL

**Jalon 5 – Prototype SaaS**

Objectifs :

* Développer une interface web basique
* Ajouter un éditeur visuel de schémas
* Intégrer la synchronisation avec Git

## 3. Spécifications non techniques

### Sujet obligatoire – Évaluer et intégrer de nouvelles technologies

Dans le cadre du projet Cod3x, un benchmark des technologies existantes a été réalisé afin de choisir les outils les plus adaptés à l’implémentation du projet et d’analyser l’écosystème des ORM et de la gestion de schémas de bases de données.

#### Benchmark des ORM et outils de modélisation

Plusieurs ORM modernes ont été étudiés, notamment **Prisma, Drizzle ORM, TypeORM, Sequelize et MikroORM**. Ces outils proposent différentes approches pour définir les modèles de données et interagir avec la base de données.

Prisma utilise un langage de schéma dédié et génère un client fortement typé. Drizzle adopte une approche SQL-first où les schémas sont directement écrits en TypeScript. TypeORM et Sequelize sont des ORM plus anciens basés sur un modèle orienté objet.

L’analyse de ces outils a permis d’identifier des différences importantes dans la représentation des schémas, les conventions utilisées et les fonctionnalités supportées. Cette diversité rend les migrations entre ORM complexes et renforce l’intérêt d’un outil capable de convertir et synchroniser ces différentes représentations.

Le benchmark a également porté sur des outils de modélisation comme **DBML**, utilisé par la plateforme dbdiagram.io. Ce langage permet de représenter visuellement un schéma de base de données mais reste principalement utilisé pour la documentation.

Certains outils spécialisés comme **prisma-dbml-generator** ou **zod-prisma** ont également été étudiés. Ils permettent de générer certaines représentations à partir de schémas existants mais restent limités à des conversions unidirectionnelles.

Ces observations ont confirmé l’absence d’un outil capable de gérer des conversions bidirectionnelles entre plusieurs ORM, ce qui constitue l’objectif principal de Cod3x.

#### Benchmark des technologies d’implémentation

Un benchmark a également été réalisé pour choisir les technologies utilisées pour développer Cod3x.

Plusieurs langages ont été étudiés pour l’implémentation du moteur principal, notamment **Rust, Go et TypeScript**. Rust a été retenu pour plusieurs raisons : ses performances élevées, sa sécurité mémoire sans garbage collector et sa capacité à produire des outils en ligne de commande rapides et fiables. Ces caractéristiques sont particulièrement adaptées à la création de parseurs et de systèmes de transformation de données.

Go a également été considéré pour sa simplicité et ses bonnes performances, mais Rust offre un contrôle plus précis sur la gestion mémoire et permet de construire des outils plus optimisés pour ce type de traitement.

TypeScript aurait facilité l’intégration avec l’écosystème JavaScript, mais ses performances sont moins adaptées pour un moteur de transformation complexe.

Pour la partie interface web, **Next.js** a été choisi. Next.js fournit un environnement complet pour développer des applications web modernes avec React, incluant la gestion du routing, le rendu côté serveur et une bonne intégration avec l’écosystème JavaScript. Cette technologie est adaptée à la création d’une interface graphique permettant de visualiser et modifier les schémas de bases de données.

Ce benchmark technologique a permis de définir une architecture combinant un moteur performant écrit en Rust et une interface web moderne basée sur Next.js.

### Sujet optionnel – Protéger et valoriser la technologie

Le projet vise à combiner collaboration open source et protection des innovations technologiques développées.

L’outil CLI sera publié sous une licence open source telle que MIT ou Apache 2.0 afin de favoriser l’adoption par la communauté et garantir la transparence du code. Cette approche permettra aux développeurs d’auditer le projet, de proposer des améliorations et d’intégrer Cod3x dans leurs propres workflows.

En parallèle, l’équipe étudiera les bonnes pratiques pour protéger la propriété intellectuelle liée à l’architecture pivot et aux algorithmes de conversion. Des échanges avec des mentors et des experts techniques permettront d’identifier si certains éléments doivent être protégés par le droit d’auteur ou d’autres mécanismes.

L’objectif est de trouver un équilibre entre ouverture et protection afin d’assurer la pérennité du projet tout en encourageant la participation de la communauté.

### Sujet optionnel – Développer les contributions de la communauté

La création d’une communauté de développeurs sera un facteur important pour la croissance de Cod3x. Le projet prévoit d’encourager les contributions externes en rendant le cœur de l’outil open source et en fournissant une documentation claire.

Des guides de contribution seront mis en place afin d’aider les développeurs externes à créer de nouveaux parseurs de formats, générateurs ou améliorations du modèle pivot.

L’équipe prévoit également de mettre en place différentes initiatives pour encourager l’engagement de la communauté :

* ouverture d’issues publiques pour implémenter le support de nouveaux ORM
* organisation de challenges de développement pour créer de nouveaux convertisseurs
* démonstrations publiques du projet

Ces initiatives visent à transformer Cod3x en un écosystème collaboratif dans lequel la communauté participe activement à l’ajout de nouvelles intégrations et à l’amélioration continue du projet.

