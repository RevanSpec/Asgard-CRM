/**
 * Calculs monétaires.
 *
 * ⚠️ Défaut connu D5 — tous les montants sont des `f64` (Number JavaScript).
 * Les fonctions ci-dessous reproduisent *exactement* l'arithmétique actuelle,
 * arrondis flottants compris. Les tests associés figent ce comportement : ils
 * servent d'oracle pour la réimplémentation en `rust_decimal` (phase 3), qui
 * produira des écarts de quelques centimes qu'il faudra pouvoir justifier.
 *
 * Ne pas « corriger » les arrondis ici : ce serait modifier des chiffres déjà
 * déclarés sans rapport d'écarts préalable.
 */

/** Arrondi d'affichage, équivalent de `.toFixed(2)` utilisé dans les vues. */
export function round2(value) {
  return Number(Number(value).toFixed(2));
}

/**
 * Calcule TVA et total à partir d'un montant HT et d'un taux.
 * Reproduit `App.jsx` : `amountTva = (amountHt * tvaRate) / 100`.
 *
 * @param {number|string} amountHt
 * @param {number|string} tvaRate - en pourcentage (20, 5.5, 0…)
 * @returns {{amountHt: number, tvaRate: number, amountTva: number, amountTotal: number}}
 */
export function computeAmounts(amountHt, tvaRate) {
  const ht = parseFloat(amountHt);
  const rate = parseFloat(tvaRate);
  const amountTva = (ht * rate) / 100;
  return {
    amountHt: ht,
    tvaRate: rate,
    amountTva,
    amountTotal: ht + amountTva,
  };
}

/** Somme naïve, dans l'ordre du tableau — l'ordre compte en virgule flottante. */
export function sum(values) {
  return values.reduce((total, value) => total + value, 0);
}

/** Formatage monétaire français utilisé dans les vues (`toLocaleString('fr-FR')`). */
export function formatEuros(value) {
  return Number(value).toLocaleString('fr-FR');
}

/** Formatage décimal pour le CSV : virgule décimale, deux chiffres. */
export function formatCsvAmount(value) {
  return Number(value).toFixed(2).replace('.', ',');
}
