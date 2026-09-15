/**
 * Agrégats du tableau de bord, livre des recettes et export CSV.
 *
 * Extrait de `App.jsx` (calculateCA, calculateTotalExpenses,
 * getServiceTypeBreakdown, getMonthlyFinancialsData, getExpensesCategoryData)
 * et de `ComptaTab.jsx` (registre des recettes, export CSV).
 *
 * Les fonctions qui dépendaient de `new Date().getFullYear()` prennent
 * désormais l'année en paramètre : sans cela elles sont intestables et leur
 * résultat change au 1er janvier.
 */

import { chargesForInvoice } from './urssaf';
import { formatCsvAmount } from './money';

export const EXPENSE_CATEGORIES = [
  'Achats',
  'Déplacements',
  'Logiciels',
  'Télécoms',
  'Bureautique',
  'Cotisations',
  'Autre',
];

export const MONTH_LABELS = [
  'Jan', 'Fév', 'Mar', 'Avr', 'Mai', 'Jun',
  'Jul', 'Aoû', 'Sep', 'Oct', 'Nov', 'Déc',
];

/** Plancher du maximum de l'axe des ordonnées du graphe mensuel. */
export const CHART_MIN_SCALE = 1000;

/** Marge appliquée aux extrema du graphe mensuel. */
export const CHART_HEADROOM = 1.15;

/**
 * Chiffre d'affaires facturé et encaissé.
 * Le facturé exclut les brouillons ; l'encaissé ne retient que les factures payées.
 */
export function calculateCA(invoices) {
  let htFacture = 0;
  let ttcFacture = 0;
  let htEncaisse = 0;
  let ttcEncaisse = 0;

  invoices.forEach((invoice) => {
    if (invoice.status !== 'brouillon') {
      htFacture += invoice.amountHt;
      ttcFacture += invoice.amountTotal;
    }
    if (invoice.status === 'payee') {
      htEncaisse += invoice.amountHt;
      ttcEncaisse += invoice.amountTotal;
    }
  });

  return { ht: htEncaisse, ttc: ttcEncaisse, htFacture, ttcFacture };
}

export function calculateTotalExpenses(expenses) {
  return expenses.reduce((total, expense) => total + expense.amount, 0);
}

/**
 * Répartition du CA encaissé par type d'activité, en pourcentage.
 * Un total nul renvoie 0 partout plutôt que NaN.
 */
export function getServiceTypeBreakdown(invoices) {
  let bnc = 0;
  let bic = 0;
  let vente = 0;

  invoices.forEach((invoice) => {
    if (invoice.status !== 'payee') return;
    if (invoice.serviceType === 'service_bnc') bnc += invoice.amountHt;
    else if (invoice.serviceType === 'service_bic') bic += invoice.amountHt;
    else if (invoice.serviceType === 'vente') vente += invoice.amountHt;
  });

  const total = bnc + bic + vente;
  return {
    bnc: total > 0 ? (bnc / total) * 100 : 0,
    bic: total > 0 ? (bic / total) * 100 : 0,
    vente: total > 0 ? (vente / total) * 100 : 0,
    total,
  };
}

/**
 * Séries mensuelles du graphe : CA, dépenses, cotisations et résultat.
 *
 * ⚠️ Incohérence connue, préservée : le CA et les cotisations mensuels portent
 * sur les factures « non brouillon » — donc émises mais pas nécessairement
 * encaissées — alors que le total annuel affiché juste à côté ne compte que
 * l'encaissé. Les deux chiffres ne se recoupent pas. À trancher en phase 3.
 */
export function getMonthlyFinancialsData(invoices, expenses, settings, year) {
  const caPerMonth = Array(12).fill(0);
  const expensesPerMonth = Array(12).fill(0);
  const chargesPerMonth = Array(12).fill(0);
  const profitPerMonth = Array(12).fill(0);

  invoices.forEach((invoice) => {
    if (invoice.status === 'brouillon') return;
    const invoiceDate = new Date(invoice.date);
    if (invoiceDate.getFullYear() !== year) return;

    const monthIndex = invoiceDate.getMonth();
    caPerMonth[monthIndex] += invoice.amountHt;
    chargesPerMonth[monthIndex] += chargesForInvoice(invoice, settings);
  });

  expenses.forEach((expense) => {
    const expenseDate = new Date(expense.date);
    if (expenseDate.getFullYear() !== year) return;
    expensesPerMonth[expenseDate.getMonth()] += expense.amount;
  });

  for (let i = 0; i < 12; i += 1) {
    profitPerMonth[i] = caPerMonth[i] - expensesPerMonth[i] - chargesPerMonth[i];
  }

  const allValues = [...caPerMonth, ...profitPerMonth];

  return {
    labels: MONTH_LABELS,
    caValues: caPerMonth,
    profitValues: profitPerMonth,
    maxVal: Math.max(...allValues, CHART_MIN_SCALE) * CHART_HEADROOM,
    minVal: Math.min(...allValues, 0) * CHART_HEADROOM,
  };
}

/**
 * Ventilation des dépenses par catégorie.
 * Une catégorie inconnue bascule dans « Autre » ; les catégories à zéro sont
 * retirées de la liste.
 */
export function getExpensesCategoryData(expenses) {
  const totals = Object.fromEntries(EXPENSE_CATEGORIES.map((category) => [category, 0]));
  let total = 0;

  expenses.forEach((expense) => {
    const category = expense.category || 'Autre';
    if (totals[category] !== undefined) {
      totals[category] += expense.amount;
    } else {
      totals.Autre += expense.amount;
    }
    total += expense.amount;
  });

  const list = EXPENSE_CATEGORIES
    .map((category) => {
      const amount = totals[category];
      return {
        key: category,
        label: category,
        amount,
        pct: total > 0 ? (amount / total) * 100 : 0,
      };
    })
    .filter((entry) => entry.amount > 0);

  return { total, list };
}

/**
 * Livre des recettes : factures encaissées, par ordre chronologique
 * d'encaissement. `direction` vaut 'asc' pour l'export réglementaire,
 * 'desc' pour l'affichage à l'écran.
 */
export function getRecettesLedger(invoices, direction = 'asc') {
  const sign = direction === 'asc' ? 1 : -1;
  return invoices
    .filter((invoice) => invoice.status === 'payee')
    .slice()
    .sort((a, b) => {
      const dateA = new Date(a.paymentDate || a.date);
      const dateB = new Date(b.paymentDate || b.date);
      return sign * (dateA - dateB);
    });
}

/**
 * Export CSV du livre des recettes.
 * Séparateur point-virgule, décimale virgule, BOM UTF-8 en tête pour qu'Excel
 * reconnaisse l'encodage — format attendu par l'existant, à reproduire en Rust.
 */
export function buildRecettesCsv(invoices) {
  const header = '﻿Date Encaissement;Facture;Client;Moyen de Paiement;Montant HT;Montant TTC\n';

  return getRecettesLedger(invoices, 'asc').reduce((csv, invoice) => {
    const date = new Date(invoice.paymentDate || invoice.date).toLocaleDateString('fr-FR');
    const method = invoice.paymentMethod || 'Virement';
    return `${csv}"${date}";"${invoice.invoiceNumber}";"${invoice.companyName}";"${method}";${formatCsvAmount(invoice.amountHt)};${formatCsvAmount(invoice.amountTotal)}\n`;
  }, header);
}

/** Nom du fichier CSV, tel que produit aujourd'hui par `ComptaTab`. */
export function recettesCsvFilename(year) {
  return `Livre_des_recettes_${year}.csv`;
}
