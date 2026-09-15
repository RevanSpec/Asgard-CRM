/**
 * Cotisations sociales URSSAF du micro-entrepreneur.
 *
 * Ce module remplace les trois implémentations indépendantes qui coexistaient
 * dans les vues (défaut D8) :
 *   - App.jsx        · calculateUrssafCharges
 *   - App.jsx        · getMonthlyFinancialsData
 *   - ComptaTab.jsx  · IIFE de l'onglet « Déclaration URSSAF »
 *
 * ⚠️ Incohérence connue, volontairement préservée : le tableau de bord mensuel
 * calcule les charges sur les factures « non brouillon » (donc facturées, même
 * impayées), alors que le total annuel et la déclaration URSSAF ne les
 * calculent que sur les factures « payée ». Voir `chargesForIssuedInvoices`
 * contre `totalUrssafCharges`. C'est un défaut à trancher en phase 3 — l'URSSAF
 * se déclare sur l'encaissé.
 */

export const SERVICE_TYPES = {
  SERVICE_BNC: 'service_bnc',
  SERVICE_BIC: 'service_bic',
  VENTE: 'vente',
};

/** Taux par défaut de l'application (réglages métier, modifiables par l'utilisateur). */
export const DEFAULT_URSSAF_RATES = {
  urssafServiceBnc: 21.1,
  urssafServiceBic: 21.1,
  urssafVente: 12.3,
};

/** Abattement ACRE : les taux sont divisés par deux la première année. */
export const ACRE_DIVISOR = 2;

/**
 * Taux brut applicable à un type d'activité, avant ACRE.
 * Un type inconnu renvoie 0 — comportement actuel des trois implémentations.
 */
export function baseRateFor(serviceType, settings) {
  if (serviceType === SERVICE_TYPES.SERVICE_BNC) return settings.urssafServiceBnc;
  if (serviceType === SERVICE_TYPES.SERVICE_BIC) return settings.urssafServiceBic;
  if (serviceType === SERVICE_TYPES.VENTE) return settings.urssafVente;
  return 0;
}

/** Taux effectivement appliqué, ACRE comprise. */
export function effectiveRateFor(serviceType, settings) {
  const rate = baseRateFor(serviceType, settings);
  return settings.acreEnabled ? rate / ACRE_DIVISOR : rate;
}

/** Cotisations dues sur une facture, calculées sur le montant HT. */
export function chargesForInvoice(invoice, settings) {
  const rate = effectiveRateFor(invoice.serviceType, settings);
  return (invoice.amountHt * rate) / 100;
}

/**
 * Total des cotisations sur les seules factures encaissées.
 * Équivalent de `App.jsx · calculateUrssafCharges`.
 */
export function totalUrssafCharges(invoices, settings) {
  return invoices
    .filter((invoice) => invoice.status === 'payee')
    .reduce((total, invoice) => total + chargesForInvoice(invoice, settings), 0);
}

/**
 * Total des cotisations sur les factures émises, payées ou non.
 * Équivalent du calcul interne à `App.jsx · getMonthlyFinancialsData`.
 * Conservé distinct pour rendre l'incohérence visible plutôt que de la masquer.
 */
export function chargesForIssuedInvoices(invoices, settings) {
  return invoices
    .filter((invoice) => invoice.status !== 'brouillon')
    .reduce((total, invoice) => total + chargesForInvoice(invoice, settings), 0);
}

/** Trimestre (1-4) d'un mois exprimé de 1 à 12. */
export function quarterOfMonth(month) {
  return Math.floor((month - 1) / 3) + 1;
}

/**
 * Une facture encaissée tombe-t-elle dans la période de déclaration ?
 * La date retenue est la date d'encaissement, avec repli sur la date d'émission.
 */
export function invoiceMatchesPeriod(invoice, { periodType, year, month, quarter }) {
  if (invoice.status !== 'payee') return false;

  const paymentDate = new Date(invoice.paymentDate || invoice.date);
  if (paymentDate.getFullYear() !== year) return false;

  const invoiceMonth = paymentDate.getMonth() + 1;
  if (periodType === 'monthly') return invoiceMonth === month;
  return quarterOfMonth(invoiceMonth) === quarter;
}

/**
 * Déclaration URSSAF d'une période : chiffre d'affaires encaissé et cotisations,
 * ventilés par type d'activité.
 *
 * @param {Array} invoices
 * @param {object} settings
 * @param {{periodType: 'monthly'|'quarterly', year: number, month?: number, quarter?: number}} period
 */
export function urssafDeclaration(invoices, settings, period) {
  const selected = invoices.filter((invoice) => invoiceMatchesPeriod(invoice, period));

  const htByType = {
    [SERVICE_TYPES.SERVICE_BNC]: 0,
    [SERVICE_TYPES.SERVICE_BIC]: 0,
    [SERVICE_TYPES.VENTE]: 0,
  };

  selected.forEach((invoice) => {
    if (htByType[invoice.serviceType] !== undefined) {
      htByType[invoice.serviceType] += invoice.amountHt;
    }
  });

  const lineFor = (serviceType) => {
    const ht = htByType[serviceType];
    const rate = effectiveRateFor(serviceType, settings);
    return { ht, rate, charges: (ht * rate) / 100 };
  };

  const bnc = lineFor(SERVICE_TYPES.SERVICE_BNC);
  const bic = lineFor(SERVICE_TYPES.SERVICE_BIC);
  const vente = lineFor(SERVICE_TYPES.VENTE);

  return {
    bnc,
    bic,
    vente,
    totalCa: bnc.ht + bic.ht + vente.ht,
    totalCharges: bnc.charges + bic.charges + vente.charges,
  };
}
