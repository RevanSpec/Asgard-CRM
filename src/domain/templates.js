/**
 * Gabarits d'e-mail : facture, devis et relance d'impayé.
 *
 * Extrait de `App.jsx` (resolveTemplate, handleOpenSendEmail). La substitution
 * est volontairement naïve — un simple remplacement de jetons `{nom}` — et le
 * restera : la phase 4 la portera à l'identique côté Rust.
 */

export const DOCUMENT_KINDS = {
  INVOICE: 'invoice',
  ESTIMATE: 'estimate',
  REMINDER: 'reminder',
};

/** Jetons reconnus dans les gabarits, dans l'ordre de substitution. */
export const TEMPLATE_TOKENS = [
  'clientName',
  'documentNumber',
  'description',
  'amountTotal',
  'dueDate',
  'documentDate',
  'senderName',
  'senderCompany',
];

/**
 * Remplace chaque jeton par sa valeur. Un jeton sans valeur est remplacé par
 * une chaîne vide plutôt que laissé en place — comportement actuel.
 */
export function resolveTemplate(template, data) {
  if (!template) return '';

  return TEMPLATE_TOKENS.reduce(
    (text, token) => text.replace(new RegExp(`{${token}}`, 'g'), data[token] || ''),
    template,
  );
}

/** Une relance porte sur une facture : elle en reprend le numéro. */
export function isInvoiceKind(kind) {
  return kind === DOCUMENT_KINDS.INVOICE || kind === DOCUMENT_KINDS.REMINDER;
}

/** Numéro de la pièce, selon qu'il s'agit d'une facture ou d'un devis. */
export function documentNumberOf(document, kind) {
  return isInvoiceKind(kind) ? document.invoiceNumber : document.estimateNumber;
}

/** Objet de l'e-mail, repris mot pour mot de l'existant. */
export function buildSubject(kind, documentNumber, companyName) {
  if (kind === DOCUMENT_KINDS.INVOICE) {
    return `Facture ${documentNumber} - ${companyName}`;
  }
  if (kind === DOCUMENT_KINDS.REMINDER) {
    return `Rappel : Facture impayée ${documentNumber} - ${companyName}`;
  }
  return `Devis ${documentNumber} - ${companyName}`;
}

/** Sélectionne le gabarit des réglages, avec repli sur le gabarit par défaut. */
export function pickTemplate(kind, settings, defaults) {
  if (kind === DOCUMENT_KINDS.INVOICE) {
    return settings.emailTemplateInvoice || defaults.emailTemplateInvoice;
  }
  if (kind === DOCUMENT_KINDS.REMINDER) {
    return settings.emailTemplateReminder || defaults.emailTemplateReminder;
  }
  return settings.emailTemplateEstimate || defaults.emailTemplateEstimate;
}

/** Valeurs de substitution pour une pièce donnée. */
export function buildTemplateData(document, kind, client, settings) {
  return {
    clientName: client ? client.companyName : '',
    documentNumber: documentNumberOf(document, kind),
    description: document.description || '',
    amountTotal: document.amountTotal.toFixed(2),
    dueDate: document.dueDate ? new Date(document.dueDate).toLocaleDateString('fr-FR') : '',
    documentDate: document.date ? new Date(document.date).toLocaleDateString('fr-FR') : '',
    senderName: settings.contactName,
    senderCompany: settings.companyName,
  };
}

/** Construit l'e-mail complet : destinataire, objet et corps. */
export function buildEmailDraft(document, kind, client, settings, defaults) {
  const data = buildTemplateData(document, kind, client, settings);

  return {
    to: client ? client.email : '',
    subject: buildSubject(kind, data.documentNumber, settings.companyName),
    text: resolveTemplate(pickTemplate(kind, settings, defaults), data),
  };
}
