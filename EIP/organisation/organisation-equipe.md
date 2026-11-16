# Organisation de l'équipe — Transf-ORM

EIP Tech 4 — réunion du 10 août 2026

## 1. Réunion hebdomadaire

**Durée :** 15 à 30 minutes (+ ~30 minutes supplémentaires si un point avec le mentor ou le référent pédagogique a lieu avant la prochaine réunion hebdomadaire).

**Objectif :** partager l'avancée de la semaine, les problèmes rencontrés, etc.

**Format**

- Tour de table des avancées de la semaine — maximum 4 minutes par personne *(responsable : tuturicide)*
- Vérifier que tout va bien, et que les tâches attribuées plaisent à chacun
- En fin de réunion : mention des événements importants à venir, puis questions

## 2. Réunion mensuelle

**Durée :** 1 h à 2 h — réunion à ne pas rater.

**Objectif :** faire le bilan du mois passé, présentation interne, définir les nouveaux objectifs.

**Format**

- Début de réunion identique à la réunion hebdomadaire (tour de table rapide)
- Présentation globale de ce qu'il s'est passé dans le mois — roulement des personnes qui présentent d'un mois sur l'autre (pas systématiquement la même personne) :
  - PR faites sur `main`
  - FU / RDV mentor
  - Avancées notables sur le projet
  - Retards notables
  - Événements tech notables
- Moment de discussion par rapport à la présentation précédente
- Discussion sur les objectifs atteints et les retards éventuels — comprendre les causes des retards s'il y en a *(responsable : nitrached)*
- Création de nouvelles tâches (issues) et assignation *(responsable : nitrached)*

## 3. Gestion des tâches et du code

### Workflow

- Les tâches sont présentées sous forme d'issue (GitHub / GitHub Projects)
- Une branche = une issue
- PR de la branche vers `dev`
- Review obligatoire avant merge :
  - vers `dev` : review par une personne autre que l'auteur de la PR ;
  - vers `main` : review faite par nitrached.
- Processus de création d'une issue : voir la documentation sur la contribution ([`CONTRIBUTING.md`](../../CONTRIBUTING.md))

### Critères de review

- Vérifier que la PR respecte tous les points mentionnés dans le template de PR — si la description indique que tel ou tel élément a été modifié, le vérifier réellement plutôt que de se fier à la déclaration
- Vérifier la qualité du code produit (fonctions documentées avec rustdoc, code lisible)
- Vérifier que la branche règle une seule issue, pas plusieurs à la fois
- Code testé, et documenté si besoin
- Vérifier sur la branche créée par la PR que tout fonctionne, en suivant le guide de vérification spécifié dans l'issue / la PR

## 4. Répartition du code

| Domaine | Responsables |
|---|---|
| Frontend | AymericJM, Babouye |
| Backend (serveur) | tuturicide, Timeotr, nitrached |
| Core (logique métier) | tuturicide, Timeotr, nitrached |
| CLI | AymericJM, Babouye |
| CI/CD | Timeotr |
| VPS | nitrached |
| Release | Timeotr |

Cette répartition est une première base — elle sera affinée au fur et à mesure de l'avancement du projet, en fonction des besoins réels et de la charge de chacun.

## 5. Suivi de l'avancement

Le suivi de l'avancement se fait via GitHub Projects, en lien direct avec les issues du dépôt : chaque tâche créée en réunion mensuelle (ou en cours de sprint) y est ajoutée, assignée, et son état est mis à jour au fil de l'eau (à faire / en cours / en review / terminé). Ce tableau sert de référence commune pour visualiser d'un coup d'œil où en est chaque chantier, et alimente directement le tour de table de la réunion hebdomadaire ainsi que le bilan de la réunion mensuelle.
