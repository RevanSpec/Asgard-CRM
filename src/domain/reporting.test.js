import { describe, it, expect } from 'vitest';
import {
  calculateCA,
  calculateTotalExpenses,
  getServiceTypeBreakdown,
  getMonthlyFinancialsData,
  getExpensesCategoryData,
  getRecettesLedger,
  buildRecettesCsv,
  recettesCsvFilename,
  MONTH_LABELS,
} from './reporting';
import { invoices, expenses, standardSettings, acreSettings } from './__tests__/fixtures';

describe('calculateCA', () => {
  const ca = calculateCA(invoices);

  it('distingue le facturé de l’encaissé', () => {
    expect(ca.ht).toBe(19650.34);
    expect(ca.htFacture).toBe(24400.34);
  });

  it('écarte les brouillons du facturé', () => {
    const drafts = invoices.filter((invoice) => invoice.status === 'brouillon');
    expect(drafts).toHaveLength(1);
    expect(ca.htFacture).toBe(ca.ht + 4750);
  });

  it('calcule aussi les totaux TTC', () => {
    expect(ca.ttc).toBe(22906.10725);
    expect(ca.ttcFacture).toBe(28606.10725);
  });

  it('renvoie zéro sur un portefeuille vide', () => {
    expect(calculateCA([])).toEqual({ ht: 0, ttc: 0, htFacture: 0, ttcFacture: 0 });
  });
});

describe('calculateTotalExpenses', () => {
  it('somme toutes les dépenses, tous exercices confondus', () => {
    expect(calculateTotalExpenses(expenses)).toBe(1846.09);
  });

  it('renvoie zéro sans dépense', () => {
    expect(calculateTotalExpenses([])).toBe(0);
  });
});

describe('getServiceTypeBreakdown', () => {
  it('répartit le CA encaissé en pourcentages', () => {
    const breakdown = getServiceTypeBreakdown(invoices);

    expect(breakdown.total).toBe(19650.34);
    expect(breakdown.bnc).toBeCloseTo(66.6655, 4);
    expect(breakdown.bic).toBeCloseTo(9.669, 3);
    expect(breakdown.vente).toBeCloseTo(23.6655, 4);
  });

  it('somme les trois parts à cent', () => {
    const { bnc, bic, vente } = getServiceTypeBreakdown(invoices);
    expect(bnc + bic + vente).toBeCloseTo(100, 10);
  });

  it('renvoie zéro plutôt que NaN sur un total nul', () => {
    expect(getServiceTypeBreakdown([])).toEqual({ bnc: 0, bic: 0, vente: 0, total: 0 });
  });
});

describe('getMonthlyFinancialsData', () => {
  const monthly = getMonthlyFinancialsData(invoices, expenses, standardSettings, 2026);

  it('porte les douze libellés de mois', () => {
    expect(monthly.labels).toEqual(MONTH_LABELS);
    expect(monthly.caValues).toHaveLength(12);
  });

  it('ventile le CA par mois d’émission', () => {
    expect(monthly.caValues).toEqual([0, 8100, 1899.99, 1450.35, 3200, 4750, 0, 0, 0, 0, 0, 0]);
  });

  /**
   * ⚠️ La facture 1, émise en décembre 2025 et encaissée en janvier 2026,
   * n'apparaît dans aucun mois de 2026 : le graphe suit la date d'émission.
   * Le total annuel affiché au-dessus la compte pourtant, puisqu'il suit
   * l'encaissement. Les deux ne se recoupent pas — défaut à trancher en phase 3.
   */
  it('classe une facture à cheval sur son mois d’émission, pas d’encaissement', () => {
    expect(monthly.caValues[0]).toBe(0);
    expect(getMonthlyFinancialsData(invoices, expenses, standardSettings, 2025).caValues[11])
      .toBe(5000);
  });

  it('calcule le résultat mensuel, cotisations et dépenses déduites', () => {
    expect(monthly.profitValues[0]).toBe(-289.9);
    expect(monthly.profitValues[1]).toBe(6236.5);
    expect(monthly.profitValues[8]).toBe(-210);
  });

  it('accumule les artefacts flottants dans le résultat (défaut D5)', () => {
    expect(monthly.profitValues[3]).toBe(451.4569499999999);
  });

  it('borne l’axe des ordonnées avec une marge de quinze pour cent', () => {
    expect(monthly.maxVal).toBe(9315);
    expect(monthly.minVal).toBe(-333.38499999999993);
  });

  it('applique un plancher de mille au maximum', () => {
    const empty = getMonthlyFinancialsData([], [], standardSettings, 2026);
    expect(empty.maxVal).toBe(1150);
    expect(empty.minVal).toBe(0);
  });

  it('réduit les cotisations mensuelles sous ACRE', () => {
    const withAcre = getMonthlyFinancialsData(invoices, expenses, acreSettings, 2026);
    expect(withAcre.profitValues[1]).toBeGreaterThan(monthly.profitValues[1]);
  });
});

