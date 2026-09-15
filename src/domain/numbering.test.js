import { describe, it, expect } from 'vitest';
import {
  sanitizeClientName,
  yearBounds,
  formatDocumentNumber,
  sequenceFromCount,
  buildDocumentNumber,
} from './numbering';
import { invoices, estimates } from './__tests__/fixtures';

describe('sanitizeClientName', () => {
  it('met en majuscules et retire les espaces', () => {
    expect(sanitizeClientName('Stark Industries')).toBe('STARKINDUS');
  });

  it('retire les accents sans perdre la lettre', () => {
    expect(sanitizeClientName('Éditions Yggdrasil')).toBe('EDITIONSYG');
    expect(sanitizeClientName('Crème Brûlée')).toBe('CREMEBRULE');
  });

  it('retire la ponctuation et les symboles', () => {
    expect(sanitizeClientName('Asgard Coffee & Co')).toBe('ASGARDCOFF');
    expect(sanitizeClientName("L'Atelier d'Odin")).toBe('LATELIERDO');
  });

  it('tronque à dix caractères', () => {
    expect(sanitizeClientName('Wayne Enterprises')).toBe('WAYNEENTER');
    expect(sanitizeClientName('ABCDEFGHIJKLMNOP')).toBe('ABCDEFGHIJ');
  });

  it('conserve les chiffres', () => {
    expect(sanitizeClientName('Studio 54')).toBe('STUDIO54');
  });

  /**
   * Deux raisons sociales distinctes peuvent produire le même fragment. Combiné
   * au défaut D3, cela rend la collision de numéros d'autant plus probable.
   */
  it('produit le même fragment pour deux noms différents', () => {
    expect(sanitizeClientName('Wayne Enterprises')).toBe(sanitizeClientName('Wayne Enterprise SA'));
  });

  it('renvoie une chaîne vide si le nom ne contient aucun caractère retenu', () => {
    expect(sanitizeClientName('*** ???')).toBe('');
  });
});

describe('yearBounds', () => {
  it('encadre l’année civile de la date', () => {
    expect(yearBounds('2026-04-18T10:00:00.000Z')).toEqual({
      year: '2026',
      start: '2026-01-01T00:00:00.000Z',
      end: '2026-12-31T23:59:59.000Z',
    });
  });

  /**
   * ⚠️ L'année est lue en heure locale alors que les bornes sont construites en
   * UTC. À Paris, une facture du 31 décembre à 23 h 30 porte l'année N côté
   * `getFullYear()` mais tombe au 31/12 22 h 30 UTC — donc dans la fenêtre.
   * L'écart se manifeste dans l'autre sens : une facture du 1er janvier à
   * 00 h 30 locale est datée du 31 décembre 22 h 30 UTC, hors de la fenêtre de
   * l'année qu'elle annonce. Comportement figé, à trancher en phase 2.
   */
  it('décale les bornes d’un fuseau horaire (défaut connu)', () => {
    const newYearEve = yearBounds('2026-01-01T00:30:00+01:00');

    expect(newYearEve.year).toBe('2026');
    expect(new Date('2026-01-01T00:30:00+01:00').toISOString()).toBe('2025-12-31T23:30:00.000Z');
    expect(new Date('2026-01-01T00:30:00+01:00').toISOString() < newYearEve.start).toBe(true);
  });
});

describe('formatDocumentNumber', () => {
  it('assemble préfixe, client, année et séquence', () => {
    expect(formatDocumentNumber('FAC', 'Stark Industries', '2026', 1))
      .toBe('FAC-STARKINDUS-2026-0001');
  });

  it('complète la séquence à quatre chiffres', () => {
    expect(formatDocumentNumber('DEV', 'Stark Industries', '2026', 42))
      .toBe('DEV-STARKINDUS-2026-0042');
  });

  it('ne tronque pas au-delà de quatre chiffres', () => {
    expect(formatDocumentNumber('FAC', 'Stark Industries', '2026', 12345))
      .toBe('FAC-STARKINDUS-2026-12345');
  });
});

