import { describe, it, expect } from 'vitest';
import {
  baseRateFor,
  effectiveRateFor,
  chargesForInvoice,
  totalUrssafCharges,
  chargesForIssuedInvoices,
  quarterOfMonth,
  invoiceMatchesPeriod,
  urssafDeclaration,
  SERVICE_TYPES,
  ACRE_DIVISOR,
} from './urssaf';
import { invoices, standardSettings, acreSettings } from './__tests__/fixtures';

describe('baseRateFor', () => {
  it('renvoie le taux du type d’activité', () => {
    expect(baseRateFor(SERVICE_TYPES.SERVICE_BNC, standardSettings)).toBe(21.1);
    expect(baseRateFor(SERVICE_TYPES.SERVICE_BIC, standardSettings)).toBe(21.1);
    expect(baseRateFor(SERVICE_TYPES.VENTE, standardSettings)).toBe(12.3);
  });

  it('renvoie zéro sur un type inconnu, sans lever', () => {
    expect(baseRateFor('type_inexistant', standardSettings)).toBe(0);
    expect(baseRateFor(undefined, standardSettings)).toBe(0);
  });
});

describe('effectiveRateFor', () => {
  it('laisse le taux intact hors ACRE', () => {
    expect(effectiveRateFor(SERVICE_TYPES.SERVICE_BNC, standardSettings)).toBe(21.1);
  });

  it('divise le taux par deux sous ACRE', () => {
    expect(effectiveRateFor(SERVICE_TYPES.SERVICE_BNC, acreSettings)).toBe(21.1 / ACRE_DIVISOR);
    expect(effectiveRateFor(SERVICE_TYPES.VENTE, acreSettings)).toBe(6.15);
  });
});

describe('chargesForInvoice', () => {
  it('calcule les cotisations sur le montant HT, pas sur le TTC', () => {
    const invoice = { serviceType: SERVICE_TYPES.SERVICE_BNC, amountHt: 8100, amountTotal: 9720 };
    expect(chargesForInvoice(invoice, standardSettings)).toBe(1709.1);
  });

  it('applique l’abattement ACRE', () => {
    const invoice = { serviceType: SERVICE_TYPES.SERVICE_BNC, amountHt: 8100 };
    expect(chargesForInvoice(invoice, acreSettings)).toBe(854.55);
  });
});

describe('totalUrssafCharges', () => {
  it('ne retient que les factures encaissées', () => {
    expect(totalUrssafCharges(invoices, standardSettings)).toBe(3736.99094);
  });

  it('divise le total par deux sous ACRE', () => {
    expect(totalUrssafCharges(invoices, acreSettings)).toBe(1868.49547);
  });

  it('ignore brouillons et factures envoyées non réglées', () => {
    const paid = invoices.filter((invoice) => invoice.status === 'payee');
    expect(totalUrssafCharges(invoices, standardSettings))
      .toBe(totalUrssafCharges(paid, standardSettings));
  });
});

/**
 * ⚠️ Incohérence connue, figée volontairement.
 *
 * Le tableau de bord mensuel calcule les cotisations sur les factures émises,
 * payées ou non, tandis que le total annuel affiché au-dessus ne compte que
 * l'encaissé. Les deux chiffres diffèrent de 1 002,25 € sur le jeu de
 * référence, et l'utilisateur voit les deux côte à côte.
 *
 * L'URSSAF se déclare sur l'encaissé : c'est `totalUrssafCharges` qui a raison.
 * La correction relève de la phase 3, pas de la phase 0.
 */
describe('chargesForIssuedInvoices — incohérence avec totalUrssafCharges', () => {
  it('compte aussi les factures émises non encaissées', () => {
    expect(chargesForIssuedInvoices(invoices, standardSettings)).toBe(4739.24094);
  });

  it('diverge du total déclarable, ce qui est le défaut', () => {
    const issued = chargesForIssuedInvoices(invoices, standardSettings);
    const collected = totalUrssafCharges(invoices, standardSettings);

    expect(issued).toBeGreaterThan(collected);
    expect(issued - collected).toBeCloseTo(1002.25, 2);
  });
});

