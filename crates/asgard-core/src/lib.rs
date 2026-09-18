//! Logique métier d'Asgard CRM.
//!
//! Ce crate ne connaît ni Tauri, ni SQLite, ni l'interface : il prend des
//! données et rend des données. C'est ce qui le rend testable sans monter quoi
//! que ce soit, et c'est la raison d'être de la phase 3 du plan de migration.
//!
//! ## Ce que la phase 3 corrige
//!
//! **D5 — les calculs cessent d'être flottants.** La phase 2 stockait déjà des
//! centimes ; ici `rust_decimal` remplace `f64` dans les calculs eux-mêmes.
//! L'invariant « HT + TVA = TTC » devient exact, et un test de propriété le
//! vérifie sur des milliers de cas plutôt que sur trois exemples.
//!
//! **D8 — le calcul URSSAF n'existe plus qu'en un exemplaire.** Il en existait
//! trois, qui divergeaient : deux d'entre eux comptaient les factures émises,
//! le troisième les factures encaissées. L'écart atteignait 1 002,25 € sur le
//! jeu de référence, et l'utilisateur voyait les deux chiffres côte à côte. La
//! divergence est tranchée dans [`urssaf::Basis`] — l'URSSAF se déclare sur
//! l'encaissé — et l'autre base subsiste, nommée pour ce qu'elle est : une
//! projection de trésorerie.
//!
//! **Les barèmes légaux sortent du JSX.** Seuils de TVA et plafonds micro
//! étaient des littéraux dans le rendu ; ils sont désormais datés par exercice
//! dans [`thresholds`].
//!
//! ## Comment la parité est démontrée
//!
//! [`fixtures`] lit `fixtures/reference-dataset.json` — **le même fichier** que
//! les golden tests JavaScript de la phase 0. Les deux implémentations sont
//! donc éprouvées sur des données identiques, et non sur deux jeux qu'on
//! supposerait équivalents. Le module `parity` documente chaque écart entre
//! l'ancien résultat flottant et le nouveau : c'est le rapport que le plan
//! exigeait avant toute bascule.

pub mod model;
pub mod money;
pub mod reporting;
pub mod thresholds;
pub mod urssaf;

#[cfg(test)]
pub mod fixtures;

#[cfg(test)]
mod parity;

pub use model::{CivilDate, Expense, Invoice, ServiceType, Settings, Status};
pub use money::{compute_amounts, from_cents, from_f64, round_cents, to_cents, to_f64, Money, Rate};
