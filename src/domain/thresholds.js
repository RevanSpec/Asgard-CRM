/**
 * Seuils de franchise en base de TVA et plafonds du régime micro-entreprise.
 *
 * Les valeurs étaient jusqu'ici des littéraux dispersés dans le JSX
 * (`App.jsx` pour les alertes, `ComptaTab.jsx` pour les jauges), ce qui rendait
 * impossible leur mise à jour annuelle sans toucher au rendu — défaut D8.
 *
 * Elles sont désormais datées : un barème est associé à une année fiscale, et
 * `thresholdsForYear` retombe sur le barème connu le plus récent. C'est la
 * structure que la phase 3 portera telle quelle dans `asgard-core::thresholds`.
 */

/**
 * Barèmes par année fiscale.
 * `tvaLimit`    — seuil de la franchise en base
 * `tvaTolerance`— limite de tolérance au-delà de laquelle la TVA devient due
 * `microLimit`  — plafond du régime micro-entreprise
 */
export const FISCAL_THRESHOLDS = {
  2026: {
    service: { tvaLimit: 36800, tvaTolerance: 39100, microLimit: 77700 },
    vente: { tvaLimit: 91900, tvaTolerance: 101000, microLimit: 188700 },
  },
};

/** Seuils déclenchant une alerte, tels qu'écrits aujourd'hui dans `App.jsx`. */
export const ALERT_TRIGGERS = {
  2026: {
    service: { tvaWatch: 34000, microWatch: 70000 },
    vente: { tvaWatch: 85000, microWatch: 170000 },
  },
};

function mostRecentKnownYear(table, year) {
  if (table[year]) return year;
  const years = Object.keys(table)
    .map(Number)
    .filter((knownYear) => knownYear <= year)
    .sort((a, b) => b - a);
  return years.length > 0 ? years[0] : Math.min(...Object.keys(table).map(Number));
}

export function thresholdsForYear(year) {
  return FISCAL_THRESHOLDS[mostRecentKnownYear(FISCAL_THRESHOLDS, year)];
}

export function alertTriggersForYear(year) {
  return ALERT_TRIGGERS[mostRecentKnownYear(ALERT_TRIGGERS, year)];
}

/**
 * Chiffre d'affaires encaissé de l'année, ventilé entre services et ventes.
 * La date retenue est celle de l'encaissement, avec repli sur la date d'émission.
 * Tout ce qui n'est pas une vente de marchandises compte comme service —
 * comportement actuel, BNC et BIC confondus.
 */
export function annualCaByCategory(invoices, year) {
  let serviceCa = 0;
  let venteCa = 0;

  invoices.forEach((invoice) => {
    if (invoice.status !== 'payee') return;
    const paymentDate = new Date(invoice.paymentDate || invoice.date);
    if (paymentDate.getFullYear() !== year) return;

    if (invoice.serviceType === 'vente') {
      venteCa += invoice.amountHt;
    } else {
      serviceCa += invoice.amountHt;
    }
  });

  return { serviceCa, venteCa };
}

/**
 * Jauges de l'onglet « Seuils de Chiffre d'Affaires ».
 * Les pourcentages sont plafonnés à 100 — comportement actuel de `ComptaTab`.
 */
export function thresholdGauges(invoices, year) {
  const { serviceCa, venteCa } = annualCaByCategory(invoices, year);
  const limits = thresholdsForYear(year);

  const gauge = (ca, { tvaLimit, tvaTolerance, microLimit }) => ({
    ca,
    tvaLimit,
    tvaTolerance,
    microLimit,
    pctTva: Math.min((ca / tvaTolerance) * 100, 100),
    pctMicro: Math.min((ca / microLimit) * 100, 100),
    exceededTva: ca > tvaLimit,
    remainingBeforeTva: tvaTolerance - ca,
  });

  return {
    year,
    service: gauge(serviceCa, limits.service),
    vente: gauge(venteCa, limits.vente),
  };
}

/**
 * Alertes affichées en tête du tableau de bord.
 *
 * Reproduit à l'identique la cascade de `App.jsx`, y compris son ordre : alerte
 * TVA services, puis plafond services, puis TVA ventes, puis plafond ventes.
 * Les libellés sont repris mot pour mot, ils sont visibles par l'utilisateur.
 */
export function buildCaAlerts(invoices, year) {
  const { serviceCa, venteCa } = annualCaByCategory(invoices, year);
  const limits = thresholdsForYear(year);
  const triggers = alertTriggersForYear(year);
  const alerts = [];

  const fr = (value) => value.toLocaleString('fr-FR');

  if (serviceCa > triggers.service.tvaWatch) {
    if (serviceCa > limits.service.tvaTolerance) {
      alerts.push({
        type: 'danger',
        title: 'Seuil de TVA Services Dépassé',
        message: `Votre CA annuel de services (${fr(serviceCa)} €) a dépassé la limite de tolérance de la franchise en base de TVA (${fr(limits.service.tvaTolerance)} €). Vous devez facturer de la TVA.`,
      });
    } else {
      alerts.push({
        type: 'warning',
        title: 'Seuil de TVA Services Proche',
        message: `Votre CA annuel de services (${fr(serviceCa)} €) approche le seuil de la franchise en base de TVA (${fr(limits.service.tvaLimit)} € / limite de tolérance : ${fr(limits.service.tvaTolerance)} €).`,
      });
    }
  }

  if (serviceCa > triggers.service.microWatch) {
    alerts.push({
      type: 'warning',
      title: 'Plafond Micro-Entreprise Services Proche',
      message: `Votre CA annuel de services (${fr(serviceCa)} €) approche le plafond de la micro-entreprise (${fr(limits.service.microLimit)} €).`,
    });
  }

  if (venteCa > triggers.vente.tvaWatch) {
    if (venteCa > limits.vente.tvaTolerance) {
      alerts.push({
        type: 'danger',
        title: 'Seuil de TVA Ventes Dépassé',
        message: `Votre CA annuel de ventes (${fr(venteCa)} €) a dépassé la limite de tolérance de la franchise en base de TVA (${fr(limits.vente.tvaTolerance)} €). Vous devez facturer de la TVA.`,
      });
    } else {
      alerts.push({
        type: 'warning',
        title: 'Seuil de TVA Ventes Proche',
        message: `Votre CA annuel de ventes (${fr(venteCa)} €) approche le seuil de la franchise en base de TVA (${fr(limits.vente.tvaLimit)} € / limite de tolérance : ${fr(limits.vente.tvaTolerance)} €).`,
      });
    }
  }

  if (venteCa > triggers.vente.microWatch) {
    alerts.push({
      type: 'warning',
      title: 'Plafond Micro-Entreprise Ventes Proche',
      message: `Votre CA annuel de ventes (${fr(venteCa)} €) approche le plafond de la micro-entreprise (${fr(limits.vente.microLimit)} €).`,
    });
  }

  return alerts;
}
