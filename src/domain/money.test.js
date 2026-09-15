import { describe, it, expect } from 'vitest';
import { computeAmounts, round2, sum, formatCsvAmount, formatEuros } from './money';
import { normalizeSpaces } from './__tests__/fixtures';

describe('computeAmounts', () => {
  it('applique un taux de TVA entier', () => {
    expect(computeAmounts(5000, 20)).toEqual({
      amountHt: 5000,
      tvaRate: 20,
      amountTva: 1000,
      amountTotal: 6000,
    });
  });

  it('applique un taux réduit', () => {
    expect(computeAmounts(3200, 5.5)).toEqual({
      amountHt: 3200,
      tvaRate: 5.5,
      amountTva: 176,
      amountTotal: 3376,
    });
  });

  it('accepte des chaînes, comme les champs de formulaire les fournissent', () => {
    expect(computeAmounts('1200', '0')).toEqual({
      amountHt: 1200,
      tvaRate: 0,
      amountTva: 0,
      amountTotal: 1200,
    });
  });

  /**
   * ⚠️ Défaut D5 — ces valeurs ne sont PAS des erreurs de test.
   * Elles figent l'arithmétique flottante réellement en production. La phase 3,
   * en `rust_decimal`, produira 379.998 et 2279.988 exactement : l'écart devra
   * apparaître dans le rapport de parité, pas être découvert après bascule.
   */
  it('accumule les artefacts de virgule flottante (défaut D5)', () => {
    const { amountTva, amountTotal } = computeAmounts(1899.99, 20);

    expect(amountTva).toBe(379.99800000000005);
    expect(amountTotal).toBe(2279.9880000000003);

    expect(amountTva).not.toBe(379.998);
    expect(amountTotal).not.toBe(2279.988);
  });

  it('accumule aussi sur un taux réduit', () => {
    const { amountTva, amountTotal } = computeAmounts(1450.35, 5.5);

    expect(amountTva).toBe(79.76925);
    expect(amountTotal).toBe(1530.11925);
  });
});

describe('round2', () => {
  it('arrondit au centime', () => {
    expect(round2(379.99800000000005)).toBe(380);
    expect(round2(79.76925)).toBe(79.77);
    expect(round2(1530.11925)).toBe(1530.12);
  });
});

describe('sum', () => {
  it('somme dans l’ordre du tableau', () => {
    expect(sum([1000, 1620, 379.99800000000005])).toBe(2999.998);
  });

  it('renvoie zéro sur un tableau vide', () => {
    expect(sum([])).toBe(0);
  });
});

describe('formatCsvAmount', () => {
  it('utilise la virgule décimale et deux chiffres', () => {
    expect(formatCsvAmount(1899.99)).toBe('1899,99');
    expect(formatCsvAmount(1530.11925)).toBe('1530,12');
    expect(formatCsvAmount(3200)).toBe('3200,00');
  });
});

describe('formatEuros', () => {
  it('sépare les milliers à la française', () => {
    expect(normalizeSpaces(formatEuros(35000))).toBe('35 000');
    expect(normalizeSpaces(formatEuros(1450.35))).toBe('1 450,35');
  });
});
