/**
 * Accès aux données.
 *
 * Phase 2 : ce fichier n'est plus un schéma Dexie mais un adaptateur vers les
 * commandes de l'hôte Rust. Les données vivent désormais dans un fichier SQLite
 * du répertoire applicatif, que l'utilisateur peut copier et sauvegarder —
 * IndexedDB vivait dans le profil du moteur de rendu et disparaissait avec lui
 * (défaut D6).
 *
 * Les objets renvoyés gardent exactement la forme que produisait Dexie, pour
 * que les composants n'aient pas à changer. Deux différences assumées :
 *
 * - les numéros de pièce sont attribués par la base, plus par l'appelant. La
 *   fonction `generateInvoiceNumber` a disparu : elle portait le défaut D3 ;
 * - supprimer une facture émise l'archive au lieu de l'effacer (défaut D4), et
 *   les fonctions de suppression rendent compte de ce qu'elles ont fait.
 */
import { invoke } from '@tauri-apps/api/core';
import { isDesktop } from './ipc';

/**
 * Hors coquille de bureau (`npm run dev:vite`), il n'y a pas d'hôte donc pas de
 * base. L'interface s'affiche avec un jeu vide plutôt que de planter, ce qui
 * reste pratique pour travailler la mise en page. Toute écriture est ignorée.
 */
const EMPTY_SNAPSHOT = { clients: [], invoices: [], estimates: [], expenses: [] };

let warned = false;

function unavailable(operation) {
  if (!warned) {
    warned = true;
    console.warn(
      "Asgard CRM tourne sans son hôte : les données ne sont ni lues ni enregistrées. " +
      "Lancez `npm run dev` pour la version complète."
    );
  }
  console.warn(`Opération ignorée hors coquille de bureau : ${operation}`);
}

async function call(command, args, fallback) {
  if (!isDesktop()) {
    unavailable(command);
    return fallback;
  }
  return invoke(command, args);
}

// ------------------------------------------------------------------ lecture

/** Charge les quatre tables en un seul aller-retour. */
export async function loadSnapshot() {
  return call('load_snapshot', undefined, EMPTY_SNAPSHOT);
}

/** Chemin du fichier de base, à afficher dans les réglages. */
export async function databasePath() {
  return call('database_path', undefined, '');
}

// ------------------------------------------------------------------ clients

/** Crée ou met à jour un client selon la présence de `id`. */
export async function saveClient(client) {
  return call('save_client', { client }, null);
}

export async function deleteClient(id) {
  return call('delete_client', { id }, null);
}

// ----------------------------------------------------------------- factures

/**
 * Crée une facture. Le numéro est attribué par la base dans la transaction
 * d'insertion — l'interface ne peut plus en proposer un.
 */
export async function createInvoice(invoice) {
  return call('create_invoice', { invoice }, null);
}

export async function setInvoiceStatus(id, status) {
  return call('set_invoice_status', { id, status }, null);
}

export async function recordPayment(payment) {
  return call('record_payment', { payment }, null);
}

/**
 * Supprime des factures.
 *
 * Renvoie `{ discarded, archived }` : les brouillons sont réellement effacés,
 * les pièces émises seulement retirées de l'affichage. Une facture émise ne se
 * supprime pas — conservation dix ans, art. L123-22 du code de commerce.
 */
export async function deleteInvoices(ids) {
  return call('delete_invoices', { ids }, { discarded: 0, archived: 0 });
}

// -------------------------------------------------------------------- devis

export async function saveEstimate(estimate) {
  return call('save_estimate', { estimate }, null);
}

export async function setEstimateStatus(id, status) {
  return call('set_estimate_status', { id, status }, null);
}

export async function deleteEstimate(id) {
  return call('delete_estimate', { id }, { discarded: 0, archived: 0 });
}

/** Convertit un devis en facture ; les deux écritures sont atomiques. */
export async function convertEstimate(id) {
  return call('convert_estimate', { id }, null);
}

// ----------------------------------------------------------------- dépenses

export async function saveExpense(expense) {
  return call('save_expense', { expense }, null);
}

export async function deleteExpense(id) {
  return call('delete_expense', { id }, null);
}

// -------------------------------------------------------------- sauvegardes

export async function exportBackup() {
  return call('export_backup', undefined, EMPTY_SNAPSHOT);
}

/**
 * Reprend une sauvegarde et renvoie un compte rendu.
 *
 * `adjustments` liste les montants que l'arrondi au centime a modifiés : le
 * passage des flottants aux entiers change réellement certaines valeurs, et
 * l'utilisateur doit pouvoir l'expliquer plutôt que de le découvrir dans une
 * déclaration.
 */
export async function importBackup(backup) {
  return call('import_backup', { backup }, null);
}

/** Copie atomique du fichier de base — ce qu'IndexedDB ne permettait pas. */
export async function backupToFile(destination) {
  return call('backup_to_file', { destination }, null);
}

// --------------------------------------------------------------- calculs

/**
 * Agrégats du tableau de bord, calculés par le noyau métier Rust.
 *
 * Phase 3 : ces chiffres étaient calculés dans `App.jsx`, en virgule flottante,
 * et le calcul URSSAF y existait en trois exemplaires divergents. Ils viennent
 * désormais d'`asgard-core`, en arithmétique décimale.
 *
 * Deux bases de cotisations sont renvoyées, là où l'ancienne version en
 * mélangeait deux sans le dire : `urssafCharges` sur l'encaissé — la seule
 * déclarable — et `urssafChargesProjected` sur le facturé.
 */
export async function loadDashboard(settings, year) {
  return call('dashboard', { settings, year }, null);
}

/**
 * Déclaration URSSAF d'une période.
 * `period` vaut `{ periodType: 'monthly', year, month }` ou
 * `{ periodType: 'quarterly', year, quarter }`.
 */
export async function loadUrssafDeclaration(settings, period) {
  return call('urssaf_declaration', { settings, period }, null);
}

/** Livre des recettes au format CSV réglementaire, produit côté Rust. */
export async function loadRecettesCsv() {
  return call('recettes_csv', undefined, '');
}

// ------------------------------------------------------------ pièces et envois

/**
 * Enregistre une pièce en PDF, après le sélecteur de fichiers du système.
 *
 * Phase 4 : le document est produit par l'hôte, plus par jsPDF. Il n'est plus
 * téléchargé par la page via un `<a download>` sur une URL `blob:` — chemin
 * dont le comportement sous la CSP de Tauri n'avait jamais pu être vérifié —
 * mais écrit sur le disque côté Rust.
 *
 * Renvoie le chemin choisi, ou `null` si l'utilisateur a renoncé.
 */
export async function exportDocument(kind, id, issuer) {
  return call('export_document', { kind, id, issuer }, null);
}

/** Enregistre un contenu texte — livre des recettes, sauvegarde JSON. */
export async function exportText(contents, suggestedName, extension) {
  return call('export_text', { contents, suggestedName, extension }, null);
}

/**
 * Envoie une pièce par e-mail, pièce jointe comprise.
 *
 * Ni le PDF ni le mot de passe SMTP ne traversent la frontière : l'hôte lit la
 * pièce en base, produit le document et l'attache lui-même.
 */
export async function sendDocument(kind, id, issuer, smtpConfig, message) {
  return call(
    'send_document',
    { kind, id, issuer, smtpConfig, message },
    { success: false, error: "Envoi indisponible hors de l'application de bureau." },
  );
}

/** Adresse enregistrée du client d'une pièce, pour pré-remplir le formulaire. */
export async function documentRecipient(kind, id) {
  return call('document_recipient', { kind, id }, null);
}