describe('buildDocumentNumber', () => {
  it('produit un numéro de facture', () => {
    expect(buildDocumentNumber('invoice', 'Wayne Enterprises', '2026-03-12T10:00:00.000Z', 1))
      .toBe('FAC-WAYNEENTER-2026-0002');
  });

  it('produit un numéro de devis', () => {
    expect(buildDocumentNumber('estimate', 'Asgard Coffee & Co', '2026-07-09T10:00:00.000Z', 3))
      .toBe('DEV-ASGARDCOFF-2026-0004');
  });
});

/**
 * ⚠️ Défaut D3 — article 242 nonies A du CGI.
 *
 * Ces trois tests décrivent un comportement illégal. Ils ne sont pas là pour
 * garantir qu'il perdure, mais pour prouver qu'il existe et donner à la phase 2
 * une cible de non-régression inversée : quand la séquence transactionnelle
 * sera en place, ces tests devront être réécrits, pas simplement supprimés.
 */
describe('sequenceFromCount — non-conformité réglementaire', () => {
  it('dérive la séquence du nombre de pièces, pas du dernier numéro émis', () => {
    expect(sequenceFromCount(0)).toBe(1);
    expect(sequenceFromCount(6)).toBe(7);
  });

  it('réutilise un numéro déjà émis après une suppression', () => {
    const existing = ['FAC-STARKINDUS-2026-0001', 'FAC-WAYNEENTER-2026-0002'];

    // La pièce 0001 est supprimée : il ne reste qu'une facture sur l'exercice.
    const afterDeletion = existing.slice(1);
    const nextNumber = buildDocumentNumber(
      'invoice', 'Asgard Coffee & Co', '2026-05-01T10:00:00.000Z', afterDeletion.length,
    );

    expect(nextNumber).toBe('FAC-ASGARDCOFF-2026-0002');
    expect(nextNumber.endsWith('0002')).toBe(true);
    expect(existing[1].endsWith('0002')).toBe(true);
  });

  it('attribue le même numéro à deux créations concurrentes', () => {
    const countAtReadTime = 6;

    const first = buildDocumentNumber('invoice', 'Stark Industries', '2026-09-01T10:00:00.000Z', countAtReadTime);
    const second = buildDocumentNumber('invoice', 'Wayne Enterprises', '2026-09-01T10:00:00.000Z', countAtReadTime);

    expect(first).toBe('FAC-STARKINDUS-2026-0007');
    expect(second).toBe('FAC-WAYNEENTER-2026-0007');
  });
});

/**
 * Vérifie que le jeu de référence est cohérent avec le générateur : sans cela,
 * les fixtures ne représenteraient pas des données réellement produites par
 * l'application et ne vaudraient rien comme oracle.
 */
describe('cohérence du jeu de référence', () => {
  it('chaque facture porte le numéro que le générateur produirait', () => {
    const perYear = {};

    invoices
      .slice()
      .sort((a, b) => new Date(a.date) - new Date(b.date))
      .forEach((invoice) => {
        const { year } = yearBounds(invoice.date);
        perYear[year] = (perYear[year] || 0) + 1;

        expect(invoice.invoiceNumber).toBe(
          formatDocumentNumber('FAC', invoice.companyName, year, perYear[year]),
        );
      });
  });

  it('chaque devis porte le numéro que le générateur produirait', () => {
    const perYear = {};

    estimates
      .slice()
      .sort((a, b) => new Date(a.date) - new Date(b.date))
      .forEach((estimate) => {
        const { year } = yearBounds(estimate.date);
        perYear[year] = (perYear[year] || 0) + 1;

        expect(estimate.estimateNumber).toBe(
          formatDocumentNumber('DEV', estimate.companyName, year, perYear[year]),
        );
      });
  });
});
