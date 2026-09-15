# `src/domain` — logique métier extraite

Phase 0 du [plan de migration vers Rust](../../docs/MIGRATION_RUST.md).

Ces modules sont des fonctions **pures** : pas de React, pas de Dexie, pas de
DOM, aucun accès à `localStorage`. Ils prennent des données en argument et
renvoient des données. C'est ce qui les rend testables, et c'est ce qui sera
porté tel quel dans le crate `asgard-core` en phase 3.

## Pourquoi cette étape existe

Avant la phase 0, la logique comptable vivait dans `App.jsx` (2 060 lignes) et
dans des IIFE au milieu du JSX de `ComptaTab.jsx`. Elle était donc :

- **intestable** — il fallait monter un composant React pour vérifier un taux ;
- **dupliquée** — le calcul des cotisations URSSAF existait en trois exemplaires ;
- **non spécifiée** — aucun test ne disait ce que l'application est censée calculer.

Sans oracle, réécrire en Rust serait une réécriture à l'aveugle : rien ne
permettrait de prouver que le nouveau code calcule la même chose que l'ancien.
Les tests de ce dossier *sont* cet oracle.

## Modules

| Module | Rôle | Cible Rust |
|---|---|---|
| `money.js` | TVA, totaux, arrondis, formats d'affichage | `asgard-core::money` |
| `urssaf.js` | Taux par activité, ACRE, déclaration par période | `asgard-core::urssaf` |
| `thresholds.js` | Seuils TVA et plafonds micro, alertes, jauges | `asgard-core::thresholds` |
| `numbering.js` | Numérotation des factures et devis | `asgard-core::numbering` |
| `reporting.js` | Agrégats du tableau de bord, livre des recettes, CSV | `asgard-core::reporting` |
| `validation.js` | Validation des formulaires | `asgard-core::validation` |
| `templates.js` | Gabarits d'e-mail et substitution de jetons | `asgard-mail::templates` |

## Règle de la phase 0 : figer, pas corriger

Les tests décrivent le comportement **réellement en production aujourd'hui**,
défauts compris. Plusieurs d'entre eux assertent explicitement un comportement
incorrect, avec un commentaire qui le signale :

| Défaut | Où | Ce que le test prouve |
|---|---|---|
| **D3** | `numbering.test.js` | Un numéro de facture est réattribué après suppression, et deux créations concurrentes produisent le même |
| **D5** | `money.test.js` | `1899,99 € × 20 %` vaut `379.99800000000005`, pas `379.998` |
| **D8** | `urssaf.test.js` | Les cotisations du graphe mensuel et celles du total annuel divergent de 1 002,25 € sur le jeu de référence |
| — | `numbering.test.js` | Les bornes d'année sont calculées en UTC alors que l'année est lue en heure locale |
| — | `validation.test.js` | `parseFloat('1899,99')` vaut `1899` : une virgule décimale tronque les centimes en silence |
| — | `validation.test.js` | La validation d'adresse refuse « 12 bis rue de Paris » et « 5 cours Mirabeau » |

Corriger ces comportements maintenant ferait bouger des chiffres déjà déclarés
sans rapport d'écarts, et priverait les phases suivantes de leur point de
comparaison. Chaque correction a une phase assignée dans le plan.

**Quand un de ces défauts sera corrigé, le test correspondant devra être
réécrit — pas simplement supprimé.** C'est la trace de la décision.

## Jeu de référence

`fixtures/reference-dataset.json` couvre :

- les trois types d'activité (BNC, BIC services, vente de marchandises) ;
- tous les statuts de facture et de devis ;
- ACRE activée et désactivée (deux jeux de réglages) ;
- une facture émise en 2025 et encaissée en 2026 ;
- une facture émise au T2 et encaissée au T3 ;
- une catégorie de dépense inconnue, qui doit basculer dans « Autre » ;
- des montants non ronds qui exposent les artefacts de virgule flottante ;
- un taux de TVA à 0, qui déclenche la mention 293 B du CGI sur le PDF.

Les numéros de pièce du jeu sont vérifiés contre le générateur réel par un test
dédié : sans cela, les fixtures ne représenteraient pas des données que
l'application a pu produire.

`fixtures/pdf-reference/` contient les 22 PDF produits par jsPDF sur ce jeu.
Leur génération est **reproductible** — date d'édition injectée, `CreationDate`
figée, identifiant de document normalisé — de sorte que `npm run pdf:reference`
ne salit pas le dépôt. Ce sont eux que la phase 4 devra reproduire.

Une comparaison octet à octet entre jsPDF et une implémentation Rust n'aura
aucun sens : la comparaison portera sur le texte extrait et sur le rendu visuel.

## Commandes

```bash
npm test              # rejoue les golden tests
npm run test:watch    # en continu pendant le développement
npm run pdf:reference # régénère les PDF de référence
```
