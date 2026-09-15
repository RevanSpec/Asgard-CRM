import { describe, it, expect } from 'vitest';
import {
  resolveTemplate,
  isInvoiceKind,
  documentNumberOf,
  buildSubject,
  pickTemplate,
  buildTemplateData,
  buildEmailDraft,
  DOCUMENT_KINDS,
} from './templates';
import { invoices, estimates, clients, standardSettings } from './__tests__/fixtures';

const defaults = {
  emailTemplateInvoice: 'Facture par défaut {documentNumber}',
  emailTemplateEstimate: 'Devis par défaut {documentNumber}',
  emailTemplateReminder: 'Relance par défaut {documentNumber}',
};

const settingsWithTemplates = {
  ...standardSettings,
  emailTemplateInvoice: 'Bonjour {clientName},\n\nFacture {documentNumber} : {amountTotal} €.\n\n{senderName}\n{senderCompany}',
  emailTemplateEstimate: 'Bonjour {clientName}, voici le devis {documentNumber}.',
  emailTemplateReminder: 'Bonjour {clientName}, la facture {documentNumber} du {documentDate} reste impayée.',
};

describe('resolveTemplate', () => {
  it('remplace chaque jeton par sa valeur', () => {
    expect(resolveTemplate('Bonjour {clientName}, facture {documentNumber}.', {
      clientName: 'Stark Industries',
      documentNumber: 'FAC-STARKINDUS-2026-0001',
    })).toBe('Bonjour Stark Industries, facture FAC-STARKINDUS-2026-0001.');
  });

  it('remplace toutes les occurrences d’un même jeton', () => {
    expect(resolveTemplate('{clientName} / {clientName}', { clientName: 'Wayne' }))
      .toBe('Wayne / Wayne');
  });

  it('remplace un jeton sans valeur par une chaîne vide', () => {
    expect(resolveTemplate('Échéance : {dueDate}.', {})).toBe('Échéance : .');
  });

  it('renvoie une chaîne vide sur un gabarit absent', () => {
    expect(resolveTemplate('', {})).toBe('');
    expect(resolveTemplate(null, {})).toBe('');
    expect(resolveTemplate(undefined, {})).toBe('');
  });

  it('laisse intact un jeton inconnu', () => {
    expect(resolveTemplate('Solde {montantRestant}.', { clientName: 'X' }))
      .toBe('Solde {montantRestant}.');
  });

  it('préserve les sauts de ligne', () => {
    expect(resolveTemplate('A\n\n{clientName}', { clientName: 'B' })).toBe('A\n\nB');
  });
});

describe('isInvoiceKind', () => {
  it('range la relance du côté des factures', () => {
    expect(isInvoiceKind(DOCUMENT_KINDS.INVOICE)).toBe(true);
    expect(isInvoiceKind(DOCUMENT_KINDS.REMINDER)).toBe(true);
    expect(isInvoiceKind(DOCUMENT_KINDS.ESTIMATE)).toBe(false);
  });
});

describe('documentNumberOf', () => {
  const invoice = invoices[1];
  const estimate = estimates[0];

  it('lit le numéro de facture pour une facture et une relance', () => {
    expect(documentNumberOf(invoice, DOCUMENT_KINDS.INVOICE)).toBe('FAC-STARKINDUS-2026-0001');
    expect(documentNumberOf(invoice, DOCUMENT_KINDS.REMINDER)).toBe('FAC-STARKINDUS-2026-0001');
  });

  it('lit le numéro de devis pour un devis', () => {
    expect(documentNumberOf(estimate, DOCUMENT_KINDS.ESTIMATE)).toBe('DEV-STARKINDUS-2026-0001');
  });
});

describe('buildSubject', () => {
  it('distingue les trois objets', () => {
    expect(buildSubject(DOCUMENT_KINDS.INVOICE, 'FAC-1', 'Asgard Solutions'))
      .toBe('Facture FAC-1 - Asgard Solutions');
    expect(buildSubject(DOCUMENT_KINDS.REMINDER, 'FAC-1', 'Asgard Solutions'))
      .toBe('Rappel : Facture impayée FAC-1 - Asgard Solutions');
    expect(buildSubject(DOCUMENT_KINDS.ESTIMATE, 'DEV-1', 'Asgard Solutions'))
      .toBe('Devis DEV-1 - Asgard Solutions');
  });
});

