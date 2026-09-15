import Dexie from 'dexie';
import { yearBounds, buildDocumentNumber } from './domain/numbering';

export const db = new Dexie('AsgardCRMDatabase');

// Define database schema with version 1 and 2 (migration)
db.version(1).stores({
  clients: '++id, companyName, contactName, email, phone, address, createdAt',
  invoices: '++id, clientId, invoiceNumber, companyName, serviceType, description, amountHt, tvaRate, amountTva, amountTotal, date'
});

db.version(2).stores({
  clients: '++id, companyName, contactName, email, phone, address, createdAt',
  invoices: '++id, clientId, invoiceNumber, companyName, serviceType, description, amountHt, tvaRate, amountTva, amountTotal, date, status, paymentDate, paymentMethod',
  estimates: '++id, clientId, estimateNumber, companyName, serviceType, description, amountHt, tvaRate, amountTva, amountTotal, date, status',
  expenses: '++id, date, merchant, category, amount, description, paymentMethod'
}).upgrade(tx => {
  // Set default status to 'payee' for existing invoices (so previous data is considered paid by default)
  return tx.invoices.toCollection().modify(invoice => {
    if (!invoice.status) {
      invoice.status = 'payee';
      invoice.paymentDate = invoice.date;
      invoice.paymentMethod = 'virement';
    }
  });
});

/**
 * Generates an automatic invoice number in the format: FAC-CLIENT-ANNEE-000X
 * @param {string} companyName - Name of the client company
 * @param {string} dateStr - ISO Date string of the invoice
 * @returns {Promise<string>} The generated invoice number
 */
export async function generateInvoiceNumber(companyName, dateStr) {
  const { start, end } = yearBounds(dateStr);

  const count = await db.invoices
    .where('date')
    .between(start, end, true, true)
    .count();

  return buildDocumentNumber('invoice', companyName, dateStr, count);
}

/**
 * Generates an automatic estimate number in the format: DEV-CLIENT-ANNEE-000X
 * @param {string} companyName - Name of the client company
 * @param {string} dateStr - ISO Date string of the estimate
 * @returns {Promise<string>} The generated estimate number
 */
export async function generateEstimateNumber(companyName, dateStr) {
  const { start, end } = yearBounds(dateStr);

  const count = await db.estimates
    .where('date')
    .between(start, end, true, true)
    .count();

  return buildDocumentNumber('estimate', companyName, dateStr, count);
}

/**
 * Exports all database data to a single object.
 * @returns {Promise<object>} The exported data
 */
export async function exportDatabaseData() {
  return {
    clients: await db.clients.toArray(),
    invoices: await db.invoices.toArray(),
    estimates: await db.estimates.toArray(),
    expenses: await db.expenses.toArray(),
    version: db.verno,
    exportedAt: new Date().toISOString(),
  };
}

/**
 * Clears current database and imports data.
 * @param {object} data - The data to import
 * @returns {Promise<void>}
 */
export async function importDatabaseData(data) {
  if (!data || typeof data !== 'object') {
    throw new Error("Données de sauvegarde invalides.");
  }

  // Execute import inside a read-write transaction for atomicity
  await db.transaction('rw', [db.clients, db.invoices, db.estimates, db.expenses], async () => {
    // 1. Clear existing tables
    await db.clients.clear();
    await db.invoices.clear();
    await db.estimates.clear();
    await db.expenses.clear();

    // 2. Repopulate with imported data if arrays exist
    if (Array.isArray(data.clients)) {
      await db.clients.bulkAdd(data.clients);
    }
    if (Array.isArray(data.invoices)) {
      await db.invoices.bulkAdd(data.invoices);
    }
    if (Array.isArray(data.estimates)) {
      await db.estimates.bulkAdd(data.estimates);
    }
    if (Array.isArray(data.expenses)) {
      await db.expenses.bulkAdd(data.expenses);
    }
  });
}

