import Dexie from 'dexie';

export const db = new Dexie('AsgardCRMDatabase');

// Define database schema
db.version(1).stores({
  clients: '++id, companyName, contactName, email, phone, address, createdAt',
  invoices: '++id, clientId, invoiceNumber, companyName, serviceType, description, amountHt, tvaRate, amountTva, amountTotal, date'
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