describe('quarterOfMonth', () => {
  it('découpe l’année en quatre', () => {
    expect([1, 2, 3].map(quarterOfMonth)).toEqual([1, 1, 1]);
    expect([4, 5, 6].map(quarterOfMonth)).toEqual([2, 2, 2]);
    expect([7, 8, 9].map(quarterOfMonth)).toEqual([3, 3, 3]);
    expect([10, 11, 12].map(quarterOfMonth)).toEqual([4, 4, 4]);
  });
});

describe('invoiceMatchesPeriod', () => {
  const monthly = (year, month) => ({ periodType: 'monthly', year, month });

  it('retient la date d’encaissement, pas celle d’émission', () => {
    const straddling = invoices.find((invoice) => invoice.id === 1);

    expect(invoiceMatchesPeriod(straddling, monthly(2026, 1))).toBe(true);
    expect(invoiceMatchesPeriod(straddling, monthly(2025, 12))).toBe(false);
  });

  it('retombe sur la date d’émission quand l’encaissement n’est pas renseigné', () => {
    const invoice = { status: 'payee', date: '2026-03-12T10:00:00.000Z' };
    expect(invoiceMatchesPeriod(invoice, monthly(2026, 3))).toBe(true);
  });

  it('exclut tout ce qui n’est pas encaissé', () => {
    const sent = invoices.find((invoice) => invoice.status === 'envoyee');
    const draft = invoices.find((invoice) => invoice.status === 'brouillon');

    expect(invoiceMatchesPeriod(sent, monthly(2026, 6))).toBe(false);
    expect(invoiceMatchesPeriod(draft, monthly(2026, 8))).toBe(false);
  });
});

describe('urssafDeclaration', () => {
  it('ventile le premier trimestre 2026 par type d’activité', () => {
    const declaration = urssafDeclaration(invoices, standardSettings, {
      periodType: 'quarterly',
      year: 2026,
      quarter: 1,
    });

    expect(declaration.bnc.ht).toBe(13100);
    expect(declaration.bic.ht).toBe(1899.99);
    expect(declaration.vente.ht).toBe(0);
    expect(declaration.totalCa).toBe(14999.99);
    expect(declaration.totalCharges).toBe(3164.99789);
  });

  it('rattache une facture au trimestre de son encaissement', () => {
    const q2 = urssafDeclaration(invoices, standardSettings, {
      periodType: 'quarterly', year: 2026, quarter: 2,
    });
    const q3 = urssafDeclaration(invoices, standardSettings, {
      periodType: 'quarterly', year: 2026, quarter: 3,
    });

    // Facture 5 : émise en mai (T2), encaissée en juillet (T3).
    expect(q2.vente.ht).toBe(1450.35);
    expect(q3.vente.ht).toBe(3200);
  });

  it('isole un mois', () => {
    const february = urssafDeclaration(invoices, standardSettings, {
      periodType: 'monthly', year: 2026, month: 2,
    });

    expect(february.bnc.ht).toBe(8100);
    expect(february.totalCa).toBe(8100);
    expect(february.totalCharges).toBe(1709.1);
  });

  it('renvoie une déclaration à zéro sur une période vide', () => {
    const november = urssafDeclaration(invoices, standardSettings, {
      periodType: 'monthly', year: 2026, month: 11,
    });

    expect(november.totalCa).toBe(0);
    expect(november.totalCharges).toBe(0);
    expect(november.bnc).toEqual({ ht: 0, rate: 21.1, charges: 0 });
  });

  it('expose le taux appliqué à chaque ligne, ACRE comprise', () => {
    const declaration = urssafDeclaration(invoices, acreSettings, {
      periodType: 'quarterly', year: 2026, quarter: 1,
    });

    expect(declaration.bnc.rate).toBe(10.55);
    expect(declaration.vente.rate).toBe(6.15);
    expect(declaration.totalCharges).toBe(3164.99789 / 2);
  });
});
