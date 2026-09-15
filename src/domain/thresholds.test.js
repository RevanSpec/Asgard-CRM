import { describe, it, expect } from 'vitest';
import {
  thresholdsForYear,
  alertTriggersForYear,
  annualCaByCategory,
  thresholdGauges,
  buildCaAlerts,
  FISCAL_THRESHOLDS,
} from './thresholds';
import { invoices, thresholdScenarios, normalizeSpaces } from './__tests__/fixtures';

describe('thresholdsForYear', () => {
  it('restitue le barème 2026', () => {
    expect(thresholdsForYear(2026)).toEqual({
      service: { tvaLimit: 36800, tvaTolerance: 39100, microLimit: 77700 },
      vente: { tvaLimit: 91900, tvaTolerance: 101000, microLimit: 188700 },
    });
  });

  it('retombe sur le barème connu le plus récent pour une année postérieure', () => {
    expect(thresholdsForYear(2030)).toBe(FISCAL_THRESHOLDS[2026]);
  });

  it('retombe sur le barème le plus ancien pour une année antérieure', () => {
    expect(thresholdsForYear(2019)).toBe(FISCAL_THRESHOLDS[2026]);
  });

  it('expose les mêmes années que la table des déclencheurs d’alerte', () => {
    expect(Object.keys(alertTriggersForYear(2026))).toEqual(['service', 'vente']);
  });
});

describe('annualCaByCategory', () => {
  it('ventile le CA encaissé de 2026', () => {
    expect(annualCaByCategory(invoices, 2026)).toEqual({
      serviceCa: 14999.99,
      venteCa: 4650.35,
    });
  });

  it('range BNC et BIC ensemble, et seules les ventes à part', () => {
    const { serviceCa } = annualCaByCategory(invoices, 2026);
    // 5000 (BNC, encaissé en janvier) + 8100 (BNC) + 1899,99 (BIC)
    expect(serviceCa).toBe(14999.99);
  });

  it('rattache une facture à l’année de son encaissement', () => {
    // Facture 1 : émise le 15/12/2025, encaissée le 08/01/2026.
    expect(annualCaByCategory(invoices, 2025)).toEqual({ serviceCa: 0, venteCa: 0 });
  });

  it('ignore les factures non encaissées', () => {
    const onlyUnpaid = invoices.filter((invoice) => invoice.status !== 'payee');
    expect(annualCaByCategory(onlyUnpaid, 2026)).toEqual({ serviceCa: 0, venteCa: 0 });
  });
});

describe('thresholdGauges', () => {
  const gauges = thresholdGauges(invoices, 2026);

  it('rapporte le CA de services à la tolérance de TVA', () => {
    expect(gauges.service.ca).toBe(14999.99);
    expect(gauges.service.tvaTolerance).toBe(39100);
    expect(gauges.service.pctTva).toBeCloseTo(38.3631, 4);
    expect(gauges.service.exceededTva).toBe(false);
    expect(gauges.service.remainingBeforeTva).toBeCloseTo(24100.01, 2);
  });

  it('rapporte le CA de ventes à son propre plafond', () => {
    expect(gauges.vente.ca).toBe(4650.35);
    expect(gauges.vente.pctMicro).toBeCloseTo(2.4644, 4);
  });

  it('plafonne les pourcentages à cent', () => {
    const exceeded = thresholdGauges(thresholdScenarios.venteApproachingMicro, 2026);

    expect(exceeded.vente.ca).toBe(175000);
    expect(exceeded.vente.pctTva).toBe(100);
    expect(exceeded.vente.pctMicro).toBeCloseTo(92.7398, 4);
  });

  it('signale le dépassement du seuil de franchise', () => {
    const exceeded = thresholdGauges(thresholdScenarios.serviceExceededTva, 2026);

    expect(exceeded.service.exceededTva).toBe(true);
    expect(exceeded.service.remainingBeforeTva).toBeLessThan(0);
  });
});

describe('buildCaAlerts', () => {
  it('reste silencieux tant qu’aucun déclencheur n’est atteint', () => {
    expect(buildCaAlerts(invoices, 2026)).toEqual([]);
  });

  it('avertit à l’approche du seuil de TVA services', () => {
    const alerts = buildCaAlerts(thresholdScenarios.serviceApproachingTva, 2026);

    expect(alerts).toHaveLength(1);
    expect(alerts[0].type).toBe('warning');
    expect(alerts[0].title).toBe('Seuil de TVA Services Proche');
    expect(normalizeSpaces(alerts[0].message)).toContain('35 000 €');
    expect(normalizeSpaces(alerts[0].message)).toContain('36 800 €');
    expect(normalizeSpaces(alerts[0].message)).toContain('39 100 €');
  });

  it('passe en danger au-delà de la tolérance services', () => {
    const alerts = buildCaAlerts(thresholdScenarios.serviceExceededTva, 2026);

    expect(alerts).toHaveLength(1);
    expect(alerts[0].type).toBe('danger');
    expect(alerts[0].title).toBe('Seuil de TVA Services Dépassé');
    expect(alerts[0].message).toContain('Vous devez facturer de la TVA.');
  });

  it('cumule alerte TVA et alerte de plafond quand les deux sont franchis', () => {
    const alerts = buildCaAlerts(thresholdScenarios.serviceApproachingMicro, 2026);

    expect(alerts.map((alert) => alert.title)).toEqual([
      'Seuil de TVA Services Dépassé',
      'Plafond Micro-Entreprise Services Proche',
    ]);
    expect(normalizeSpaces(alerts[1].message)).toContain('77 700 €');
  });

  it('avertit à l’approche du seuil de TVA ventes', () => {
    const alerts = buildCaAlerts(thresholdScenarios.venteApproachingTva, 2026);

    expect(alerts).toHaveLength(1);
    expect(alerts[0].type).toBe('warning');
    expect(alerts[0].title).toBe('Seuil de TVA Ventes Proche');
    expect(normalizeSpaces(alerts[0].message)).toContain('91 900 €');
  });

  it('passe en danger au-delà de la tolérance ventes', () => {
    const alerts = buildCaAlerts(thresholdScenarios.venteExceededTva, 2026);

    expect(alerts).toHaveLength(1);
    expect(alerts[0].type).toBe('danger');
    expect(alerts[0].title).toBe('Seuil de TVA Ventes Dépassé');
  });

  it('cumule les deux alertes ventes', () => {
    const alerts = buildCaAlerts(thresholdScenarios.venteApproachingMicro, 2026);

    expect(alerts.map((alert) => alert.title)).toEqual([
      'Seuil de TVA Ventes Dépassé',
      'Plafond Micro-Entreprise Ventes Proche',
    ]);
    expect(normalizeSpaces(alerts[1].message)).toContain('188 700 €');
  });

  it('ordonne les alertes services avant les alertes ventes', () => {
    const mixed = [
      ...thresholdScenarios.serviceApproachingMicro,
      ...thresholdScenarios.venteApproachingMicro,
    ];

    expect(buildCaAlerts(mixed, 2026).map((alert) => alert.title)).toEqual([
      'Seuil de TVA Services Dépassé',
      'Plafond Micro-Entreprise Services Proche',
      'Seuil de TVA Ventes Dépassé',
      'Plafond Micro-Entreprise Ventes Proche',
    ]);
  });
});
