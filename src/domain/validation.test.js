import { describe, it, expect } from 'vitest';
import {
  EMAIL_REGEX,
  ADDRESS_REGEX,
  formatPhoneInput,
  validateClientForm,
  validateInvoiceForm,
  validateEstimateForm,
  validateExpenseForm,
  isValid,
} from './validation';
import { clients } from './__tests__/fixtures';

const validClient = {
  companyName: 'Stark Industries',
  contactName: 'Pepper Potts',
  email: 'pepper@stark.com',
  phone: '06 11 22 33 44',
  address: '108 route de Malibu',
};

describe('EMAIL_REGEX', () => {
  it('accepte les adresses courantes', () => {
    ['a@b.fr', 'pepper@stark.com', 'contact+devis@sous-domaine.example.co']
      .forEach((email) => expect(EMAIL_REGEX.test(email)).toBe(true));
  });

  it('refuse les adresses malformées', () => {
    ['', 'pepper', 'pepper@', '@stark.com', 'pepper@stark', 'pepper @stark.com']
      .forEach((email) => expect(EMAIL_REGEX.test(email)).toBe(false));
  });
});

describe('ADDRESS_REGEX', () => {
  it('accepte les types de voie reconnus', () => {
    ['12 rue de Paris', '1 boulevard Haussmann', '45 av des Champs', '3 place des Neuf Mondes',
      '108 route de Malibu', '7 impasse du Chat', '2 quai de Seine']
      .forEach((address) => expect(ADDRESS_REGEX.test(address)).toBe(true));
  });

  it('ignore la casse', () => {
    expect(ADDRESS_REGEX.test('12 RUE DE PARIS')).toBe(true);
  });

  /**
   * ⚠️ Comportement restrictif, figé mais discutable. Ces adresses sont
   * parfaitement valides et pourtant refusées : la validation bloque des
   * saisies légitimes. À revoir hors phase 0.
   */
  it('refuse des adresses françaises valides (défaut connu)', () => {
    ['12 bis rue de Paris', 'Lieu-dit Le Moulin', '5 cours Mirabeau',
      '8 passage Verdeau', '3 villa des Ternes', 'Le Bourg']
      .forEach((address) => expect(ADDRESS_REGEX.test(address)).toBe(false));
  });

  it('refuse une adresse sans numéro ou sans libellé', () => {
    expect(ADDRESS_REGEX.test('rue de Paris')).toBe(false);
    expect(ADDRESS_REGEX.test('12 rue')).toBe(false);
  });

  it('accepte les adresses du jeu de référence', () => {
    clients.forEach((client) => {
      expect(ADDRESS_REGEX.test(client.address)).toBe(true);
    });
  });
});

describe('formatPhoneInput', () => {
  it('groupe les chiffres par deux', () => {
    expect(formatPhoneInput('0611223344')).toBe('06 11 22 33 44');
  });

  it('retire tout ce qui n’est pas un chiffre', () => {
    expect(formatPhoneInput('+33 (0)6.11.22.33.44')).toBe('33 06 11 22 33');
  });

  it('tronque au-delà de dix chiffres', () => {
    expect(formatPhoneInput('06112233445566')).toBe('06 11 22 33 44');
  });

  it('gère une saisie partielle', () => {
    expect(formatPhoneInput('')).toBe('');
    expect(formatPhoneInput('0')).toBe('0');
    expect(formatPhoneInput('061')).toBe('06 1');
  });
});

