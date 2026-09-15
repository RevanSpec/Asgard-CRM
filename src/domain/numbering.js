/**
 * Numérotation des pièces : factures (`FAC-`) et devis (`DEV-`).
 *
 * ⚠️ Défaut connu D3, volontairement reproduit ici.
 * La séquence est dérivée du *nombre* de pièces existantes de l'année
 * (`count + 1`). Deux conséquences, toutes deux illustrées par les tests :
 *   - supprimer une pièce fait réutiliser un numéro déjà émis ;
 *   - deux créations concurrentes produisent le même numéro.
 *
 * L'article 242 nonies A du CGI impose une séquence chronologique continue et
 * sans rupture. La correction relève de la phase 2 (table `document_sequences`
 * incrémentée dans la transaction d'insertion) : ne pas la porter ici, sans quoi
 * les golden tests ne figeraient plus le comportement réellement en production.
 */

export const DOCUMENT_PREFIXES = {
  invoice: 'FAC',
  estimate: 'DEV',
};

/** Longueur maximale du fragment client dans un numéro de pièce. */
export const CLIENT_FRAGMENT_LENGTH = 10;

/** Nombre de chiffres de la séquence. */
export const SEQUENCE_PADDING = 4;

/**
 * Normalise un nom d'entreprise pour l'insérer dans un numéro de pièce :
 * majuscules, accents retirés, caractères non alphanumériques supprimés,
 * tronqué à dix caractères.
 */
export function sanitizeClientName(companyName) {
  return companyName
    .toUpperCase()
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[^A-Z0-9]/g, '')
    .substring(0, CLIENT_FRAGMENT_LENGTH);
}

/**
 * Bornes ISO de l'année civile d'une date, telles qu'utilisées par la requête
 * Dexie `where('date').between(start, end, true, true)`.
 *
 * ⚠️ Les bornes sont construites en UTC alors que l'année est lue dans le
 * fuseau local : une facture du 31 décembre au soir peut donc sortir de la
 * fenêtre. Comportement actuel, figé par les tests.
 */
export function yearBounds(dateStr) {
  const year = new Date(dateStr).getFullYear().toString();
  return {
    year,
    start: new Date(`${year}-01-01T00:00:00Z`).toISOString(),
    end: new Date(`${year}-12-31T23:59:59Z`).toISOString(),
  };
}

/** Assemble un numéro de pièce à partir de ses quatre fragments. */
export function formatDocumentNumber(prefix, companyName, year, sequence) {
  const fragment = sanitizeClientName(companyName);
  const padded = sequence.toString().padStart(SEQUENCE_PADDING, '0');
  return `${prefix}-${fragment}-${year}-${padded}`;
}

/**
 * Séquence dérivée du nombre de pièces existantes — l'implémentation actuelle.
 * @see Défaut D3 ci-dessus.
 */
export function sequenceFromCount(existingCount) {
  return existingCount + 1;
}

/**
 * Construit un numéro de pièce.
 *
 * @param {'invoice'|'estimate'} kind
 * @param {string} companyName
 * @param {string} dateStr - date ISO de la pièce
 * @param {number} existingCount - pièces déjà enregistrées sur l'année
 */
export function buildDocumentNumber(kind, companyName, dateStr, existingCount) {
  const { year } = yearBounds(dateStr);
  return formatDocumentNumber(
    DOCUMENT_PREFIXES[kind],
    companyName,
    year,
    sequenceFromCount(existingCount),
  );
}
