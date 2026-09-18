# `src/domain` — ce qui reste côté JavaScript

Ce dossier contenait sept modules à l'issue de la phase 0. Il en reste deux.

| Module | Sort |
|---|---|
| `money.js` | → [`asgard-core::money`](../../crates/asgard-core/src/money.rs) (phase 3) |
| `urssaf.js` | → [`asgard-core::urssaf`](../../crates/asgard-core/src/urssaf.rs) (phase 3) |
| `thresholds.js` | → [`asgard-core::thresholds`](../../crates/asgard-core/src/thresholds.rs) (phase 3) |
| `reporting.js` | → [`asgard-core::reporting`](../../crates/asgard-core/src/reporting.rs) (phase 3) |
| `numbering.js` | → [`src-tauri/src/db/numbering.rs`](../../src-tauri/src/db/numbering.rs) (phase 2) |
| **`validation.js`** | **reste** — validation de formulaires, pas de comptabilité |
| **`templates.js`** | **reste** — gabarits d'e-mail, jusqu'à la phase 4 |

## Pourquoi ces deux-là restent

La validation de formulaire appartient à l'interface : elle doit répondre à la
frappe, sans aller-retour, et ses messages sont du texte d'interface. La
déplacer coûterait de la latence pour aucun gain de justesse.

Les gabarits d'e-mail suivront en phase 4, avec la génération de PDF et l'envoi.

## Où sont passés les golden tests

Ils étaient 151 à l'issue de la phase 0 ; il en reste 49 ici. Les 102 autres
portaient sur la comptabilité et **n'ont pas été supprimés : ils ont changé de
camp**, avec le code qu'ils éprouvaient.

Ce qui rend le transfert vérifiable, c'est que les deux côtés lisent **le même
fichier** : [`fixtures/reference-dataset.json`](../../fixtures/reference-dataset.json),
figé en phase 0. `asgard-core::fixtures` l'embarque à la compilation.

La phase 0 posait une règle : *« quand un de ces défauts sera corrigé, le test
correspondant devra être réécrit — pas simplement supprimé. C'est la trace de
la décision. »* C'est ce que fait
[`asgard-core::parity`](../../crates/asgard-core/src/parity.rs) : chaque valeur
figée par un golden test y est reprise, comparée à ce que Rust produit, et
classée en trois catégories — identique, artefact flottant supprimé, ou montant
réellement modifié par l'arrondi au centime.

## Commandes

```bash
npm test                      # validation et gabarits (49 tests)
cargo test -p asgard-core     # logique comptable (87 tests, proptests compris)
cargo test                    # tout, hôte compris
```
