# Transf-ORM — Spécification technique : routes API & interface CLI

Ce document rassemble et met au propre l'ensemble des décisions concernant l'architecture applicative de Transf-ORM : le référentiel des routes de l'API (authentification, teams, billing, projects, transformations, intégrations Git, historique) ainsi que la spécification du binaire en ligne de commande (modes CLI et serveur, flags globaux, commandes disponibles).

**Équipe :** Timeotr · AymericJM · tuturicide · Babouye · nitrached

## Sommaire

1. [Rôles & permissions](#1-rôles--permissions)
2. [Référentiel des routes API](#2-référentiel-des-routes-api)
   - 2.1 Authentification & compte
   - 2.2 Teams
   - 2.3 Billing
   - 2.4 Référentiel des ORM
   - 2.5 Projects
   - 2.6 Fichiers de schéma
   - 2.7 Transformation (asynchrone)
   - 2.8 Intégrations Git
   - 2.9 Historique & revert
3. [Spécification du binaire CLI](#3-spécification-du-binaire-cli)
   - 3.1 Flags globaux
   - 3.2 Mode CLI (par défaut)
   - 3.3 Commandes de la CLI
   - 3.4 Mode serveur

## 1. Rôles & permissions

Trois niveaux d'accès régissent l'ensemble des routes de l'API. Ils sont hiérarchiques et s'appliquent au périmètre d'une team.

| Rôle | Cardinalité | Périmètre |
|---|---|---|
| Owner | Unique par team. | Hérite de tous les droits Admin, plus la suppression de la team et le transfert d'ownership. Ne peut ni être supprimé ni rétrogradé sans transfert préalable. |
| Admin | Plusieurs par team. | Gère le billing, les invitations et la suppression de membres simples. Ne peut agir ni sur un autre admin, ni sur le owner. |
| Membre | — | Accès aux projects, aux transformations, à l'historique et au revert, dans la limite du quota de la team. |

## 2. Référentiel des routes API

Le tableau ci-dessous liste, pour chaque domaine fonctionnel, la méthode HTTP, la route, sa description et le niveau d'accès requis.

### 2.1 Authentification & compte

| Méthode | Route | Description | Accès |
|---|---|---|---|
| POST | `/auth/register` | Création de compte. | Public |
| POST | `/auth/login` | Connexion. | Public |
| POST | `/auth/logout` | Déconnexion. | Authentifié |
| POST | `/auth/refresh` | Rafraîchissement du token. | Authentifié |
| GET | `/me` | Profil de l'utilisateur connecté. | Authentifié |
| PATCH | `/me` | Mise à jour du profil. | Authentifié |

### 2.2 Teams

| Méthode | Route | Description | Accès |
|---|---|---|---|
| POST | `/teams` | Créer une team (le créateur en devient owner). | Authentifié |
| GET | `/teams/{id}` | Détails d'une team. | Membre |
| PATCH | `/teams/{id}` | Modifier les informations générales. | Admin+ |
| DELETE | `/teams/{id}` | Supprimer la team. | Owner |
| POST | `/teams/{id}/transfer-ownership` | Transférer l'ownership. Corps : `{ newOwnerId }`. | Owner |
| GET | `/teams/{id}/members` | Liste des membres. | Membre |
| POST | `/teams/{id}/members/invite` | Inviter un membre. Corps : `{ email, role }`. | Admin+ |
| DELETE | `/teams/{id}/members/{userId}` | Supprimer un membre (le owner ne peut pas être visé). | Admin+ |
| PATCH | `/teams/{id}/members/{userId}` | Changer le rôle d'un membre. Cible membre : Admin+. Cible admin : Owner uniquement. | Admin+ / Owner |

### 2.3 Billing

| Méthode | Route | Description | Accès |
|---|---|---|---|
| GET | `/teams/{id}/billing/plan` | Plan actuel de la team. | Admin+ |
| POST | `/teams/{id}/billing/checkout` | Souscrire ou changer de plan. Corps : `{ planId }`. | Admin+ |
| GET | `/teams/{id}/billing/usage` | Usage courant : nombre de projects, de membres et de transformations mensuelles. | Admin+ |

> Modèle de facturation basé sur des plans fixes, au niveau de la team.

### 2.4 Référentiel des ORM

| Méthode | Route | Description | Accès |
|---|---|---|---|
| GET | `/orms` | Liste des ORM supportés par la plateforme. | Public / Authentifié |

### 2.5 Projects

| Méthode | Route | Description | Accès |
|---|---|---|---|
| GET | `/teams/{id}/projects` | Projects où l'utilisateur est collaborateur. | Membre |
| POST | `/teams/{id}/projects` | Créer un project (le créateur devient collaborateur automatiquement). Refus 402/403 si le quota de la team est atteint. | Membre |
| GET | `/projects/{id}` | Détails d'un project. | Collaborateur |
| PATCH | `/projects/{id}` | Modifier un project. | Collaborateur |
| DELETE | `/projects/{id}` | Supprimer un project. | Admin+ |
| GET | `/projects/{id}/collaborators` | Liste des collaborateurs du project. | Collaborateur |
| POST | `/projects/{id}/collaborators` | Ajouter un collaborateur. Corps : `{ userId }` (doit être membre de la team). | Collaborateur |
| DELETE | `/projects/{id}/collaborators/{userId}` | Retirer un collaborateur. | Collaborateur |

> Visibilité par collaborateurs (à la manière de GitHub) : un project n'est visible et modifiable que par les membres explicitement ajoutés, et non par toute la team.

### 2.6 Fichiers de schéma

| Méthode | Route | Description | Accès |
|---|---|---|---|
| POST | `/projects/{id}/files` | Upload manuel de fichier(s) en multipart, ou import automatique depuis le repo lié via `{ source: "repo" }`. Déclenche la détection de l'ORM. | Collaborateur |
| GET | `/projects/{id}/files` | Liste des fichiers suivis. | Collaborateur |
| GET | `/projects/{id}/files/{fileId}` | Contenu et métadonnées d'un fichier. | Collaborateur |
| DELETE | `/projects/{id}/files/{fileId}` | Supprimer un fichier. | Collaborateur |
| GET | `/projects/{id}/detected-orm` | Résultat de la détection automatique de l'ORM. | Collaborateur |

> Les fichiers de schéma sont stockés au niveau du project.

### 2.7 Transformation (asynchrone)

| Méthode | Route | Description | Accès |
|---|---|---|---|
| POST | `/projects/{id}/transform` | Lancer une transformation. Corps : `{ from?, to, fileIds[], description? }`. Répond `202` avec un `job_id`. | Collaborateur |
| GET | `/jobs/{job_id}` | Statut du job (`pending` / `running` / `partial` / `done` / `failed`) et résultat ou suggestions. | Collaborateur |
| GET | `/jobs/{job_id}/stream` | Progression en temps réel (SSE / WebSocket). | Collaborateur |
| DELETE | `/jobs/{job_id}` | Annuler un job en cours. | Collaborateur |

### 2.8 Intégrations Git (GitHub, GitLab, etc.)

| Méthode | Route | Description | Accès |
|---|---|---|---|
| GET | `/teams/{id}/integrations` | Intégrations Git connectées à la team. | Admin+ |
| POST | `/teams/{id}/integrations/{provider}/connect` | Démarre le flow OAuth (provider = github / gitlab / …). | Admin+ |
| GET | `/teams/{id}/integrations/{provider}/callback` | Callback OAuth (redirection par le provider). | Public, validé par state/token |
| DELETE | `/teams/{id}/integrations/{provider}` | Déconnecter l'intégration. | Admin+ |
| GET | `/teams/{id}/integrations/{provider}/repos` | Repos accessibles via le token connecté. | Admin+ |
| POST | `/projects/{id}/repo/link` | Lier un repo à un project. Corps : `{ provider, repoId, branch, autoImport? }`. Si `autoImport: true`, importe automatiquement les fichiers de schéma détectés. | Collaborateur |
| DELETE | `/projects/{id}/repo/link` | Délier le repo du project. | Collaborateur |
| POST | `/projects/{id}/repo/push` | Ouvre une Pull Request avec le schéma transformé. Corps : `{ fileIds[], commitMessage, branch?, prTitle?, prDescription? }`. Toujours via PR, jamais de push direct. | Collaborateur |
| GET | `/projects/{id}/repo/status` | Statut du repo lié (branche, dernière PR ouverte ou mergée). | Collaborateur |

> La connexion OAuth se fait au niveau de la team : le token appartient à la team et non à un membre individuel, ce qui évite toute perte d'accès en cas de départ. Le repo est ensuite lié à un project spécifique. Aucun point ouvert restant sur cette section.

### 2.9 Historique & revert

| Méthode | Route | Description | Accès |
|---|---|---|---|
| GET | `/projects/{id}/history` | Liste des fichiers modifiés. | Collaborateur |
| GET | `/projects/{id}/history/{fileId}/versions` | Versions disponibles pour un fichier. | Collaborateur |
| GET | `/projects/{id}/history/{fileId}/diff?version=` | Diff façon git / VS Code entre deux versions. | Collaborateur |
| POST | `/projects/{id}/revert` | Revert. Corps : `{ files: [{ fileId, version }] }`. | Collaborateur |

> Historique et revert sont scopés au project.

### Points ouverts

Aucun point ouvert restant — toutes les décisions de permissions et de flux ont été tranchées.

## 3. Spécification du binaire CLI

Le binaire Transf-ORM propose deux modes d'exécution : un mode CLI interactif (comportement par défaut) et un mode serveur. Un ensemble de flags globaux s'applique indépendamment du mode choisi.

### 3.1 Flags globaux

| Flag | Description |
|---|---|
| `--cli` | Lance le binaire en mode CLI interactif. Comportement par défaut si aucun mode n'est précisé. |
| `--serveur` | Lance le binaire en mode serveur (API), voir §3.4. |
| `--debug <niveau>` | Active les logs de debug, avec plusieurs niveaux de verbosité disponibles. |
| `--verbose` | Augmente la verbosité de la sortie standard. |
| `--perf` | Active le suivi et l'affichage de métriques de performance. |
| `--tts` | Active la synthèse vocale (text-to-speech) pour les retours de la CLI. |
| `--daltonien` | Active un mode d'affichage adapté au daltonisme (palette de contraste alternative). |
| `-h` / `--help` | Affiche l'aide et la liste des commandes disponibles. |
| `--version` | Affiche la version du binaire. |

### 3.2 Mode CLI (par défaut)

La CLI s'exécute toujours dans le répertoire courant. Au lancement, elle affiche le nom de l'ORM détecté automatiquement dans ce répertoire. Si aucun ORM n'est détecté, l'utilisateur doit le préciser explicitement via l'option `--from` lors de l'appel à la commande `transform`.

### 3.3 Commandes de la CLI

#### `transform`

```
transform [--from <orm>] --to <orm> [--description <texte>]
```

- `--from` est optionnel si l'ORM source a été détecté automatiquement, obligatoire sinon.
- `--to` est obligatoire et désigne l'ORM cible.
- `--description` est optionnel.
- Une fois lancée, la commande ouvre un sélecteur de fichiers interactif (façon lazygit) permettant de choisir, parmi les fichiers de schéma détectés, ceux à traduire.
- Lorsqu'aucune traduction exacte n'existe pour un élément du schéma, la CLI propose des solutions alternatives si elles existent, ou explique pourquoi la traduction n'est pas possible.

#### `revert`

```
revert
```

- Ouvre un sélecteur permettant de choisir les fichiers à revert, puis la version cible pour chacun.
- Une sélection globale (« tout sélectionner ») est possible, avec ensuite un choix de version fichier par fichier.

#### `history`

```
history
```

- Affiche la liste des fichiers modifiés.
- Permet d'en sélectionner un et d'en visualiser les diffs dans un rendu façon git diff / VS Code.

### 3.4 Mode serveur

Activé via le flag `--serveur`, ce mode expose l'API et accepte les flags de configuration réseau suivants :

| Flag | Description |
|---|---|
| `--ip <adresse>` | Adresse IP sur laquelle le serveur écoute. |
| `--port <port>` | Port d'écoute du serveur. |
| `--host <host>` | Nom d'hôte associé au serveur. |
