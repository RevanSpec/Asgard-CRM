/**
 * Validation des formulaires.
 *
 * Extrait de `App.jsx` (validateClientForm, validateInvoiceForm,
 * validateEstimateForm, validateExpenseForm, handlePhoneInput). Les messages
 * d'erreur sont repris mot pour mot : ils sont visibles par l'utilisateur et
 * les tests les figent.
 */

export const EMAIL_REGEX = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;

/**
 * Adresse française : 1 à 4 chiffres, un type de voie, puis un libellé.
 *
 * ⚠️ Restrictif — refuse les adresses sans numéro, les « 12 bis rue … », les
 * lieux-dits et les types de voie absents de la liste (cours, passage, villa,
 * hameau, résidence…). À revoir, mais hors périmètre de la phase 0 : le
 * comportement actuel est figé tel quel.
 */
export const ADDRESS_REGEX = /^\d{1,4}\s+(?:rue|boulevard|bd|avenue|av|place|impasse|route|chemin|allée|voie|square|quai)\s+.+/i;

export const PHONE_DIGITS = 10;

/** Formate une saisie de téléphone en « xx xx xx xx xx ». */
export function formatPhoneInput(raw) {
  const digits = raw.replace(/\D/g, '').slice(0, PHONE_DIGITS);
  const pairs = digits.match(/(\d{1,2})/g);
  return pairs ? pairs.join(' ') : digits;
}

export function validateClientForm(form) {
  const errors = {};

  if (!form.companyName.trim()) errors.companyName = "Nom d'entreprise requis";
  if (!form.contactName.trim()) errors.contactName = 'Nom du contact requis';

  if (!form.email.trim()) {
    errors.email = 'Email requis';
  } else if (!EMAIL_REGEX.test(form.email)) {
    errors.email = 'Format email invalide (ex: client@domaine.fr)';
  }

  const cleanPhone = form.phone.replace(/\s/g, '');
  if (!form.phone.trim()) {
    errors.phone = 'Numéro de téléphone requis';
  } else if (cleanPhone.length !== PHONE_DIGITS) {
    errors.phone = 'Le numéro doit faire exactement 10 chiffres';
  }

  if (!form.address.trim()) {
    errors.address = 'Adresse requise';
  } else if (!ADDRESS_REGEX.test(form.address)) {
    errors.address = "Format invalide. Ex: '12 rue de Paris' (1-4 chiffres + rue/bd/av/place/impasse/route/chemin/allée/voie/square/quai + nom)";
  }

  return errors;
}

export function validateInvoiceForm(form) {
  const errors = {};

  if (!form.clientId) errors.clientId = 'Client requis';
  if (!form.description.trim()) errors.description = 'Description de la prestation requise';

  const amount = parseFloat(form.amountHt);
  if (!form.amountHt || Number.isNaN(amount) || amount <= 0) {
    errors.amountHt = 'Veuillez entrer un montant HT valide supérieur à 0';
  }

  return errors;
}

export function validateEstimateForm(form) {
  const errors = {};

  if (!form.clientId) errors.clientId = 'Client requis';
  if (!form.description.trim()) errors.description = 'Description requise';

  const amount = parseFloat(form.amountHt);
  if (!form.amountHt || Number.isNaN(amount) || amount <= 0) {
    errors.amountHt = 'Montant HT valide requis (> 0)';
  }

  return errors;
}

export function validateExpenseForm(form) {
  const errors = {};

  if (!form.merchant.trim()) errors.merchant = 'Fournisseur requis';

  const amount = parseFloat(form.amount);
  if (!form.amount || Number.isNaN(amount) || amount <= 0) {
    errors.amount = 'Montant supérieur à 0 requis';
  }

  return errors;
}

/** Un formulaire est valide quand aucune erreur n'a été relevée. */
export function isValid(errors) {
  return Object.keys(errors).length === 0;
}
