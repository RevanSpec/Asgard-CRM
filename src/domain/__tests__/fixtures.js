import dataset from '../../../fixtures/reference-dataset.json';

export const {
  clients,
  invoices,
  estimates,
  expenses,
  settings,
  thresholdScenarios,
  referenceYear,
} = dataset;

export const standardSettings = settings.standard;
export const acreSettings = settings.acre;

/**
 * Normalise les espaces avant comparaison.
 *
 * `toLocaleString('fr-FR')` sépare les milliers par une espace insécable dont
 * la nature — U+00A0 ou U+202F — dépend de la version d'ICU. Comparer les
 * libellés à l'octet près rendrait les tests dépendants de la version de Node.
 */
export function normalizeSpaces(text) {
  return text.replace(/\s+/g, ' ');
}