describe('validateClientForm', () => {
  it('accepte un client complet', () => {
    expect(validateClientForm(validClient)).toEqual({});
    expect(isValid(validateClientForm(validClient))).toBe(true);
  });

  it('exige chaque champ', () => {
    const errors = validateClientForm({
      companyName: '', contactName: '', email: '', phone: '', address: '',
    });

    expect(errors).toEqual({
      companyName: "Nom d'entreprise requis",
      contactName: 'Nom du contact requis',
      email: 'Email requis',
      phone: 'Numéro de téléphone requis',
      address: 'Adresse requise',
    });
  });

  it('traite une suite d’espaces comme un champ vide', () => {
    const errors = validateClientForm({ ...validClient, companyName: '   ' });
    expect(errors.companyName).toBe("Nom d'entreprise requis");
  });

  it('signale un e-mail mal formé', () => {
    const errors = validateClientForm({ ...validClient, email: 'pepper@stark' });
    expect(errors.email).toBe('Format email invalide (ex: client@domaine.fr)');
  });

  it('exige exactement dix chiffres, espaces exclus', () => {
    expect(validateClientForm({ ...validClient, phone: '06 11 22 33' }).phone)
      .toBe('Le numéro doit faire exactement 10 chiffres');
    expect(validateClientForm({ ...validClient, phone: '0611223344' }).phone)
      .toBeUndefined();
  });

  it('signale une adresse au mauvais format', () => {
    const errors = validateClientForm({ ...validClient, address: 'Quelque part' });
    expect(errors.address).toContain('Format invalide');
  });
});

describe('validateInvoiceForm', () => {
  const validInvoice = { clientId: '1', description: 'Audit', amountHt: '5000' };

  it('accepte une facture complète', () => {
    expect(validateInvoiceForm(validInvoice)).toEqual({});
  });

  it('exige client, description et montant', () => {
    expect(validateInvoiceForm({ clientId: '', description: '', amountHt: '' })).toEqual({
      clientId: 'Client requis',
      description: 'Description de la prestation requise',
      amountHt: 'Veuillez entrer un montant HT valide supérieur à 0',
    });
  });

  it('refuse un montant nul ou négatif', () => {
    expect(validateInvoiceForm({ ...validInvoice, amountHt: '0' }).amountHt).toBeDefined();
    expect(validateInvoiceForm({ ...validInvoice, amountHt: '-100' }).amountHt).toBeDefined();
  });

  it('refuse un montant non numérique', () => {
    expect(validateInvoiceForm({ ...validInvoice, amountHt: 'abc' }).amountHt).toBeDefined();
  });

  it('accepte un montant décimal', () => {
    expect(validateInvoiceForm({ ...validInvoice, amountHt: '1899.99' })).toEqual({});
  });

  /**
   * ⚠️ `parseFloat('1899,99')` vaut 1899 : une virgule décimale, saisie
   * naturelle en France, tronque silencieusement les centimes au lieu d'être
   * refusée. Comportement figé, à corriger hors phase 0.
   */
  it('tronque une virgule décimale au lieu de la refuser (défaut connu)', () => {
    expect(validateInvoiceForm({ ...validInvoice, amountHt: '1899,99' })).toEqual({});
    expect(parseFloat('1899,99')).toBe(1899);
  });
});

describe('validateEstimateForm', () => {
  it('porte ses propres libellés, plus courts que ceux des factures', () => {
    expect(validateEstimateForm({ clientId: '', description: '', amountHt: '' })).toEqual({
      clientId: 'Client requis',
      description: 'Description requise',
      amountHt: 'Montant HT valide requis (> 0)',
    });
  });

  it('accepte un devis complet', () => {
    expect(validateEstimateForm({ clientId: '2', description: 'Étude', amountHt: '24000' }))
      .toEqual({});
  });
});

describe('validateExpenseForm', () => {
  it('exige fournisseur et montant', () => {
    expect(validateExpenseForm({ merchant: '', amount: '' })).toEqual({
      merchant: 'Fournisseur requis',
      amount: 'Montant supérieur à 0 requis',
    });
  });

  it('accepte une dépense complète', () => {
    expect(validateExpenseForm({ merchant: 'JetBrains', amount: '289.90' })).toEqual({});
  });

  it('n’exige ni description ni catégorie', () => {
    expect(validateExpenseForm({ merchant: 'OVH', amount: '119' })).toEqual({});
  });
});

describe('isValid', () => {
  it('ne vaut vrai que sans erreur', () => {
    expect(isValid({})).toBe(true);
    expect(isValid({ email: 'Email requis' })).toBe(false);
  });
});