describe('getExpensesCategoryData', () => {
  const data = getExpensesCategoryData(expenses);

  it('totalise toutes les dépenses', () => {
    expect(data.total).toBe(1846.09);
  });

  it('bascule une catégorie inconnue dans « Autre »', () => {
    const autre = data.list.find((entry) => entry.key === 'Autre');
    expect(autre.amount).toBe(145);
  });

  it('retire les catégories sans dépense', () => {
    expect(data.list.map((entry) => entry.key)).toEqual([
      'Achats', 'Déplacements', 'Logiciels', 'Télécoms', 'Bureautique', 'Cotisations', 'Autre',
    ]);
    expect(data.list.every((entry) => entry.amount > 0)).toBe(true);
  });

  it('calcule le poids de chaque catégorie', () => {
    const logiciels = data.list.find((entry) => entry.key === 'Logiciels');
    expect(logiciels.amount).toBe(408.9);
    expect(logiciels.pct).toBeCloseTo(22.1495, 4);
  });

  it('renvoie une liste vide sans dépense', () => {
    expect(getExpensesCategoryData([])).toEqual({ total: 0, list: [] });
  });
});

describe('getRecettesLedger', () => {
  it('ne retient que les factures encaissées', () => {
    const ledger = getRecettesLedger(invoices);
    expect(ledger).toHaveLength(5);
    expect(ledger.every((invoice) => invoice.status === 'payee')).toBe(true);
  });

  it('classe par date d’encaissement croissante pour l’export', () => {
    expect(getRecettesLedger(invoices, 'asc').map((invoice) => invoice.id))
      .toEqual([1, 2, 3, 4, 5]);
  });

  it('classe par date décroissante pour l’affichage', () => {
    expect(getRecettesLedger(invoices, 'desc').map((invoice) => invoice.id))
      .toEqual([5, 4, 3, 2, 1]);
  });

  it('ne modifie pas le tableau reçu', () => {
    const original = invoices.map((invoice) => invoice.id);
    getRecettesLedger(invoices, 'desc');
    expect(invoices.map((invoice) => invoice.id)).toEqual(original);
  });
});

describe('buildRecettesCsv', () => {
  const csv = buildRecettesCsv(invoices);

  it('commence par un BOM UTF-8', () => {
    expect(csv.charCodeAt(0)).toBe(0xfeff);
  });

  it('porte l’en-tête réglementaire', () => {
    expect(csv.split('\n')[0]).toBe(
      '\uFEFFDate Encaissement;Facture;Client;Moyen de Paiement;Montant HT;Montant TTC',
    );
  });

  it('écrit une ligne par recette, dans l’ordre chronologique', () => {
    const rows = csv.trimEnd().split('\n').slice(1);

    expect(rows).toHaveLength(5);
    expect(rows[0]).toBe(
      '"08/01/2026";"FAC-STARKINDUS-2025-0001";"Stark Industries";"virement";5000,00;6000,00',
    );
    expect(rows[4]).toBe(
      '"02/07/2026";"FAC-EDITIONSYG-2026-0004";"Éditions Yggdrasil";"especes";3200,00;3376,00',
    );
  });

  it('arrondit les montants au centime avec une virgule décimale', () => {
    const rows = csv.trimEnd().split('\n').slice(1);
    expect(rows[2]).toContain('1899,99;2279,99');
  });

  it('remplace un moyen de paiement absent par « Virement »', () => {
    const withoutMethod = [{
      status: 'payee',
      date: '2026-05-05T10:00:00.000Z',
      invoiceNumber: 'FAC-TEST-2026-0001',
      companyName: 'Test',
      amountHt: 100,
      amountTotal: 120,
    }];

    expect(buildRecettesCsv(withoutMethod)).toContain('"Virement"');
  });

  it('ne produit que l’en-tête sans recette', () => {
    const emptyCsv = buildRecettesCsv([]);

    expect(emptyCsv.trimEnd().split('\n')).toHaveLength(1);
    expect(emptyCsv.endsWith('Montant TTC\n')).toBe(true);
  });
});

describe('recettesCsvFilename', () => {
  it('nomme le fichier par exercice', () => {
    expect(recettesCsvFilename(2026)).toBe('Livre_des_recettes_2026.csv');
  });
});