describe('pickTemplate', () => {
  it('préfère le gabarit personnalisé', () => {
    expect(pickTemplate(DOCUMENT_KINDS.INVOICE, settingsWithTemplates, defaults))
      .toBe(settingsWithTemplates.emailTemplateInvoice);
  });

  it('retombe sur le gabarit par défaut quand le réglage est vide', () => {
    const blank = { emailTemplateInvoice: '', emailTemplateEstimate: '', emailTemplateReminder: '' };

    expect(pickTemplate(DOCUMENT_KINDS.INVOICE, blank, defaults))
      .toBe('Facture par défaut {documentNumber}');
    expect(pickTemplate(DOCUMENT_KINDS.REMINDER, blank, defaults))
      .toBe('Relance par défaut {documentNumber}');
    expect(pickTemplate(DOCUMENT_KINDS.ESTIMATE, blank, defaults))
      .toBe('Devis par défaut {documentNumber}');
  });
});

describe('buildTemplateData', () => {
  it('formate le montant à deux décimales et la date à la française', () => {
    const data = buildTemplateData(invoices[1], DOCUMENT_KINDS.INVOICE, clients[0], standardSettings);

    expect(data.amountTotal).toBe('9720.00');
    expect(data.documentDate).toBe('05/02/2026');
    expect(data.clientName).toBe('Stark Industries');
    expect(data.senderCompany).toBe('Asgard Solutions');
  });

  it('arrondit un montant flottant au centime', () => {
    const data = buildTemplateData(invoices[2], DOCUMENT_KINDS.INVOICE, clients[1], standardSettings);
    expect(data.amountTotal).toBe('2279.99');
  });

  it('laisse l’échéance vide, faute de champ en base', () => {
    const data = buildTemplateData(invoices[1], DOCUMENT_KINDS.INVOICE, clients[0], standardSettings);
    expect(data.dueDate).toBe('');
  });

  it('tolère un client supprimé', () => {
    const data = buildTemplateData(invoices[1], DOCUMENT_KINDS.INVOICE, null, standardSettings);
    expect(data.clientName).toBe('');
  });
});

describe('buildEmailDraft', () => {
  it('compose un envoi de facture', () => {
    const draft = buildEmailDraft(
      invoices[1], DOCUMENT_KINDS.INVOICE, clients[0], settingsWithTemplates, defaults,
    );

    expect(draft.to).toBe('pepper@stark.com');
    expect(draft.subject).toBe('Facture FAC-STARKINDUS-2026-0001 - Asgard Solutions');
    expect(draft.text).toBe(
      'Bonjour Stark Industries,\n\nFacture FAC-STARKINDUS-2026-0001 : 9720.00 €.\n\nThor Odinson\nAsgard Solutions',
    );
  });

  it('compose une relance sur une facture impayée', () => {
    const unpaid = invoices.find((invoice) => invoice.status === 'envoyee');
    const draft = buildEmailDraft(
      unpaid, DOCUMENT_KINDS.REMINDER, clients[1], settingsWithTemplates, defaults,
    );

    expect(draft.subject).toBe('Rappel : Facture impayée FAC-WAYNEENTER-2026-0005 - Asgard Solutions');
    expect(draft.text).toBe(
      'Bonjour Wayne Enterprises, la facture FAC-WAYNEENTER-2026-0005 du 11/06/2026 reste impayée.',
    );
  });

  it('compose un envoi de devis', () => {
    const draft = buildEmailDraft(
      estimates[0], DOCUMENT_KINDS.ESTIMATE, clients[0], settingsWithTemplates, defaults,
    );

    expect(draft.subject).toBe('Devis DEV-STARKINDUS-2026-0001 - Asgard Solutions');
    expect(draft.text).toBe('Bonjour Stark Industries, voici le devis DEV-STARKINDUS-2026-0001.');
  });

  it('laisse le destinataire vide quand le client a été supprimé', () => {
    const draft = buildEmailDraft(
      invoices[1], DOCUMENT_KINDS.INVOICE, null, settingsWithTemplates, defaults,
    );
    expect(draft.to).toBe('');
  });
});
