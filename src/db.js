import Dexie from 'dexie';

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
  const sanitizedClient = companyName
    .toUpperCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "") // Remove accents
    .replace(/[^A-Z0-9]/g, "") // Keep only alphanumeric
    .substring(0, 10); // Keep max 10 characters for readability
  
  const year = new Date(dateStr).getFullYear().toString();
  
  // Parse year boundaries
  const startOfYear = new Date(`${year}-01-01T00:00:00Z`).toISOString();
  const endOfYear = new Date(`${year}-12-31T23:59:59Z`).toISOString();
  
  // Count how many invoices exist in the system for this year
  const count = await db.invoices
    .where('date')
    .between(startOfYear, endOfYear, true, true)
    .count();
    
  const sequence = (count + 1).toString().padStart(4, '0');
  
  return `FAC-${sanitizedClient}-${year}-${sequence}`;
}

/**
 * Generates an automatic estimate number in the format: DEV-CLIENT-ANNEE-000X
 * @param {string} companyName - Name of the client company
 * @param {string} dateStr - ISO Date string of the estimate
 * @returns {Promise<string>} The generated estimate number
 */
export async function generateEstimateNumber(companyName, dateStr) {
  const sanitizedClient = companyName
    .toUpperCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "") // Remove accents
    .replace(/[^A-Z0-9]/g, "") // Keep only alphanumeric
    .substring(0, 10); // Keep max 10 characters for readability
  
  const year = new Date(dateStr).getFullYear().toString();
  
  // Parse year boundaries
  const startOfYear = new Date(`${year}-01-01T00:00:00Z`).toISOString();
  const endOfYear = new Date(`${year}-12-31T23:59:59Z`).toISOString();
  
  // Count how many estimates exist in the system for this year
  const count = await db.estimates
    .where('date')
    .between(startOfYear, endOfYear, true, true)
    .count();
    
  const sequence = (count + 1).toString().padStart(4, '0');
  
  return `DEV-${sanitizedClient}-${year}-${sequence}`;
}
