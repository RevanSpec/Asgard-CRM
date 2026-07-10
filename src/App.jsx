import React, { useState, useEffect } from 'react';
import { db, generateInvoiceNumber, exportDatabaseData, importDatabaseData } from './db';
import { exportInvoiceToPDF, generateInvoicePDF, exportEstimateToPDF } from './pdfGenerator';

// Electron IPC renderer (safe for web testing too)
const ipcRenderer = window.require ? window.require('electron').ipcRenderer : null;

// --- INLINE SVG ICONS (Premium Gold Theme) ---
const Icons = {
  Dashboard: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="7" height="9" />
      <rect x="14" y="3" width="7" height="5" />
      <rect x="14" y="12" width="7" height="9" />
      <rect x="3" y="16" width="7" height="5" />
    </svg>
  ),
  Clients: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
      <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
      <path d="M16 3.13a4 4 0 0 1 0 7.75" />
    </svg>
  ),
  Invoices: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <polyline points="10 9 9 9 8 9" />
    </svg>
  ),
  Settings: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  ),
  Add: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  ),
  Edit: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
      <path d="M18.5 2.5a2.121 2.121 0 1 1 3 3L12 15l-4 1 1-4z" />
    </svg>
  ),
  Delete: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      <line x1="10" y1="11" x2="10" y2="17" />
      <line x1="14" y1="11" x2="14" y2="17" />
    </svg>
  ),
  Download: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
  ),
  Search: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
  ),
  Email: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
      <polyline points="22,6 12,13 2,6" />
    </svg>
  ),
  Estimates: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
      <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
      <path d="M9 14h6" />
      <path d="M9 18h6" />
      <path d="M9 10h6" />
    </svg>
  ),
  Expenses: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="1" y="4" width="22" height="16" rx="2" ry="2" />
      <line x1="1" y1="10" x2="23" y2="10" />
    </svg>
  ),
  Accounting: () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="18" y1="20" x2="18" y2="10" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="6" y1="20" x2="6" y2="14" />
    </svg>
  )
};

// Default business details in localstorage if empty
const defaultSettings = {
  companyName: 'Asgard Solutions',
  contactName: 'Thor Odinson',
  email: 'thor@asgard-solutions.fr',
  phone: '06 12 34 56 78',
  address: '1 Rue du Valhalla, 75008 Paris',
  siret: '839 204 123 00019',
  iban: 'FR76 3000 2000 0001 2345 6789 012',
  urssafServiceBnc: 21.1,
  urssafServiceBic: 21.1,
  urssafVente: 12.3,
  acreEnabled: false,
  smtpHost: '127.0.0.1',
  smtpPort: '1025',
  smtpUser: '',
  smtpPass: '',
  smtpSecure: 'none',
  customColor: '#E5A93C',
  logoBase64: ''
};

// Regex declarations
const EMAIL_REGEX = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
// Format: Starts with 1-4 digits, space, street type (rue/bd/av/...), space, then name
const ADDRESS_REGEX = /^\d{1,4}\s+(?:rue|boulevard|bd|avenue|av|place|impasse|route|chemin|allée|voie|square|quai)\s+.+/i;

export default function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [clients, setClients] = useState([]);
  const [invoices, setInvoices] = useState([]);
  const [businessSettings, setBusinessSettings] = useState(defaultSettings);

  // Search filter states
  const [clientSearch, setClientSearch] = useState('');
  const [invoiceSearch, setInvoiceSearch] = useState('');
  const [selectedInvoiceIds, setSelectedInvoiceIds] = useState([]);

  // Reset selected invoices on tab change
  useEffect(() => {
    setSelectedInvoiceIds([]);
  }, [activeTab]);

  // Email states
  const [emailModalOpen, setEmailModalOpen] = useState(false);
  const [emailForm, setEmailForm] = useState({ to: '', subject: '', text: '', invoice: null, type: 'invoice' });
  const [sendingEmail, setSendingEmail] = useState(false);
  const [smtpTesting, setSmtpTesting] = useState(false);

  // Estimates, Expenses and Compta States
  const [estimates, setEstimates] = useState([]);
  const [expenses, setExpenses] = useState([]);
  
  const [estimatesSearch, setEstimatesSearch] = useState('');
  const [expensesSearch, setExpensesSearch] = useState('');
  
  const [estimateModalOpen, setEstimateModalOpen] = useState(false);
  const [expenseModalOpen, setExpenseModalOpen] = useState(false);
  const [paymentModalOpen, setPaymentModalOpen] = useState(false);
  
  const [estimateForm, setEstimateForm] = useState({ id: null, clientId: '', serviceType: 'service_bnc', description: '', amountHt: '', tvaRate: '20', status: 'brouillon', date: new Date().toISOString().split('T')[0] });
  const [estimateFormErrors, setEstimateFormErrors] = useState({});
  
  const [expenseForm, setExpenseForm] = useState({ id: null, merchant: '', category: 'Logiciels', amount: '', description: '', paymentMethod: 'carte', date: new Date().toISOString().split('T')[0] });
  const [expenseFormErrors, setExpenseFormErrors] = useState({});
  
  const [paymentForm, setPaymentForm] = useState({ invoiceId: null, invoiceNumber: '', paymentDate: new Date().toISOString().split('T')[0], paymentMethod: 'virement' });
  
  const [periodType, setPeriodType] = useState('monthly');
  const [selectedYear, setSelectedYear] = useState(new Date().getFullYear());
  const [selectedMonth, setSelectedMonth] = useState(new Date().getMonth() + 1); // 1-12
  const [selectedQuarter, setSelectedQuarter] = useState(Math.floor(new Date().getMonth() / 3) + 1); // 1-4
  const [comptaActiveTab, setComptaActiveTab] = useState('recettes');

  // Modals States
  const [clientModalOpen, setClientModalOpen] = useState(false);
  const [invoiceModalOpen, setInvoiceModalOpen] = useState(false);

  // Form states
  const [clientForm, setClientForm] = useState({ id: null, companyName: '', contactName: '', email: '', phone: '', address: '' });
  const [clientFormErrors, setClientFormErrors] = useState({});

  const [invoiceForm, setInvoiceForm] = useState({ clientId: '', serviceType: 'service_bnc', description: '', amountHt: '', tvaRate: '20' });
  const [invoiceFormErrors, setInvoiceFormErrors] = useState({});

  // Custom alert & confirm states
  const [customAlert, setCustomAlert] = useState({ open: false, title: '', message: '' });
  const [customConfirm, setCustomConfirm] = useState({ open: false, title: '', message: '', onConfirm: null });

  const showAlert = (title, message) => {
    setCustomAlert({ open: true, title, message });
  };
  const showConfirm = (title, message, onConfirm) => {
    setCustomConfirm({ open: true, title, message, onConfirm });
  };
  const closeAlert = () => setCustomAlert({ ...customAlert, open: false });
  const closeConfirm = () => setCustomConfirm({ ...customConfirm, open: false });

  // Seed database if empty
  const seedDatabase = async () => {
    const clientsCount = await db.clients.count();
    if (clientsCount === 0) {
      // Seed clients
      const starkId = await db.clients.add({
        companyName: 'Stark Industries',
        contactName: 'Pepper Potts',
        email: 'pepper@stark.com',
        phone: '06 11 22 33 44',
        address: '108 Route de Malibu, 75008 Paris',
        createdAt: new Date('2026-01-10').toISOString()
      });
      const wayneId = await db.clients.add({
        companyName: 'Wayne Enterprises',
        contactName: 'Lucius Fox',
        email: 'lucius@wayne.com',
        phone: '07 88 99 00 11',
        address: '12 Avenue de Gotham, 75016 Paris',
        createdAt: new Date('2026-02-15').toISOString()
      });
      const asgardId = await db.clients.add({
        companyName: 'Asgard Coffee',
        contactName: 'Valkyrie',
        email: 'valk@coffee.asgard',
        phone: '06 55 55 55 55',
        address: '45 Rue du Bifrost, 75011 Paris',
        createdAt: new Date('2026-03-20').toISOString()
      });

      // Seed invoices spread over months for beautiful Dashboard metrics & charts
      const invoicesSeed = [
        {
          clientId: starkId,
          companyName: 'Stark Industries',
          invoiceNumber: 'FAC-STARK-2026-0001',
          serviceType: 'service_bnc',
          description: 'Consulting en nanotechnologies',
          amountHt: 6000,
          tvaRate: 20,
          amountTva: 1200,
          amountTotal: 7200,
          date: new Date('2026-02-05').toISOString(),
          status: 'payee',
          paymentDate: new Date('2026-02-05').toISOString(),
          paymentMethod: 'virement'
        },
        {
          clientId: wayneId,
          companyName: 'Wayne Enterprises',
          invoiceNumber: 'FAC-WAYNE-2026-0002',
          serviceType: 'service_bnc',
          description: 'Audit système de défense sonar',
          amountHt: 12500,
          tvaRate: 20,
          amountTva: 2500,
          amountTotal: 15000,
          date: new Date('2026-03-12').toISOString(),
          status: 'payee',
          paymentDate: new Date('2026-03-12').toISOString(),
          paymentMethod: 'virement'
        },
        {
          clientId: asgardId,
          companyName: 'Asgard Coffee',
          invoiceNumber: 'FAC-ASGARDCOF-2026-0003',
          serviceType: 'vente',
          description: 'Livraison de grains de café d\'Éthiopie',
          amountHt: 1400,
          tvaRate: 5.5,
          amountTva: 77,
          amountTotal: 1477,
          date: new Date('2026-04-18').toISOString(),
          status: 'payee',
          paymentDate: new Date('2026-04-18').toISOString(),
          paymentMethod: 'virement'
        },
        {
          clientId: starkId,
          companyName: 'Stark Industries',
          invoiceNumber: 'FAC-STARK-2026-0004',
          serviceType: 'service_bnc',
          description: 'Optimisation de l\'IA Jarvis',
          amountHt: 4500,
          tvaRate: 20,
          amountTva: 900,
          amountTotal: 5400,
          date: new Date('2026-05-02').toISOString(),
          status: 'payee',
          paymentDate: new Date('2026-05-02').toISOString(),
          paymentMethod: 'virement'
        },
        {
          clientId: wayneId,
          companyName: 'Wayne Enterprises',
          invoiceNumber: 'FAC-WAYNE-2026-0005',
          serviceType: 'service_bnc',
          description: 'Développement application de traque',
          amountHt: 9000,
          tvaRate: 20,
          amountTva: 1800,
          amountTotal: 10800,
          date: new Date('2026-06-25').toISOString(),
          status: 'payee',
          paymentDate: new Date('2026-06-25').toISOString(),
          paymentMethod: 'virement'
        },
        {
          clientId: asgardId,
          companyName: 'Asgard Coffee',
          invoiceNumber: 'FAC-ASGARDCOF-2026-0006',
          serviceType: 'vente',
          description: 'Achat de machines expresso professionnelles',
          amountHt: 2800,
          tvaRate: 20,
          amountTva: 560,
          amountTotal: 3360,
          date: new Date('2026-07-01').toISOString(),
          status: 'envoyee'
        }
      ];

      for (const inv of invoicesSeed) {
        await db.invoices.add(inv);
      }

      // Seed Estimates
      await db.estimates.add({
        clientId: starkId,
        companyName: 'Stark Industries',
        estimateNumber: 'DEV-STARK-2026-0001',
        serviceType: 'service_bnc',
        description: 'Déploiement infrastructure IA Jarvis',
        amountHt: 8000,
        tvaRate: 20,
        amountTva: 1600,
        amountTotal: 9600,
        date: new Date('2026-06-10').toISOString(),
        status: 'accepte'
      });
      await db.estimates.add({
        clientId: wayneId,
        companyName: 'Wayne Enterprises',
        estimateNumber: 'DEV-WAYNE-2026-0002',
        serviceType: 'service_bic',
        description: 'Maintenance des capteurs sonar',
        amountHt: 3500,
        tvaRate: 20,
        amountTva: 700,
        amountTotal: 4200,
        date: new Date('2026-07-02').toISOString(),
        status: 'brouillon'
      });

      // Seed Expenses
      await db.expenses.add({
        date: new Date('2026-02-10').toISOString(),
        merchant: 'OVHcloud',
        category: 'Logiciels',
        amount: 49.99,
        description: 'Hébergement VPS Asgard CRM',
        paymentMethod: 'carte'
      });
      await db.expenses.add({
        date: new Date('2026-03-01').toISOString(),
        merchant: 'Adobe Creative Cloud',
        category: 'Logiciels',
        amount: 35.99,
        description: 'Abonnement Photoshop/Illustrator',
        paymentMethod: 'carte'
      });
      await db.expenses.add({
        date: new Date('2026-04-05').toISOString(),
        merchant: 'SNCF',
        category: 'Déplacements',
        amount: 120.00,
        description: 'Trajet Paris-Lyon rendez-vous client',
        paymentMethod: 'carte'
      });
    }
  };

  // Fetch all data
  const loadAllData = async () => {
    const clientsList = await db.clients.toArray();
    const invoicesList = await db.invoices.toArray();
    const estimatesList = await db.estimates.toArray();
    const expensesList = await db.expenses.toArray();
    
    // Sort clients by name
    setClients(clientsList.sort((a, b) => a.companyName.localeCompare(b.companyName)));
    // Sort invoices by date descending
    setInvoices(invoicesList.sort((a, b) => new Date(b.date) - new Date(a.date)));
    // Sort estimates by date descending
    setEstimates(estimatesList.sort((a, b) => new Date(b.date) - new Date(a.date)));
    // Sort expenses by date descending
    setExpenses(expensesList.sort((a, b) => new Date(b.date) - new Date(a.date)));
  };

  // Load Settings from LocalStorage
  useEffect(() => {
    const saved = localStorage.getItem('asgard_crm_settings');
    if (saved) {
      setBusinessSettings({ ...defaultSettings, ...JSON.parse(saved) });
    } else {
      localStorage.setItem('asgard_crm_settings', JSON.stringify(defaultSettings));
    }

    const init = async () => {
      await seedDatabase();
      await loadAllData();
    };
    init();
  }, []);

  // Save Settings to LocalStorage & DB helper
  const saveSettings = (newSettings) => {
    setBusinessSettings(newSettings);
    localStorage.setItem('asgard_crm_settings', JSON.stringify(newSettings));
  };

  // Export all DB tables and settings as a JSON file backup
  const handleExportBackup = async () => {
    try {
      const dbData = await exportDatabaseData();
      const settings = localStorage.getItem('asgard_crm_settings');
      const backup = {
        db: dbData,
        settings: settings ? JSON.parse(settings) : defaultSettings,
        backupVersion: 1,
      };

      const jsonString = `data:text/json;charset=utf-8,${encodeURIComponent(JSON.stringify(backup, null, 2))}`;
      const downloadAnchor = document.createElement('a');
      downloadAnchor.setAttribute("href", jsonString);

      const dateStr = new Date().toISOString().split('T')[0];
      downloadAnchor.setAttribute("download", `asgard_crm_backup_${dateStr}.json`);
      document.body.appendChild(downloadAnchor);
      downloadAnchor.click();
      downloadAnchor.remove();

      showAlert("Succès", "Sauvegarde exportée avec succès !");
    } catch (error) {
      console.error(error);
      showAlert("Erreur", "Échec de l'exportation de la sauvegarde : " + error.message);
    }
  };

  // Import a JSON file backup and reload DB state
  const handleImportBackup = async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    const confirmImport = window.confirm("Êtes-vous sûr de vouloir importer cette sauvegarde ? Cette action écrasera TOUTES les données actuelles de l'application (clients, factures, devis, dépenses et paramètres).");
    if (!confirmImport) {
      e.target.value = null;
      return;
    }

    try {
      const reader = new FileReader();
      reader.onload = async (event) => {
        try {
          const backup = JSON.parse(event.target.result);

          if (!backup || (!backup.db && !backup.clients)) {
            throw new Error("Le fichier importé n'est pas une sauvegarde Asgard CRM valide.");
          }

          const dbData = backup.db || backup;
          const settingsData = backup.settings || null;

          // 1. Import database tables
          await importDatabaseData(dbData);

          // 2. Restore settings
          if (settingsData) {
            saveSettings(settingsData);
          }

          // 3. Reload state
          await loadAllData();

          showAlert("Succès", "Sauvegarde restaurée avec succès ! Toutes les données ont été mises à jour.");
        } catch (innerError) {
          console.error(innerError);
          showAlert("Erreur", "Erreur lors du traitement du fichier de sauvegarde : " + innerError.message);
        }
      };
      reader.readAsText(file);
    } catch (error) {
      console.error(error);
      showAlert("Erreur", "Impossible de lire le fichier de sauvegarde : " + error.message);
    } finally {
      e.target.value = null; // Clear file input
    }
  };

  // Format Phone dynamically as 'xx xx xx xx xx'
  const handlePhoneInput = (e) => {
    const clean = e.target.value.replace(/\D/g, '').slice(0, 10);
    const match = clean.match(/(\d{1,2})/g);
    const formatted = match ? match.join(' ') : clean;
    setClientForm({ ...clientForm, phone: formatted });
  };

  // --- CLIENT ACTIONS ---

  const handleOpenAddClient = () => {
    setClientForm({ id: null, companyName: '', contactName: '', email: '', phone: '', address: '' });
    setClientFormErrors({});
    setClientModalOpen(true);
  };

  const handleOpenEditClient = (client) => {
    setClientForm(client);
    setClientFormErrors({});
    setClientModalOpen(true);
  };

  const validateClientForm = () => {
    const errors = {};
    if (!clientForm.companyName.trim()) errors.companyName = "Nom d'entreprise requis";
    if (!clientForm.contactName.trim()) errors.contactName = "Nom du contact requis";
    
    if (!clientForm.email.trim()) {
      errors.email = "Email requis";
    } else if (!EMAIL_REGEX.test(clientForm.email)) {
      errors.email = "Format email invalide (ex: client@domaine.fr)";
    }

    const cleanPhone = clientForm.phone.replace(/\s/g, '');
    if (!clientForm.phone.trim()) {
      errors.phone = "Numéro de téléphone requis";
    } else if (cleanPhone.length !== 10) {
      errors.phone = "Le numéro doit faire exactement 10 chiffres";
    }

    if (!clientForm.address.trim()) {
      errors.address = "Adresse requise";
    } else if (!ADDRESS_REGEX.test(clientForm.address)) {
      errors.address = "Format invalide. Ex: '12 rue de Paris' (1-4 chiffres + rue/bd/av/place/impasse/route/chemin/allée/voie/square/quai + nom)";
    }

    setClientFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSaveClient = async (e) => {
    e.preventDefault();
    if (!validateClientForm()) return;

    try {
      if (clientForm.id) {
        // Edit
        await db.clients.update(clientForm.id, {
          companyName: clientForm.companyName,
          contactName: clientForm.contactName,
          email: clientForm.email,
          phone: clientForm.phone,
          address: clientForm.address
        });
        
        // Update companyName in all existing invoices of this client
        await db.invoices.where('clientId').equals(clientForm.id).modify({
          companyName: clientForm.companyName
        });
      } else {
        // Add
        await db.clients.add({
          companyName: clientForm.companyName,
          contactName: clientForm.contactName,
          email: clientForm.email,
          phone: clientForm.phone,
          address: clientForm.address,
          createdAt: new Date().toISOString()
        });
      }
      setClientModalOpen(false);
      await loadAllData();
    } catch (err) {
      console.error(err);
    }
  };

  const handleDeleteClient = async (id) => {
    showConfirm(
      "Supprimer le client",
      "Êtes-vous sûr de vouloir supprimer ce client ? Toutes ses factures associées resteront dans l'historique mais déconnectées.",
      async () => {
        await db.clients.delete(id);
        await loadAllData();
      }
    );
  };

  // --- INVOICE ACTIONS ---

  const handleOpenAddInvoice = () => {
    if (clients.length === 0) {
      showAlert("Client requis", "Veuillez d'abord créer au moins un client avant de générer une facture.");
      return;
    }
    setInvoiceForm({
      clientId: clients[0].id.toString(),
      serviceType: 'service_bnc',
      description: '',
      amountHt: '',
      tvaRate: '20'
    });
    setInvoiceFormErrors({});
    setInvoiceModalOpen(true);
  };

  const validateInvoiceForm = () => {
    const errors = {};
    if (!invoiceForm.clientId) errors.clientId = "Client requis";
    if (!invoiceForm.description.trim()) errors.description = "Description de la prestation requise";
    
    const htVal = parseFloat(invoiceForm.amountHt);
    if (!invoiceForm.amountHt || isNaN(htVal) || htVal <= 0) {
      errors.amountHt = "Veuillez entrer un montant HT valide supérieur à 0";
    }

    setInvoiceFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSaveInvoice = async (e) => {
    e.preventDefault();
    if (!validateInvoiceForm()) return;

    const selectedClient = clients.find(c => c.id === parseInt(invoiceForm.clientId));
    const amountHt = parseFloat(invoiceForm.amountHt);
    const tvaRate = parseFloat(invoiceForm.tvaRate);
    const amountTva = (amountHt * tvaRate) / 100;
    const amountTotal = amountHt + amountTva;
    const date = new Date().toISOString();

    // Auto-generate invoice number
    const invoiceNumber = await generateInvoiceNumber(selectedClient.companyName, date);

    try {
      await db.invoices.add({
        clientId: selectedClient.id,
        companyName: selectedClient.companyName,
        invoiceNumber,
        serviceType: invoiceForm.serviceType,
        description: invoiceForm.description,
        amountHt,
        tvaRate,
        amountTva,
        amountTotal,
        date
      });

      setInvoiceModalOpen(false);
      await loadAllData();
    } catch (err) {
      console.error(err);
    }
  };

  const handleDeleteInvoice = async (id) => {
    showConfirm(
      "Supprimer la facture",
      "Êtes-vous sûr de vouloir supprimer cette facture ? Cette action est irréversible.",
      async () => {
        await db.invoices.delete(id);
        setSelectedInvoiceIds(prev => prev.filter(item => item !== id));
        await loadAllData();
      }
    );
  };

  const handleDeleteSelectedInvoices = () => {
    if (selectedInvoiceIds.length === 0) return;
    showConfirm(
      "Supprimer les factures sélectionnées",
      `Êtes-vous sûr de vouloir supprimer les ${selectedInvoiceIds.length} factures sélectionnées ? Cette action est irréversible.`,
      async () => {
        try {
          await db.invoices.bulkDelete(selectedInvoiceIds);
          setSelectedInvoiceIds([]);
          await loadAllData();
        } catch (err) {
          console.error(err);
        }
      }
    );
  };

  const handleOpenSendEmail = async (invoice, type = 'invoice') => {
    // Get client email
    let client = await db.clients.get(invoice.clientId);
    const toEmail = client ? client.email : '';
    const isInvoice = type === 'invoice';
    const num = isInvoice ? invoice.invoiceNumber : invoice.estimateNumber;
    const docName = isInvoice ? 'la facture' : 'le devis';
    
    setEmailForm({
      to: toEmail,
      subject: `${isInvoice ? 'Facture' : 'Devis'} ${num} - ${businessSettings.companyName}`,
      text: `Bonjour,\n\nVeuillez trouver ci-joint ${docName} ${num} pour la prestation : ${invoice.description}.\n\nLe montant total est de ${invoice.amountTotal.toFixed(2)} €.\n\nCordialement,\n\n${businessSettings.contactName}\n${businessSettings.companyName}`,
      invoice: invoice,
      type: type
    });
    
    setEmailModalOpen(true);
  };

  const handleSendEmail = async (e) => {
    e.preventDefault();
    if (!ipcRenderer) {
      showAlert("Erreur", "L'envoi d'e-mails n'est disponible que dans la version de bureau de l'application.");
      return;
    }

    if (!emailForm.to.trim()) {
      showAlert("Erreur", "L'adresse e-mail du destinataire est requise.");
      return;
    }

    setSendingEmail(true);

    try {
      const invoice = emailForm.invoice;
      const isInvoice = emailForm.type === 'invoice';
      let client = await db.clients.get(invoice.clientId);
      if (!client) {
        client = {
          companyName: invoice.companyName,
          contactName: 'Client',
          email: emailForm.to,
          phone: '',
          address: ''
        };
      }

      // 1. Generate PDF document and get base64 string
      const doc = isInvoice 
        ? generateInvoicePDF(invoice, client, businessSettings)
        : generateEstimatePDF(invoice, client, businessSettings);
      const pdfDataUri = doc.output('datauristring');
      const pdfBase64 = pdfDataUri.split(',')[1];

      // 2. Prepare SMTP configuration from settings
      const smtpConfig = {
        host: businessSettings.smtpHost,
        port: businessSettings.smtpPort,
        user: businessSettings.smtpUser,
        pass: businessSettings.smtpPass,
        secure: businessSettings.smtpSecure,
        from: businessSettings.email
      };

      // 3. Prepare email data
      const docNum = isInvoice ? invoice.invoiceNumber : invoice.estimateNumber;
      const emailData = {
        to: emailForm.to,
        subject: emailForm.subject,
        text: emailForm.text,
        filename: `${docNum}.pdf`,
        pdfBase64
      };

      // 4. Send via IPC
      const result = await ipcRenderer.invoke('send-email', { smtpConfig, emailData });

      if (result.success) {
        setEmailModalOpen(false);
        showAlert("Succès", `La facture a été envoyée avec succès à ${emailForm.to} !`);
      } else {
        showAlert("Échec de l'envoi", `Erreur SMTP : ${result.error}`);
      }
    } catch (err) {
      console.error(err);
      showAlert("Erreur", `Une erreur s'est produite lors de la génération ou de l'envoi du mail : ${err.message}`);
    } finally {
      setSendingEmail(false);
    }
  };

  const handleTestSMTP = async () => {
    if (!ipcRenderer) {
      showAlert("Erreur", "Les fonctions de messagerie ne sont pas disponibles hors d'Electron.");
      return;
    }

    setSmtpTesting(true);

    try {
      const smtpConfig = {
        host: businessSettings.smtpHost,
        port: businessSettings.smtpPort,
        user: businessSettings.smtpUser,
        pass: businessSettings.smtpPass,
        secure: businessSettings.smtpSecure
      };

      const result = await ipcRenderer.invoke('test-smtp', smtpConfig);

      if (result.success) {
        showAlert("Connexion réussie", "La configuration SMTP est correcte ! Le serveur a validé les identifiants.");
      } else {
        showAlert("Échec de la connexion", `Erreur de connexion SMTP : ${result.error}`);
      }
    } catch (err) {
      console.error(err);
      showAlert("Erreur", `Impossible de tester la connexion : ${err.message}`);
    } finally {
      setSmtpTesting(false);
    }
  };

  const handleExportPDF = async (invoice) => {
    // Find client details (even if client was deleted, we fallback gracefully using the invoice stored metadata)
    let client = await db.clients.get(invoice.clientId);
    if (!client) {
      client = {
        companyName: invoice.companyName,
        contactName: 'Client supprimé',
        email: 'N/A',
        phone: 'N/A',
        address: 'N/A'
      };
    }
    exportInvoiceToPDF(invoice, client, businessSettings);
  };

  // --- ESTIMATES HANDLERS ---
  const handleCreateEstimateClick = () => {
    if (clients.length === 0) {
      showAlert("Client requis", "Veuillez d'abord créer au moins un client avant de générer un devis.");
      return;
    }
    setEstimateForm({
      id: null,
      clientId: clients[0].id.toString(),
      serviceType: 'service_bnc',
      description: '',
      amountHt: '',
      tvaRate: '20',
      status: 'brouillon',
      date: new Date().toISOString().split('T')[0]
    });
    setEstimateFormErrors({});
    setEstimateModalOpen(true);
  };

  const handleEditEstimateClick = (est) => {
    setEstimateForm({
      id: est.id,
      clientId: est.clientId.toString(),
      serviceType: est.serviceType,
      description: est.description,
      amountHt: est.amountHt.toString(),
      tvaRate: est.tvaRate.toString(),
      status: est.status,
      date: new Date(est.date).toISOString().split('T')[0]
    });
    setEstimateFormErrors({});
    setEstimateModalOpen(true);
  };

  const validateEstimateForm = () => {
    const errors = {};
    if (!estimateForm.clientId) errors.clientId = "Client requis";
    if (!estimateForm.description.trim()) errors.description = "Description requise";
    const htVal = parseFloat(estimateForm.amountHt);
    if (!estimateForm.amountHt || isNaN(htVal) || htVal <= 0) {
      errors.amountHt = "Montant HT valide requis (> 0)";
    }
    setEstimateFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSaveEstimate = async (e) => {
    e.preventDefault();
    if (!validateEstimateForm()) return;

    const selectedClient = clients.find(c => c.id === parseInt(estimateForm.clientId));
    const amountHt = parseFloat(estimateForm.amountHt);
    const tvaRate = parseFloat(estimateForm.tvaRate);
    const amountTva = (amountHt * tvaRate) / 100;
    const amountTotal = amountHt + amountTva;
    const date = new Date(estimateForm.date).toISOString();

    try {
      if (estimateForm.id) {
        // Update
        await db.estimates.update(estimateForm.id, {
          clientId: selectedClient.id,
          companyName: selectedClient.companyName,
          serviceType: estimateForm.serviceType,
          description: estimateForm.description,
          amountHt,
          tvaRate,
          amountTva,
          amountTotal,
          date,
          status: estimateForm.status
        });
      } else {
        // Add new
        const estimateNumber = await generateEstimateNumber(selectedClient.companyName, date);
        await db.estimates.add({
          clientId: selectedClient.id,
          companyName: selectedClient.companyName,
          estimateNumber,
          serviceType: estimateForm.serviceType,
          description: estimateForm.description,
          amountHt,
          tvaRate,
          amountTva,
          amountTotal,
          date,
          status: 'brouillon'
        });
      }

      setEstimateModalOpen(false);
      await loadAllData();
      showAlert("Succès", "Devis enregistré avec succès !");
    } catch (err) {
      console.error(err);
      showAlert("Erreur", `Erreur d'enregistrement : ${err.message}`);
    }
  };

  const handleDeleteEstimate = async (id) => {
    showConfirm(
      "Supprimer le devis",
      "Êtes-vous sûr de vouloir supprimer ce devis ? Cette action est irréversible.",
      async () => {
        await db.estimates.delete(id);
        await loadAllData();
      }
    );
  };

  const handleExportEstimatePDF = async (est) => {
    let client = await db.clients.get(est.clientId);
    if (!client) {
      client = { companyName: est.companyName, contactName: 'Client', email: 'N/A', phone: 'N/A', address: 'N/A' };
    }
    exportEstimateToPDF(est, client, businessSettings);
  };

  const handleConvertEstimateToInvoice = async (est) => {
    showConfirm(
      "Convertir en facture",
      `Voulez-vous convertir le devis ${est.estimateNumber} en facture ? Un nouveau numéro de facture sera généré automatiquement.`,
      async () => {
        try {
          const date = new Date().toISOString();
          const invoiceNumber = await generateInvoiceNumber(est.companyName, date);
          
          await db.invoices.add({
            clientId: est.clientId,
            companyName: est.companyName,
            invoiceNumber,
            serviceType: est.serviceType,
            description: est.description,
            amountHt: est.amountHt,
            tvaRate: est.tvaRate,
            amountTva: est.amountTva,
            amountTotal: est.amountTotal,
            date,
            status: 'brouillon'
          });

          // Mark estimate as accepted
          await db.estimates.update(est.id, { status: 'accepte' });

          await loadAllData();
          setActiveTab('invoices');
          showAlert("Conversion réussie !", `Le devis a été converti en facture ${invoiceNumber} et enregistré en brouillon.`);
        } catch (err) {
          console.error(err);
          showAlert("Erreur", `Erreur lors de la conversion : ${err.message}`);
        }
      }
    );
  };

  // --- EXPENSES HANDLERS ---
  const handleCreateExpenseClick = () => {
    setExpenseForm({
      id: null,
      merchant: '',
      category: 'Logiciels',
      amount: '',
      description: '',
      paymentMethod: 'carte',
      date: new Date().toISOString().split('T')[0]
    });
    setExpenseFormErrors({});
    setExpenseModalOpen(true);
  };

  const handleEditExpenseClick = (exp) => {
    setExpenseForm({
      id: exp.id,
      merchant: exp.merchant,
      category: exp.category,
      amount: exp.amount.toString(),
      description: exp.description || '',
      paymentMethod: exp.paymentMethod || 'carte',
      date: new Date(exp.date).toISOString().split('T')[0]
    });
    setExpenseFormErrors({});
    setExpenseModalOpen(true);
  };

  const validateExpenseForm = () => {
    const errors = {};
    if (!expenseForm.merchant.trim()) errors.merchant = "Fournisseur requis";
    const amt = parseFloat(expenseForm.amount);
    if (!expenseForm.amount || isNaN(amt) || amt <= 0) {
      errors.amount = "Montant supérieur à 0 requis";
    }
    setExpenseFormErrors(errors);
    return Object.keys(errors).length === 0;
  };

  const handleSaveExpense = async (e) => {
    e.preventDefault();
    if (!validateExpenseForm()) return;

    const amount = parseFloat(expenseForm.amount);
    const date = new Date(expenseForm.date).toISOString();

    try {
      if (expenseForm.id) {
        await db.expenses.update(expenseForm.id, {
          merchant: expenseForm.merchant,
          category: expenseForm.category,
          amount,
          description: expenseForm.description,
          paymentMethod: expenseForm.paymentMethod,
          date
        });
      } else {
        await db.expenses.add({
          merchant: expenseForm.merchant,
          category: expenseForm.category,
          amount,
          description: expenseForm.description,
          paymentMethod: expenseForm.paymentMethod,
          date
        });
      }
      setExpenseModalOpen(false);
      await loadAllData();
      showAlert("Succès", "Dépense enregistrée !");
    } catch (err) {
      console.error(err);
      showAlert("Erreur", `Erreur d'enregistrement : ${err.message}`);
    }
  };

  const handleDeleteExpense = async (id) => {
    showConfirm(
      "Supprimer la dépense",
      "Êtes-vous sûr de vouloir supprimer cette dépense ?",
      async () => {
        await db.expenses.delete(id);
        await loadAllData();
      }
    );
  };

  // --- PAYMENTS HANDLERS ---
  const handleOpenPaymentModal = (invoice) => {
    setPaymentForm({
      invoiceId: invoice.id,
      invoiceNumber: invoice.invoiceNumber,
      paymentDate: new Date().toISOString().split('T')[0],
      paymentMethod: 'virement'
    });
    setPaymentModalOpen(true);
  };

  const handleSavePayment = async (e) => {
    e.preventDefault();
    try {
      await db.invoices.update(paymentForm.invoiceId, {
        status: 'payee',
        paymentDate: new Date(paymentForm.paymentDate).toISOString(),
        paymentMethod: paymentForm.paymentMethod
      });
      setPaymentModalOpen(false);
      await loadAllData();
      showAlert("Succès", `Facture ${paymentForm.invoiceNumber} marquée comme payée.`);
    } catch (err) {
      console.error(err);
      showAlert("Erreur", `Impossible d'enregistrer le règlement : ${err.message}`);
    }
  };

  const handleMarkInvoiceAsSent = async (id) => {
    try {
      await db.invoices.update(id, { status: 'envoyee' });
      await loadAllData();
    } catch (err) {
      console.error(err);
    }
  };

  // --- CALCULATION HELPERS FOR DASHBOARD ---

  const calculateCA = () => {
    let htFacture = 0;
    let ttcFacture = 0;
    let htEncaisse = 0;
    let ttcEncaisse = 0;
    invoices.forEach(inv => {
      if (inv.status !== 'brouillon') {
        htFacture += inv.amountHt;
        ttcFacture += inv.amountTotal;
      }
      if (inv.status === 'payee') {
        htEncaisse += inv.amountHt;
        ttcEncaisse += inv.amountTotal;
      }
    });
    return { ht: htEncaisse, ttc: ttcEncaisse, htFacture, ttcFacture };
  };

  const calculateUrssafCharges = () => {
    let charges = 0;
    invoices.forEach(inv => {
      if (inv.status !== 'payee') return; // Only pay charges on encashed CA
      
      let rate = 0;
      if (inv.serviceType === 'service_bnc') {
        rate = businessSettings.urssafServiceBnc;
      } else if (inv.serviceType === 'service_bic') {
        rate = businessSettings.urssafServiceBic;
      } else if (inv.serviceType === 'vente') {
        rate = businessSettings.urssafVente;
      }
      
      // If ACRE is enabled, rates are halved
      if (businessSettings.acreEnabled) {
        rate = rate / 2;
      }

      charges += (inv.amountHt * rate) / 100;
    });
    return charges;
  };

  const calculateTotalExpenses = () => {
    return expenses.reduce((sum, exp) => sum + exp.amount, 0);
  };

  const getServiceTypeBreakdown = () => {
    let bnc = 0;
    let bic = 0;
    let vente = 0;
    invoices.forEach(inv => {
      if (inv.status !== 'payee') return; // Only encashed CA
      if (inv.serviceType === 'service_bnc') bnc += inv.amountHt;
      else if (inv.serviceType === 'service_bic') bic += inv.amountHt;
      else if (inv.serviceType === 'vente') vente += inv.amountHt;
    });
    const total = bnc + bic + vente;
    return {
      bnc: total > 0 ? (bnc / total) * 100 : 0,
      bic: total > 0 ? (bic / total) * 100 : 0,
      vente: total > 0 ? (vente / total) * 100 : 0,
      total
    };
  };

  const getMonthlyCAData = () => {
    // Generate data for past 6 months to display in SVG chart
    const months = ['Jan', 'Fév', 'Mar', 'Avr', 'Mai', 'Jun', 'Jul', 'Aoû', 'Sep', 'Oct', 'Nov', 'Déc'];
    const currentYear = new Date().getFullYear();
    
    // Initialize monthly values
    const caPerMonth = Array(12).fill(0);
    
    invoices.forEach(inv => {
      if (inv.status === 'brouillon') return;
      const invDate = new Date(inv.date);
      if (invDate.getFullYear() === currentYear) {
        const monthIndex = invDate.getMonth();
        caPerMonth[monthIndex] += inv.amountHt;
      }
    });

    return {
      labels: months,
      values: caPerMonth,
      maxVal: Math.max(...caPerMonth, 1000) * 1.15 // Avoid divide by zero, min scale 1000, add 15% padding
    };
  };

  const { ht: totalCaHt, ttc: totalCaTtc, htFacture, ttcFacture } = calculateCA();
  const totalUrssaf = calculateUrssafCharges();
  const totalExpenses = calculateTotalExpenses();
  const netProfit = totalCaHt - totalUrssaf - totalExpenses;
  const breakdown = getServiceTypeBreakdown();
  const monthlyCA = getMonthlyCAData();

  // Calculate annual CA by category for current year to check thresholds
  const currentYearForThresholds = new Date().getFullYear();
  let annualServiceCa = 0;
  let annualVenteCa = 0;
  invoices.forEach(inv => {
    if (inv.status !== 'payee') return;
    const paymentDate = new Date(inv.paymentDate || inv.date);
    if (paymentDate.getFullYear() === currentYearForThresholds) {
      if (inv.serviceType === 'vente') {
        annualVenteCa += inv.amountHt;
      } else {
        annualServiceCa += inv.amountHt;
      }
    }
  });

  const caAlerts = [];
  if (annualServiceCa > 34000) {
    if (annualServiceCa > 39100) {
      caAlerts.push({
        type: 'danger',
        title: 'Seuil de TVA Services Dépassé',
        message: `Votre CA annuel de services (${annualServiceCa.toLocaleString('fr-FR')} €) a dépassé la limite de tolérance de la franchise en base de TVA (39 100 €). Vous devez facturer de la TVA.`
      });
    } else {
      caAlerts.push({
        type: 'warning',
        title: 'Seuil de TVA Services Proche',
        message: `Votre CA annuel de services (${annualServiceCa.toLocaleString('fr-FR')} €) approche le seuil de la franchise en base de TVA (36 800 € / limite de tolérance : 39 100 €).`
      });
    }
  }
  if (annualServiceCa > 70000) {
    caAlerts.push({
      type: 'warning',
      title: 'Plafond Micro-Entreprise Services Proche',
      message: `Votre CA annuel de services (${annualServiceCa.toLocaleString('fr-FR')} €) approche le plafond de la micro-entreprise (77 700 €).`
    });
  }
  if (annualVenteCa > 85000) {
    if (annualVenteCa > 101000) {
      caAlerts.push({
        type: 'danger',
        title: 'Seuil de TVA Ventes Dépassé',
        message: `Votre CA annuel de ventes (${annualVenteCa.toLocaleString('fr-FR')} €) a dépassé la limite de tolérance de la franchise en base de TVA (101 000 €). Vous devez facturer de la TVA.`
      });
    } else {
      caAlerts.push({
        type: 'warning',
        title: 'Seuil de TVA Ventes Proche',
        message: `Votre CA annuel de ventes (${annualVenteCa.toLocaleString('fr-FR')} €) approche le seuil de la franchise en base de TVA (91 900 € / limite de tolérance : 101 000 €).`
      });
    }
  }
  if (annualVenteCa > 170000) {
    caAlerts.push({
      type: 'warning',
      title: 'Plafond Micro-Entreprise Ventes Proche',
      message: `Votre CA annuel de ventes (${annualVenteCa.toLocaleString('fr-FR')} €) approche le plafond de la micro-entreprise (188 700 €).`
    });
  }

  // Filter lists
  const filteredClients = clients.filter(c => 
    c.companyName.toLowerCase().includes(clientSearch.toLowerCase()) || 
    c.contactName.toLowerCase().includes(clientSearch.toLowerCase()) || 
    c.email.toLowerCase().includes(clientSearch.toLowerCase())
  );

  const filteredInvoices = invoices.filter(inv => 
    inv.invoiceNumber.toLowerCase().includes(invoiceSearch.toLowerCase()) || 
    inv.companyName.toLowerCase().includes(invoiceSearch.toLowerCase()) ||
    inv.description.toLowerCase().includes(invoiceSearch.toLowerCase())
  );

  // Top 10 Invoices
  const top10Invoices = invoices.slice(0, 10);
  // Top 5 Clients (Sorted by registration date/id)
  const top5Clients = [...clients]
    .sort((a, b) => new Date(b.createdAt || 0) - new Date(a.createdAt || 0))
    .slice(0, 5);

  return (
    <div className="app-container">
      {/* --- SIDEBAR --- */}
      <aside className="sidebar">
        <div className="logo-container">
          <div className="logo-icon">ᛟ</div>
          <span className="logo-text">ASGARD CRM</span>
        </div>
        
        <nav className="nav-menu">
          <li 
            className={`nav-item ${activeTab === 'dashboard' ? 'active' : ''}`}
            onClick={() => setActiveTab('dashboard')}
          >
            <Icons.Dashboard />
            <span>Dashboard</span>
          </li>
          <li 
            className={`nav-item ${activeTab === 'clients' ? 'active' : ''}`}
            onClick={() => setActiveTab('clients')}
          >
            <Icons.Clients />
            <span>Clients</span>
          </li>
          <li 
            className={`nav-item ${activeTab === 'estimates' ? 'active' : ''}`}
            onClick={() => setActiveTab('estimates')}
          >
            <Icons.Estimates />
            <span>Devis</span>
          </li>
          <li 
            className={`nav-item ${activeTab === 'invoices' ? 'active' : ''}`}
            onClick={() => setActiveTab('invoices')}
          >
            <Icons.Invoices />
            <span>Factures</span>
          </li>
          <li 
            className={`nav-item ${activeTab === 'expenses' ? 'active' : ''}`}
            onClick={() => setActiveTab('expenses')}
          >
            <Icons.Expenses />
            <span>Dépenses</span>
          </li>
          <li 
            className={`nav-item ${activeTab === 'compta' ? 'active' : ''}`}
            onClick={() => setActiveTab('compta')}
          >
            <Icons.Accounting />
            <span>Comptabilité</span>
          </li>
          <li 
            className={`nav-item ${activeTab === 'settings' ? 'active' : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            <Icons.Settings />
            <span>Paramètres</span>
          </li>
        </nav>

        <div className="sidebar-footer">
          <p>© {new Date().getFullYear()} Asgard CRM</p>
          <p style={{ color: 'var(--color-gold)', marginTop: '0.25rem', fontWeight: 600 }}>Mode Bureau Local</p>
        </div>
      </aside>

      {/* --- MAIN CONTENT AREA --- */}
      <main className="main-content">
        
        {/* --- TAB: DASHBOARD --- */}
        {activeTab === 'dashboard' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Tableau de bord</h1>
                <p>Aperçu financier en temps réel de votre auto-entreprise.</p>
              </div>
              <div className="flex-gap-2">
                <button className="btn btn-primary" onClick={handleOpenAddInvoice}>
                  <Icons.Add /> Nouvelle Facture
                </button>
              </div>
            </div>

            {/* Alerts */}
            {caAlerts.length > 0 && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', marginBottom: '1.5rem' }}>
                {caAlerts.map((alert, index) => (
                  <div 
                    key={index} 
                    className="card-glass"
                    style={{ 
                      padding: '1rem 1.25rem', 
                      borderColor: alert.type === 'danger' ? 'rgba(239, 68, 68, 0.4)' : 'rgba(229, 169, 60, 0.4)',
                      background: alert.type === 'danger' ? 'rgba(239, 68, 68, 0.05)' : 'rgba(229, 169, 60, 0.03)',
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: '0.75rem'
                    }}
                  >
                    <span style={{ fontSize: '1.25rem' }}>{alert.type === 'danger' ? '🚨' : '⚠️'}</span>
                    <div>
                      <h4 style={{ margin: 0, fontWeight: 700, color: alert.type === 'danger' ? '#EF4444' : 'var(--color-gold)', fontSize: '0.95rem' }}>
                        {alert.title}
                      </h4>
                      <p style={{ margin: '0.25rem 0 0 0', fontSize: '0.85rem', color: 'var(--text-secondary)', lineHeight: '1.4' }}>
                        {alert.message}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Metric Grid */}
            <div className="dashboard-grid">
              <div className="card-glass">
                <div className="metric-label">CA HT Encaissé</div>
                <div className="metric-value metric-highlight">{totalCaHt.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">Facturé : {htFacture.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} € HT (TTC : {totalCaTtc.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €)</div>
              </div>
              <div className="card-glass" style={{ borderColor: 'rgba(239, 68, 68, 0.2)' }}>
                <div className="metric-label">Dépenses Totales</div>
                <div className="metric-value" style={{ color: '#EF4444' }}>{totalExpenses.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">Achats et frais professionnels enregistrés</div>
              </div>
              <div className="card-glass" style={{ borderColor: 'rgba(244, 63, 94, 0.2)' }}>
                <div className="metric-label">Charges URSSAF</div>
                <div className="metric-value" style={{ color: '#FF6B8B' }}>{totalUrssaf.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">
                  Sur encaissé • {businessSettings.acreEnabled ? 'Taux ACRE (-50%)' : 'Taux plein'}
                </div>
              </div>
              <div className="card-glass" style={{ borderColor: 'rgba(16, 185, 129, 0.3)' }}>
                <div className="metric-label">Bénéfice Net Réel</div>
                <div className="metric-value" style={{ color: '#10B981' }}>{netProfit.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">Trésorerie réelle après charges et dépenses</div>
              </div>
            </div>

            {/* Detailed Analytics Panel */}
            <div className="dashboard-details-grid" style={{ marginBottom: '2rem' }}>
              {/* SVG CA Evolution Chart */}
              <div className="card-glass">
                <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '1.25rem' }}>Évolution du Chiffre d'Affaires HT ({new Date().getFullYear()})</h3>
                <div style={{ position: 'relative', width: '100%', height: '320px' }}>
                  <svg width="100%" height="100%" viewBox="0 0 600 280" preserveAspectRatio="none">
                    <defs>
                      <linearGradient id="chart-gradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="var(--color-gold)" stopOpacity="0.3" />
                        <stop offset="100%" stopColor="var(--color-gold)" stopOpacity="0.0" />
                      </linearGradient>
                    </defs>

                    {/* Horizontal Grid Lines */}
                    {[0, 0.25, 0.5, 0.75, 1].map((ratio, idx) => (
                      <line
                        key={idx}
                        x1="30"
                        y1={30 + ratio * 200}
                        x2="570"
                        y2={30 + ratio * 200}
                        className="chart-grid-line"
                      />
                    ))}

                    {/* Chart Area and Line path */}
                    {(() => {
                      const points = monthlyCA.values.map((val, idx) => {
                        const x = 50 + idx * 46;
                        const y = 230 - (val / monthlyCA.maxVal) * 200;
                        return { x, y };
                      });

                      const pathD = points.reduce((acc, p, idx) => 
                        idx === 0 ? `M ${p.x} ${p.y}` : `${acc} L ${p.x} ${p.y}`, ''
                      );

                      const areaD = points.length > 0 
                        ? `${pathD} L ${points[points.length-1].x} 230 L ${points[0].x} 230 Z` 
                        : '';

                      return (
                        <>
                          <path d={areaD} className="chart-area" />
                          <path d={pathD} className="chart-line" />
                          
                          {/* Data points */}
                          {points.map((p, idx) => (
                            <g key={idx}>
                              <circle
                                cx={p.x}
                                cy={p.y}
                                r="4.5"
                                className="chart-dot"
                              />
                              {monthlyCA.values[idx] > 0 && (
                                <text
                                  x={p.x}
                                  y={p.y - 12}
                                  textAnchor="middle"
                                  fill="var(--text-primary)"
                                  fontSize="9"
                                  fontWeight="600"
                                >
                                  {Math.round(monthlyCA.values[idx])}€
                                </text>
                              )}
                            </g>
                          ))}
                        </>
                      );
                    })()}

                    {/* X-Axis Labels */}
                    {monthlyCA.labels.map((label, idx) => (
                      <text
                        key={idx}
                        x={50 + idx * 46}
                        y="255"
                        textAnchor="middle"
                        className="chart-label"
                      >
                        {label}
                      </text>
                    ))}
                  </svg>
                </div>
              </div>

              {/* Service Types Breakdown */}
              <div className="card-glass flex-between" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
                <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '1.5rem' }}>Types d'activité</h3>
                <div style={{ flexGrow: 1, display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: '1.25rem' }}>
                  
                  {/* Category BNC */}
                  <div>
                    <div className="flex-between" style={{ fontSize: '0.85rem', marginBottom: '0.4rem' }}>
                      <span style={{ fontWeight: 500 }}>Libérale (BNC)</span>
                      <span style={{ color: 'var(--color-gold)', fontWeight: 600 }}>{breakdown.bnc.toFixed(1)}%</span>
                    </div>
                    <div style={{ height: '8px', backgroundColor: 'var(--bg-tertiary)', borderRadius: '4px', overflow: 'hidden' }}>
                      <div style={{ width: `${breakdown.bnc}%`, height: '100%', backgroundColor: 'var(--color-gold)', borderRadius: '4px' }}></div>
                    </div>
                  </div>

                  {/* Category BIC Service */}
                  <div>
                    <div className="flex-between" style={{ fontSize: '0.85rem', marginBottom: '0.4rem' }}>
                      <span style={{ fontWeight: 500 }}>Artisanale/Comm. (BIC)</span>
                      <span style={{ color: 'var(--color-blue)', fontWeight: 600 }}>{breakdown.bic.toFixed(1)}%</span>
                    </div>
                    <div style={{ height: '8px', backgroundColor: 'var(--bg-tertiary)', borderRadius: '4px', overflow: 'hidden' }}>
                      <div style={{ width: `${breakdown.bic}%`, height: '100%', backgroundColor: 'var(--color-blue)', borderRadius: '4px' }}></div>
                    </div>
                  </div>

                  {/* Category BIC Vente */}
                  <div>
                    <div className="flex-between" style={{ fontSize: '0.85rem', marginBottom: '0.4rem' }}>
                      <span style={{ fontWeight: 500 }}>Vente Marchandises</span>
                      <span style={{ color: 'var(--success)', fontWeight: 600 }}>{breakdown.vente.toFixed(1)}%</span>
                    </div>
                    <div style={{ height: '8px', backgroundColor: 'var(--bg-tertiary)', borderRadius: '4px', overflow: 'hidden' }}>
                      <div style={{ width: `${breakdown.vente}%`, height: '100%', backgroundColor: 'var(--success)', borderRadius: '4px' }}></div>
                    </div>
                  </div>

                </div>
              </div>
            </div>

            {/* Bottom grids: Top 10 Invoices & Top 5 Clients */}
            <div className="dashboard-details-grid">
              {/* Top 10 Invoices */}
              <div className="card-glass">
                <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '1rem' }}>Dernières factures saisies (Top 10)</h3>
                
                {top10Invoices.length === 0 ? (
                  <div className="empty-state" style={{ padding: '2rem' }}>
                    <p>Aucune facture enregistrée pour le moment.</p>
                  </div>
                ) : (
                  <div className="table-container" style={{ marginTop: '0' }}>
                    <table className="table-glass">
                      <thead>
                        <tr>
                          <th>Numéro</th>
                          <th>Client</th>
                          <th>Statut</th>
                          <th>Montant TTC</th>
                          <th className="text-right">Actions</th>
                        </tr>
                      </thead>
                      <tbody>
                        {top10Invoices.map((inv) => (
                          <tr key={inv.id}>
                            <td style={{ fontWeight: 600, color: 'var(--color-gold)' }}>{inv.invoiceNumber}</td>
                            <td>{inv.companyName}</td>
                            <td>
                              {inv.status === 'brouillon' && <span className="badge badge-secondary">Brouillon</span>}
                              {inv.status === 'envoye' && <span className="badge badge-blue">Envoyée</span>}
                              {inv.status === 'payee' && <span className="badge badge-success">Payée</span>}
                            </td>
                            <td style={{ fontWeight: 600 }}>{inv.amountTotal.toFixed(2)} €</td>
                            <td className="text-right">
                              <div className="flex-gap-2" style={{ justifyContent: 'flex-end' }}>
                                {inv.status !== 'payee' && (
                                  <button 
                                    className="btn btn-secondary"
                                    style={{ padding: '0.4rem 0.75rem', fontSize: '0.8rem', border: '1px solid var(--color-gold)', color: 'var(--color-gold)' }}
                                    title="Enregistrer le règlement"
                                    onClick={() => handleOpenPaymentModal(inv)}
                                  >
                                    Régler
                                  </button>
                                )}
                                <button 
                                  className="btn btn-secondary btn-icon-only" 
                                  title="Envoyer par e-mail"
                                  onClick={() => handleOpenSendEmail(inv, 'invoice')}
                                >
                                  <Icons.Email />
                                </button>
                                <button 
                                  className="btn btn-secondary btn-icon-only" 
                                  title="Exporter en PDF"
                                  onClick={() => handleExportPDF(inv)}
                                >
                                  <Icons.Download />
                                </button>
                              </div>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>

              {/* Top 5 Clients */}
              <div className="card-glass">
                <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '1rem' }}>Derniers clients (Top 5)</h3>
                
                {top5Clients.length === 0 ? (
                  <div className="empty-state" style={{ padding: '2rem' }}>
                    <p>Aucun client enregistré.</p>
                  </div>
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                    {top5Clients.map((client) => (
                      <div key={client.id} className="flex-between" style={{ padding: '0.75rem 1rem', backgroundColor: 'rgba(255,255,255,0.02)', borderRadius: '8px', border: '1px solid var(--border-glass)' }}>
                        <div>
                          <div style={{ fontWeight: 600, fontSize: '0.9rem' }}>{client.companyName}</div>
                          <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)' }}>{client.contactName}</div>
                        </div>
                        <div className="badge badge-blue">Client</div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {/* --- TAB: CLIENTS --- */}
        {activeTab === 'clients' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Fichiers Clients</h1>
                <p>Gérez les coordonnées de vos clients et partenaires commerciaux.</p>
              </div>
              <div className="flex-gap-2">
                <div className="search-container">
                  <span className="search-icon"><Icons.Search /></span>
                  <input 
                    type="text" 
                    placeholder="Rechercher un client..." 
                    className="search-input"
                    value={clientSearch}
                    onChange={(e) => setClientSearch(e.target.value)}
                  />
                </div>
                <button className="btn btn-primary" onClick={handleOpenAddClient}>
                  <Icons.Add /> Nouveau Client
                </button>
              </div>
            </div>

            <div className="card-glass" style={{ padding: '0.5rem 0' }}>
              {filteredClients.length === 0 ? (
                <div className="empty-state">
                  <div className="empty-state-icon">ᛟ</div>
                  <p>Aucun client trouvé.</p>
                </div>
              ) : (
                <div className="table-container">
                  <table className="table-glass">
                    <thead>
                      <tr>
                        <th>Entreprise</th>
                        <th>Contact</th>
                        <th>Email</th>
                        <th>Téléphone</th>
                        <th>Adresse</th>
                        <th className="text-right">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredClients.map((client) => (
                        <tr key={client.id}>
                          <td style={{ fontWeight: 700, color: 'var(--color-gold)' }}>{client.companyName}</td>
                          <td>{client.contactName}</td>
                          <td>{client.email}</td>
                          <td style={{ whiteSpace: 'nowrap' }}>{client.phone}</td>
                          <td style={{ maxWidth: '220px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={client.address}>
                            {client.address}
                          </td>
                          <td className="text-right">
                            <div className="flex-gap-2" style={{ justifyContent: 'flex-end' }}>
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Modifier"
                                onClick={() => handleOpenEditClient(client)}
                              >
                                <Icons.Edit />
                              </button>
                              <button 
                                className="btn btn-danger btn-icon-only" 
                                title="Supprimer"
                                onClick={() => handleDeleteClient(client.id)}
                              >
                                <Icons.Delete />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </div>
        )}

        {/* --- TAB: INVOICES --- */}
        {activeTab === 'invoices' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Factures</h1>
                <p>Historique des prestations de services, facturation et exports PDF.</p>
              </div>
              <div className="flex-gap-2" style={{ alignItems: 'center' }}>
                {selectedInvoiceIds.length > 0 && (
                  <button className="btn btn-danger" onClick={handleDeleteSelectedInvoices}>
                    <Icons.Delete /> Supprimer ({selectedInvoiceIds.length})
                  </button>
                )}
                <div className="search-container">
                  <span className="search-icon"><Icons.Search /></span>
                  <input 
                    type="text" 
                    placeholder="Numéro, client, description..." 
                    className="search-input"
                    value={invoiceSearch}
                    onChange={(e) => setInvoiceSearch(e.target.value)}
                  />
                </div>
                <button className="btn btn-primary" onClick={handleOpenAddInvoice}>
                  <Icons.Add /> Nouvelle Facture
                </button>
              </div>
            </div>

            <div className="card-glass" style={{ padding: '0.5rem 0' }}>
              {filteredInvoices.length === 0 ? (
                <div className="empty-state">
                  <div className="empty-state-icon">ᛟ</div>
                  <p>Aucune facture trouvée.</p>
                </div>
              ) : (
                <div className="table-container">
                  <table className="table-glass">
                    <thead>
                      <tr>
                        <th style={{ width: '40px' }}>
                          <input 
                            type="checkbox"
                            style={{ transform: 'scale(1.15)', cursor: 'pointer', accentColor: 'var(--color-gold)' }}
                            checked={filteredInvoices.length > 0 && selectedInvoiceIds.length === filteredInvoices.length}
                            onChange={(e) => {
                              if (e.target.checked) {
                                setSelectedInvoiceIds(filteredInvoices.map(inv => inv.id));
                              } else {
                                setSelectedInvoiceIds([]);
                              }
                            }}
                          />
                        </th>
                        <th>Numéro</th>
                        <th>Date</th>
                        <th>Client</th>
                        <th>Type</th>
                        <th>Statut</th>
                        <th>Montant HT</th>
                        <th>TVA</th>
                        <th>Montant TTC</th>
                        <th className="text-right">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredInvoices.map((inv) => (
                        <tr key={inv.id} className={selectedInvoiceIds.includes(inv.id) ? 'selected-row' : ''}>
                          <td>
                            <input 
                              type="checkbox"
                              style={{ transform: 'scale(1.15)', cursor: 'pointer', accentColor: 'var(--color-gold)' }}
                              checked={selectedInvoiceIds.includes(inv.id)}
                              onChange={(e) => {
                                if (e.target.checked) {
                                  setSelectedInvoiceIds([...selectedInvoiceIds, inv.id]);
                                } else {
                                  setSelectedInvoiceIds(selectedInvoiceIds.filter(id => id !== inv.id));
                                }
                              }}
                            />
                          </td>
                          <td style={{ fontWeight: 700, color: 'var(--color-gold)' }}>{inv.invoiceNumber}</td>
                          <td>{new Date(inv.date).toLocaleDateString('fr-FR')}</td>
                          <td>{inv.companyName}</td>
                          <td>
                            {inv.serviceType === 'service_bnc' && <span className="badge badge-blue">BNC</span>}
                            {inv.serviceType === 'service_bic' && <span className="badge badge-blue">BIC</span>}
                            {inv.serviceType === 'vente' && <span className="badge badge-success">Vente</span>}
                          </td>
                          <td>
                            {inv.status === 'brouillon' && <span className="badge badge-secondary">Brouillon</span>}
                            {inv.status === 'envoye' && <span className="badge badge-blue">Envoyée</span>}
                            {inv.status === 'payee' && (
                              <span className="badge badge-success" title={`Payée le ${new Date(inv.paymentDate || inv.date).toLocaleDateString('fr-FR')} par ${inv.paymentMethod || 'Virement'}`}>
                                Payée
                              </span>
                            )}
                          </td>
                          <td>{inv.amountHt.toFixed(2)} €</td>
                          <td>{inv.tvaRate}%</td>
                          <td style={{ fontWeight: 600 }}>{inv.amountTotal.toFixed(2)} €</td>
                          <td className="text-right">
                            <div className="flex-gap-2" style={{ justifyContent: 'flex-end' }}>
                              {inv.status !== 'payee' && (
                                <button 
                                  className="btn btn-secondary"
                                  style={{ padding: '0.4rem 0.75rem', fontSize: '0.8rem', border: '1px solid var(--color-gold)', color: 'var(--color-gold)' }}
                                  title="Enregistrer le règlement"
                                  onClick={() => handleOpenPaymentModal(inv)}
                                >
                                  Régler
                                </button>
                              )}
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Envoyer par e-mail"
                                onClick={() => handleOpenSendEmail(inv, 'invoice')}
                              >
                                <Icons.Email />
                              </button>
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Télécharger le PDF"
                                onClick={() => handleExportPDF(inv)}
                              >
                                <Icons.Download />
                              </button>
                              <button 
                                className="btn btn-danger btn-icon-only" 
                                title="Supprimer la facture"
                                onClick={() => handleDeleteInvoice(inv.id)}
                              >
                                <Icons.Delete />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </div>
        )}

        {/* --- TAB: SETTINGS --- */}
        {activeTab === 'settings' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Paramètres de l'entreprise</h1>
                <p>Configurez vos mentions légales, vos taux de charge URSSAF et vos coordonnées.</p>
              </div>
            </div>

            <div className="card-glass">
              <form onSubmit={(e) => { e.preventDefault(); showAlert("Succès", "Paramètres enregistrés !"); }}>
                
                {/* Section : My Info */}
                <div className="settings-section">
                  <div className="settings-section-title">Coordonnées de l'émetteur</div>
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem' }}>
                    <div className="form-group">
                      <label className="form-label">Nom de l'entreprise</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        value={businessSettings.companyName}
                        onChange={(e) => saveSettings({ ...businessSettings, companyName: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Nom du contact</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        value={businessSettings.contactName}
                        onChange={(e) => saveSettings({ ...businessSettings, contactName: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Email professionnel</label>
                      <input 
                        type="email" 
                        className="form-input" 
                        value={businessSettings.email}
                        onChange={(e) => saveSettings({ ...businessSettings, email: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Numéro de téléphone</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        value={businessSettings.phone}
                        onChange={(e) => saveSettings({ ...businessSettings, phone: e.target.value })}
                      />
                    </div>
                    <div className="form-group" style={{ gridColumn: 'span 2' }}>
                      <label className="form-label">Adresse professionnelle</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        value={businessSettings.address}
                        onChange={(e) => saveSettings({ ...businessSettings, address: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">SIRET</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        value={businessSettings.siret}
                        onChange={(e) => saveSettings({ ...businessSettings, siret: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">IBAN bancaire (Règlement)</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        value={businessSettings.iban}
                        onChange={(e) => saveSettings({ ...businessSettings, iban: e.target.value })}
                      />
                    </div>
                  </div>
                </div>

                {/* Section : URSSAF config */}
                <div className="settings-section" style={{ borderBottom: 'none', paddingBottom: '0' }}>
                  <div className="settings-section-title">Cotisations sociales (URSSAF)</div>
                  
                  <div className="form-group" style={{ marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                    <input 
                      type="checkbox" 
                      id="acreCheckbox"
                      style={{ transform: 'scale(1.25)', accentColor: 'var(--color-gold)', cursor: 'pointer' }}
                      checked={businessSettings.acreEnabled}
                      onChange={(e) => saveSettings({ ...businessSettings, acreEnabled: e.target.checked })}
                    />
                    <label htmlFor="acreCheckbox" style={{ fontWeight: 600, fontSize: '0.9rem', cursor: 'pointer' }}>
                      Bénéficiaire de l'ACRE (Taux de cotisations réduits de 50%)
                    </label>
                  </div>

                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '1.25rem' }}>
                    <div className="form-group">
                      <label className="form-label">Taux Service BNC (%)</label>
                      <input 
                        type="number" 
                        step="0.01"
                        className="form-input" 
                        value={businessSettings.urssafServiceBnc}
                        onChange={(e) => saveSettings({ ...businessSettings, urssafServiceBnc: parseFloat(e.target.value) || 0 })}
                      />
                      <span className="metric-subtext">Par défaut 21.1% (Libérale BNC)</span>
                    </div>
                    <div className="form-group">
                      <label className="form-label">Taux Service BIC (%)</label>
                      <input 
                        type="number" 
                        step="0.01"
                        className="form-input" 
                        value={businessSettings.urssafServiceBic}
                        onChange={(e) => saveSettings({ ...businessSettings, urssafServiceBic: parseFloat(e.target.value) || 0 })}
                      />
                      <span className="metric-subtext">Par défaut 21.1% (Artisanal/Comm. BIC)</span>
                    </div>
                    <div className="form-group">
                      <label className="form-label">Taux Vente BIC (%)</label>
                      <input 
                        type="number" 
                        step="0.01"
                        className="form-input" 
                        value={businessSettings.urssafVente}
                        onChange={(e) => saveSettings({ ...businessSettings, urssafVente: parseFloat(e.target.value) || 0 })}
                      />
                      <span className="metric-subtext">Par défaut 12.3% (Vente marchandises)</span>
                    </div>
                  </div>
                </div>

                {/* Section : Identité Visuelle & PDF */}
                <div className="settings-section" style={{ borderTop: '1px solid var(--border-glass)', paddingTop: '1.5rem', marginTop: '1.5rem' }}>
                  <div className="settings-section-title">Identité Visuelle & PDF</div>
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem' }}>
                    <div className="form-group">
                      <label className="form-label">Couleur d'accentuation des PDF</label>
                      <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                        <input 
                          type="color" 
                          className="form-input"
                          style={{ width: '50px', height: '40px', padding: '2px', cursor: 'pointer', border: '1px solid var(--border-glass)' }}
                          value={businessSettings.customColor || '#E5A93C'}
                          onChange={(e) => saveSettings({ ...businessSettings, customColor: e.target.value })}
                        />
                        <input 
                          type="text" 
                          className="form-input"
                          style={{ fontFamily: 'monospace' }}
                          value={businessSettings.customColor || '#E5A93C'}
                          onChange={(e) => saveSettings({ ...businessSettings, customColor: e.target.value })}
                        />
                        <button 
                          type="button" 
                          className="btn btn-secondary" 
                          onClick={() => saveSettings({ ...businessSettings, customColor: '#E5A93C' })}
                          style={{ padding: '0.5rem 1rem', fontSize: '0.8rem' }}
                        >
                          Réinitialiser
                        </button>
                      </div>
                      <span className="metric-subtext">Couleur utilisée pour les titres et totaux sur les devis/factures exportés.</span>
                    </div>

                    <div className="form-group">
                      <label className="form-label">Logo de l'entreprise (Format image)</label>
                      <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                        {businessSettings.logoBase64 ? (
                          <div style={{ position: 'relative', border: '1px solid var(--border-glass)', borderRadius: '4px', padding: '4px', background: 'rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', justifyContent: 'center', height: '40px', width: '60px' }}>
                            <img src={businessSettings.logoBase64} alt="Logo" style={{ maxHeight: '100%', maxWidth: '100%', objectFit: 'contain' }} />
                            <button 
                              type="button" 
                              style={{ position: 'absolute', top: '-5px', right: '-5px', background: '#EF4444', color: 'white', border: 'none', borderRadius: '50%', width: '16px', height: '16px', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '10px', fontWeight: 'bold' }}
                              onClick={() => saveSettings({ ...businessSettings, logoBase64: '' })}
                              title="Supprimer le logo"
                            >
                              ×
                            </button>
                          </div>
                        ) : (
                          <div style={{ border: '1px dashed var(--border-glass)', borderRadius: '4px', height: '40px', width: '60px', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '12px', color: 'var(--text-muted)' }}>
                            Aucun
                          </div>
                        )}
                        <input 
                          type="file" 
                          accept="image/*"
                          style={{ display: 'none' }}
                          id="logoUploadInput"
                          onChange={(e) => {
                            const file = e.target.files[0];
                            if (file) {
                              const reader = new FileReader();
                              reader.onload = (event) => {
                                saveSettings({ ...businessSettings, logoBase64: event.target.result });
                              };
                              reader.readAsDataURL(file);
                            }
                          }}
                        />
                        <label htmlFor="logoUploadInput" className="btn btn-secondary" style={{ padding: '0.5rem 1rem', fontSize: '0.8rem', cursor: 'pointer', margin: 0 }}>
                          Choisir un logo
                        </label>
                      </div>
                      <span className="metric-subtext">Recommandé : PNG transparent, format paysage (hauteur max 60px).</span>
                    </div>
                  </div>
                </div>

                {/* Section : SMTP config */}
                <div className="settings-section" style={{ borderTop: '1px solid var(--border-glass)', paddingTop: '1.5rem', marginTop: '1.5rem' }}>
                  <div className="flex-between" style={{ marginBottom: '1.25rem' }}>
                    <div className="settings-section-title" style={{ marginBottom: '0' }}>Configuration de messagerie (SMTP)</div>
                    <button 
                      type="button" 
                      className="btn btn-secondary" 
                      onClick={handleTestSMTP}
                      disabled={smtpTesting}
                    >
                      {smtpTesting ? "Vérification en cours..." : "Tester la connexion SMTP"}
                    </button>
                  </div>
                  
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem', marginBottom: '1rem' }}>
                    <div className="form-group">
                      <label className="form-label">Hôte SMTP</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        placeholder="ex: 127.0.0.1 (Proton Mail Bridge)"
                        value={businessSettings.smtpHost}
                        onChange={(e) => saveSettings({ ...businessSettings, smtpHost: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Port SMTP</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        placeholder="ex: 1025"
                        value={businessSettings.smtpPort}
                        onChange={(e) => saveSettings({ ...businessSettings, smtpPort: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Utilisateur SMTP / Adresse Mail</label>
                      <input 
                        type="text" 
                        className="form-input" 
                        placeholder="Votre adresse e-mail"
                        value={businessSettings.smtpUser}
                        onChange={(e) => saveSettings({ ...businessSettings, smtpUser: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Mot de passe SMTP</label>
                      <input 
                        type="password" 
                        className="form-input" 
                        placeholder="Mot de passe ou clé générée"
                        value={businessSettings.smtpPass}
                        onChange={(e) => saveSettings({ ...businessSettings, smtpPass: e.target.value })}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">Sécurité Connexion</label>
                      <select 
                        className="form-input"
                        value={businessSettings.smtpSecure}
                        onChange={(e) => saveSettings({ ...businessSettings, smtpSecure: e.target.value })}
                      >
                        <option value="none">Aucune (STARTTLS automatique / Proton Mail Bridge)</option>
                        <option value="ssl">SSL Strict (Port 465)</option>
                      </select>
                    </div>
                  </div>
                  
                  <span className="metric-subtext" style={{ display: 'block', color: 'var(--color-gold)', lineHeight: '1.5' }}>
                    💡 Pour Proton Mail Bridge, laissez l'Hôte sur <strong>127.0.0.1</strong>, le port sur celui indiqué par votre application Bridge (souvent 1025), et choisissez "Aucune" pour la sécurité.
                  </span>
                </div>

                {/* Section : Sauvegarde et Restauration */}
                <div className="settings-section" style={{ borderTop: '1px solid var(--border-glass)', paddingTop: '1.5rem', marginTop: '1.5rem' }}>
                  <div className="settings-section-title">Sécurité & Sauvegarde des données</div>
                  <p className="metric-subtext" style={{ marginBottom: '1.25rem', lineHeight: '1.5' }}>
                    Vos données sont stockées localement dans votre base de données locale. Exportez régulièrement des sauvegardes pour éviter toute perte de données en cas de panne de votre ordinateur.
                  </p>
                  
                  <div style={{ display: 'flex', gap: '1rem' }}>
                    <button 
                      type="button" 
                      className="btn btn-primary"
                      onClick={handleExportBackup}
                    >
                      Exporter les données (.json)
                    </button>
                    
                    <input 
                      type="file" 
                      accept=".json"
                      id="dbBackupImportInput"
                      style={{ display: 'none' }}
                      onChange={handleImportBackup}
                    />
                    <button 
                      type="button" 
                      className="btn btn-secondary"
                      onClick={() => document.getElementById('dbBackupImportInput').click()}
                    >
                      Restaurer une sauvegarde
                    </button>
                  </div>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* --- TAB: ESTIMATES (DEVIS) --- */}
        {activeTab === 'estimates' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Gestion des Devis</h1>
                <p>Créez, éditez, exportez et convertissez vos devis en factures.</p>
              </div>
              <div className="flex-gap-2">
                <button className="btn btn-primary" onClick={handleCreateEstimateClick}>
                  <Icons.Add /> Nouveau Devis
                </button>
              </div>
            </div>

            {/* Filter Bar */}
            <div className="filter-bar">
              <div style={{ position: 'relative', flexGrow: 1 }}>
                <span className="search-icon"><Icons.Search /></span>
                <input 
                  type="text" 
                  className="search-input" 
                  placeholder="Rechercher un devis (Numéro, client, description...)"
                  value={estimatesSearch}
                  onChange={(e) => setEstimatesSearch(e.target.value)}
                />
              </div>
            </div>

            {/* Estimates Table */}
            {estimates.length === 0 ? (
              <div className="empty-state">
                <p>Aucun devis créé pour le moment. Cliquez sur "Nouveau Devis" pour commencer.</p>
              </div>
            ) : (
              <div className="table-container">
                <table className="table-glass">
                  <thead>
                    <tr>
                      <th>Numéro</th>
                      <th>Client</th>
                      <th>Date</th>
                      <th>Statut</th>
                      <th>Montant HT</th>
                      <th>Total TTC</th>
                      <th className="text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {estimates
                      .filter(est => 
                        est.estimateNumber.toLowerCase().includes(estimatesSearch.toLowerCase()) ||
                        est.companyName.toLowerCase().includes(estimatesSearch.toLowerCase()) ||
                        (est.description && est.description.toLowerCase().includes(estimatesSearch.toLowerCase()))
                      )
                      .map(est => (
                        <tr key={est.id}>
                          <td style={{ fontWeight: 600, color: 'var(--color-gold)' }}>{est.estimateNumber}</td>
                          <td>{est.companyName}</td>
                          <td>{new Date(est.date).toLocaleDateString('fr-FR')}</td>
                          <td>
                            {est.status === 'brouillon' && <span className="badge badge-secondary">Brouillon</span>}
                            {est.status === 'envoye' && <span className="badge badge-blue">Envoyé</span>}
                            {est.status === 'accepte' && <span className="badge badge-success">Accepté</span>}
                            {est.status === 'refuse' && <span className="badge badge-danger">Refusé</span>}
                          </td>
                          <td>{est.amountHt.toFixed(2)} €</td>
                          <td style={{ fontWeight: 600 }}>{est.amountTotal.toFixed(2)} €</td>
                          <td className="text-right">
                            <div className="flex-gap-2" style={{ justifyContent: 'flex-end' }}>
                              {est.status !== 'accepte' && (
                                <button 
                                  className="btn btn-secondary"
                                  style={{ padding: '0.4rem 0.75rem', fontSize: '0.8rem', border: '1px solid var(--color-gold)', color: 'var(--color-gold)' }}
                                  title="Convertir en Facture"
                                  onClick={() => handleConvertEstimateToInvoice(est)}
                                >
                                  Facturer
                                </button>
                              )}
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Envoyer par e-mail"
                                onClick={() => handleOpenSendEmail(est, 'estimate')}
                              >
                                <Icons.Email />
                              </button>
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Télécharger le PDF"
                                onClick={() => handleExportEstimatePDF(est)}
                              >
                                <Icons.Download />
                              </button>
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Modifier le devis"
                                onClick={() => handleEditEstimateClick(est)}
                              >
                                <Icons.Edit />
                              </button>
                              <button 
                                className="btn btn-danger btn-icon-only" 
                                title="Supprimer le devis"
                                onClick={() => handleDeleteEstimate(est.id)}
                              >
                                <Icons.Delete />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* --- TAB: EXPENSES (DÉPENSES) --- */}
        {activeTab === 'expenses' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Registre des Dépenses</h1>
                <p>Enregistrez vos frais et achats professionnels pour calculer votre bénéfice réel.</p>
              </div>
              <div className="flex-gap-2">
                <button className="btn btn-primary" onClick={handleCreateExpenseClick}>
                  <Icons.Add /> Nouvelle Dépense
                </button>
              </div>
            </div>

            {/* Filter Bar */}
            <div className="filter-bar">
              <div style={{ position: 'relative', flexGrow: 1 }}>
                <span className="search-icon"><Icons.Search /></span>
                <input 
                  type="text" 
                  className="search-input" 
                  placeholder="Rechercher une dépense (Fournisseur, catégorie, description...)"
                  value={expensesSearch}
                  onChange={(e) => setExpensesSearch(e.target.value)}
                />
              </div>
            </div>

            {/* Expenses Table */}
            {expenses.length === 0 ? (
              <div className="empty-state">
                <p>Aucune dépense enregistrée. Cliquez sur "Nouvelle Dépense" pour commencer.</p>
              </div>
            ) : (
              <div className="table-container">
                <table className="table-glass">
                  <thead>
                    <tr>
                      <th>Date</th>
                      <th>Fournisseur</th>
                      <th>Catégorie</th>
                      <th>Description</th>
                      <th>Moyen</th>
                      <th>Montant HT</th>
                      <th className="text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    {expenses
                      .filter(exp => 
                        exp.merchant.toLowerCase().includes(expensesSearch.toLowerCase()) ||
                        exp.category.toLowerCase().includes(expensesSearch.toLowerCase()) ||
                        (exp.description && exp.description.toLowerCase().includes(expensesSearch.toLowerCase()))
                      )
                      .map(exp => (
                        <tr key={exp.id}>
                          <td>{new Date(exp.date).toLocaleDateString('fr-FR')}</td>
                          <td style={{ fontWeight: 600, color: 'var(--color-gold)' }}>{exp.merchant}</td>
                          <td>
                            <span className="badge badge-secondary">{exp.category}</span>
                          </td>
                          <td>{exp.description}</td>
                          <td>
                            {exp.paymentMethod === 'carte' && '💳 Carte'}
                            {exp.paymentMethod === 'virement' && '🏦 Virement'}
                            {exp.paymentMethod === 'prelevement' && '🔄 Prélèv.'}
                            {exp.paymentMethod === 'especes' && '💵 Espèces'}
                            {exp.paymentMethod === 'cheque' && '✉️ Chèque'}
                          </td>
                          <td style={{ fontWeight: 600, color: '#FF6B8B' }}>{exp.amount.toFixed(2)} €</td>
                          <td className="text-right">
                            <div className="flex-gap-2" style={{ justifyContent: 'flex-end' }}>
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Modifier la dépense"
                                onClick={() => handleEditExpenseClick(exp)}
                              >
                                <Icons.Edit />
                              </button>
                              <button 
                                className="btn btn-danger btn-icon-only" 
                                title="Supprimer la dépense"
                                onClick={() => handleDeleteExpense(exp.id)}
                              >
                                <Icons.Delete />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        )}

        {/* --- TAB: COMPTABILITÉ & DÉCLARATIONS --- */}
        {activeTab === 'compta' && (
          <div>
            <div className="page-header">
              <div className="page-title-container">
                <h1>Comptabilité & Déclarations</h1>
                <p>Suivez vos recettes encaissées, estimez vos cotisations et surveillez vos seuils légaux.</p>
              </div>
            </div>

            {/* Sub-navigation Tabs */}
            <div className="flex-gap-2" style={{ borderBottom: '1px solid var(--border-glass)', paddingBottom: '0.75rem', marginBottom: '1.5rem' }}>
              <button 
                className={`btn ${comptaActiveTab === 'recettes' ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => setComptaActiveTab('recettes')}
              >
                Livre des Recettes
              </button>
              <button 
                className={`btn ${comptaActiveTab === 'urssaf' ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => setComptaActiveTab('urssaf')}
              >
                Déclaration URSSAF
              </button>
              <button 
                className={`btn ${comptaActiveTab === 'seuils' ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => setComptaActiveTab('seuils')}
              >
                Seuils de Chiffre d'Affaires
              </button>
            </div>

            {/* SUBTAB: RECETTES (LIVRE DES RECETTES) */}
            {comptaActiveTab === 'recettes' && (
              <div>
                <div className="flex-between" style={{ marginBottom: '1rem' }}>
                  <h3 style={{ fontFamily: 'var(--font-title)' }}>Registre Chronologique des Recettes Encaissées</h3>
                  <button 
                    className="btn btn-secondary"
                    onClick={() => {
                      // Export to CSV
                      const paidInvoices = invoices.filter(inv => inv.status === 'payee').sort((a,b) => new Date(a.paymentDate || a.date) - new Date(b.paymentDate || b.date));
                      if (paidInvoices.length === 0) {
                        showAlert("Erreur", "Aucune recette encaissée à exporter.");
                        return;
                      }
                      let csv = "\uFEFFDate Encaissement;Facture;Client;Moyen de Paiement;Montant HT;Montant TTC\n";
                      paidInvoices.forEach(inv => {
                        const date = new Date(inv.paymentDate || inv.date).toLocaleDateString('fr-FR');
                        csv += `"${date}";"${inv.invoiceNumber}";"${inv.companyName}";"${inv.paymentMethod || 'Virement'}";${inv.amountHt.toFixed(2).replace('.', ',')};${inv.amountTotal.toFixed(2).replace('.', ',')}\n`;
                      });
                      
                      const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
                      const url = URL.createObjectURL(blob);
                      const link = document.createElement("a");
                      link.setAttribute("href", url);
                      link.setAttribute("download", `Livre_des_recettes_${new Date().getFullYear()}.csv`);
                      document.body.appendChild(link);
                      link.click();
                      document.body.removeChild(link);
                    }}
                  >
                    Exporter en CSV (.excel)
                  </button>
                </div>

                <div className="table-container" style={{ marginTop: '0' }}>
                  <table className="table-glass">
                    <thead>
                      <tr>
                        <th>Date Encaissement</th>
                        <th>N° Facture</th>
                        <th>Client</th>
                        <th>Moyen de Règlement</th>
                        <th>Montant HT</th>
                        <th>Montant TTC</th>
                      </tr>
                    </thead>
                    <tbody>
                      {invoices
                        .filter(inv => inv.status === 'payee')
                        .sort((a, b) => new Date(b.paymentDate || b.date) - new Date(a.paymentDate || a.date))
                        .map(inv => (
                          <tr key={inv.id}>
                            <td style={{ fontWeight: 600, color: 'var(--color-gold)' }}>
                              {new Date(inv.paymentDate || inv.date).toLocaleDateString('fr-FR')}
                            </td>
                            <td>{inv.invoiceNumber}</td>
                            <td>{inv.companyName}</td>
                            <td style={{ textTransform: 'capitalize' }}>
                              {inv.paymentMethod === 'carte' && '💳 Carte'}
                              {inv.paymentMethod === 'virement' && '🏦 Virement'}
                              {inv.paymentMethod === 'especes' && '💵 Espèces'}
                              {inv.paymentMethod === 'cheque' && '✉️ Chèque'}
                              {(!inv.paymentMethod) && '🏦 Virement'}
                            </td>
                            <td>{inv.amountHt.toFixed(2)} €</td>
                            <td style={{ fontWeight: 600 }}>{inv.amountTotal.toFixed(2)} €</td>
                          </tr>
                        ))}
                      {invoices.filter(inv => inv.status === 'payee').length === 0 && (
                        <tr>
                          <td colSpan="6" style={{ textAlign: 'center', padding: '2rem', color: 'var(--text-secondary)' }}>
                            Aucune facture n'est encore marquée comme "Payée".
                          </td>
                        </tr>
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* SUBTAB: URSSAF DECLARATION PANEL */}
            {comptaActiveTab === 'urssaf' && (
              <div>
                <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '1rem' }}>Simulateur de Déclaration Mensuelle / Trimestrielle</h3>
                
                <div className="card-glass" style={{ marginBottom: '1.5rem', padding: '1.25rem' }}>
                  <div style={{ display: 'flex', gap: '1.5rem', alignItems: 'center', flexWrap: 'wrap' }}>
                    <div className="form-group" style={{ marginBottom: '0', minWidth: '150px' }}>
                      <label className="form-label">Type de Période</label>
                      <select className="form-input" value={periodType} onChange={(e) => setPeriodType(e.target.value)}>
                        <option value="monthly">Mensuelle</option>
                        <option value="quarterly">Trimestrielle</option>
                      </select>
                    </div>

                    <div className="form-group" style={{ marginBottom: '0', minWidth: '120px' }}>
                      <label className="form-label">Année</label>
                      <select className="form-input" value={selectedYear} onChange={(e) => setSelectedYear(parseInt(e.target.value))}>
                        <option value={2026}>2026</option>
                        <option value={2025}>2025</option>
                      </select>
                    </div>

                    {periodType === 'monthly' ? (
                      <div className="form-group" style={{ marginBottom: '0', minWidth: '150px' }}>
                        <label className="form-label">Mois</label>
                        <select className="form-input" value={selectedMonth} onChange={(e) => setSelectedMonth(parseInt(e.target.value))}>
                          <option value={1}>Janvier</option>
                          <option value={2}>Février</option>
                          <option value={3}>Mars</option>
                          <option value={4}>Avril</option>
                          <option value={5}>Mai</option>
                          <option value={6}>Juin</option>
                          <option value={7}>Juillet</option>
                          <option value={8}>Août</option>
                          <option value={9}>Septembre</option>
                          <option value={10}>Octobre</option>
                          <option value={11}>Novembre</option>
                          <option value={12}>Décembre</option>
                        </select>
                      </div>
                    ) : (
                      <div className="form-group" style={{ marginBottom: '0', minWidth: '150px' }}>
                        <label className="form-label">Trimestre</label>
                        <select className="form-input" value={selectedQuarter} onChange={(e) => setSelectedQuarter(parseInt(e.target.value))}>
                          <option value={1}>T1 (Jan - Fév - Mar)</option>
                          <option value={2}>T2 (Avr - Mai - Jun)</option>
                          <option value={3}>T3 (Jul - Aoû - Sep)</option>
                          <option value={4}>T4 (Oct - Nov - Déc)</option>
                        </select>
                      </div>
                    )}
                  </div>
                </div>

                {/* Calculate Period Data */}
                {(() => {
                  let bncHt = 0;
                  let bicHt = 0;
                  let venteHt = 0;
                  
                  invoices.forEach(inv => {
                    if (inv.status !== 'payee') return;
                    
                    const paymentDate = new Date(inv.paymentDate || inv.date);
                    if (paymentDate.getFullYear() !== selectedYear) return;
                    
                    const month = paymentDate.getMonth() + 1; // 1-12
                    
                    let matchesPeriod = false;
                    if (periodType === 'monthly') {
                      matchesPeriod = month === selectedMonth;
                    } else {
                      const quarter = Math.floor((month - 1) / 3) + 1;
                      matchesPeriod = quarter === selectedQuarter;
                    }
                    
                    if (matchesPeriod) {
                      if (inv.serviceType === 'service_bnc') bncHt += inv.amountHt;
                      else if (inv.serviceType === 'service_bic') bicHt += inv.amountHt;
                      else if (inv.serviceType === 'vente') venteHt += inv.amountHt;
                    }
                  });

                  const bncRate = businessSettings.acreEnabled ? (businessSettings.urssafServiceBnc / 2) : businessSettings.urssafServiceBnc;
                  const bicRate = businessSettings.acreEnabled ? (businessSettings.urssafServiceBic / 2) : businessSettings.urssafServiceBic;
                  const venteRate = businessSettings.acreEnabled ? (businessSettings.urssafVente / 2) : businessSettings.urssafVente;

                  const bncCharges = (bncHt * bncRate) / 100;
                  const bicCharges = (bicHt * bicRate) / 100;
                  const venteCharges = (venteHt * venteRate) / 100;
                  
                  const totalPeriodCa = bncHt + bicHt + venteHt;
                  const totalPeriodCharges = bncCharges + bicCharges + venteCharges;

                  return (
                    <div style={{ display: 'grid', gridTemplateColumns: '1.8fr 1.2fr', gap: '1.5rem' }}>
                      <div className="card-glass" style={{ padding: '1.5rem' }}>
                        <h4 style={{ fontFamily: 'var(--font-title)', color: 'var(--color-gold)', marginBottom: '1.25rem' }}>
                          Montants à déclarer à l'URSSAF
                        </h4>
                        
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                          <div className="flex-between" style={{ paddingBottom: '0.75rem', borderBottom: '1px solid var(--border-glass)' }}>
                            <div>
                              <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>Prestations de Services Libérales (BNC)</div>
                              <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Taux de cotisation appliqué : {bncRate}%</span>
                            </div>
                            <div style={{ textAlign: 'right' }}>
                              <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--color-gold)' }}>{bncHt.toFixed(2)} €</div>
                              <span style={{ fontSize: '0.75rem', color: '#FF6B8B' }}>Cotisations : {bncCharges.toFixed(2)} €</span>
                            </div>
                          </div>

                          <div className="flex-between" style={{ paddingBottom: '0.75rem', borderBottom: '1px solid var(--border-glass)' }}>
                            <div>
                              <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>Prestations de Services Artisanales/Comm. (BIC)</div>
                              <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Taux de cotisation appliqué : {bicRate}%</span>
                            </div>
                            <div style={{ textAlign: 'right' }}>
                              <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--color-gold)' }}>{bicHt.toFixed(2)} €</div>
                              <span style={{ fontSize: '0.75rem', color: '#FF6B8B' }}>Cotisations : {bicCharges.toFixed(2)} €</span>
                            </div>
                          </div>

                          <div className="flex-between" style={{ paddingBottom: '0.75rem', borderBottom: '1px solid var(--border-glass)' }}>
                            <div>
                              <div style={{ fontWeight: 600, fontSize: '0.95rem' }}>Achat / Vente de Marchandises (BIC)</div>
                              <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>Taux de cotisation appliqué : {venteRate}%</span>
                            </div>
                            <div style={{ textAlign: 'right' }}>
                              <div style={{ fontSize: '1.15rem', fontWeight: 700, color: 'var(--color-gold)' }}>{venteHt.toFixed(2)} €</div>
                              <span style={{ fontSize: '0.75rem', color: '#FF6B8B' }}>Cotisations : {venteCharges.toFixed(2)} €</span>
                            </div>
                          </div>
                        </div>

                        <div className="flex-between" style={{ marginTop: '1.5rem', padding: '1rem', backgroundColor: 'rgba(255,255,255,0.02)', borderRadius: '8px', border: '1px solid var(--border-glass)' }}>
                          <span style={{ fontWeight: 700 }}>Total Charges Période</span>
                          <span style={{ fontSize: '1.25rem', fontWeight: 800, color: '#FF6B8B' }}>{totalPeriodCharges.toFixed(2)} €</span>
                        </div>
                      </div>

                      <div className="card-glass" style={{ padding: '1.5rem', display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}>
                        <div>
                          <h4 style={{ fontFamily: 'var(--font-title)', marginBottom: '1rem' }}>Informations de déclaration</h4>
                          <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', lineHeight: '1.5', marginBottom: '1rem' }}>
                            Pour déclarer vos cotisations, rendez-vous sur votre espace professionnel <strong>autoentrepreneur.urssaf.fr</strong>.
                          </p>
                          <div style={{ padding: '1rem', backgroundColor: 'rgba(229,169,60,0.03)', borderRadius: '8px', border: '1px dashed rgba(229,169,60,0.2)', fontSize: '0.85rem', lineHeight: '1.5' }}>
                            📌 <strong>Rappel :</strong> Vous devez déclarer le Chiffre d'Affaires <strong>réellement encaissé</strong> au cours de la période sélectionnée, et non le montant facturé non payé.
                          </div>
                        </div>
                        
                        <div style={{ marginTop: '1.5rem' }}>
                          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginBottom: '0.25rem' }}>CA Encaissé sur la période</div>
                          <div style={{ fontSize: '1.75rem', fontWeight: 800, color: 'var(--color-gold)' }}>{totalPeriodCa.toFixed(2)} €</div>
                        </div>
                      </div>
                    </div>
                  );
                })()}
              </div>
            )}

            {/* SUBTAB: SEUILS DE CA & TVA */}
            {comptaActiveTab === 'seuils' && (
              <div>
                <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '1.25rem' }}>Surveillance des Seuils (Plafonds Annuels)</h3>
                
                {(() => {
                  // Calculate cumulative CA by category for current year
                  const currentYear = new Date().getFullYear();
                  let serviceCa = 0; // Services BNC + Services BIC
                  let venteCa = 0; // Vente BIC
                  
                  invoices.forEach(inv => {
                    if (inv.status !== 'payee') return;
                    const paymentDate = new Date(inv.paymentDate || inv.date);
                    if (paymentDate.getFullYear() === currentYear) {
                      if (inv.serviceType === 'vente') {
                        venteCa += inv.amountHt;
                      } else {
                        serviceCa += inv.amountHt;
                      }
                    }
                  });

                  // Thresholds definition
                  const limitTvaService = 36800;
                  const toleranceTvaService = 39100;
                  const limitMicroService = 77700;

                  const limitTvaVente = 91900;
                  const toleranceTvaVente = 101000;
                  const limitMicroVente = 188700;

                  // Percentages
                  const pctTvaService = Math.min((serviceCa / toleranceTvaService) * 100, 100);
                  const pctMicroService = Math.min((serviceCa / limitMicroService) * 100, 100);
                  
                  const pctTvaVente = Math.min((venteCa / toleranceTvaVente) * 100, 100);
                  const pctMicroVente = Math.min((venteCa / limitMicroVente) * 100, 100);

                  return (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
                      {/* Section : Services */}
                      <div className="card-glass" style={{ padding: '1.5rem' }}>
                        <h4 style={{ fontFamily: 'var(--font-title)', color: 'var(--color-gold)', marginBottom: '1rem' }}>
                          Activités de Services (Plafonds pour {currentYear})
                        </h4>
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2rem' }}>
                          {/* TVA Services */}
                          <div>
                            <div className="flex-between" style={{ marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                              <span>Seuil Franchise de TVA (Services)</span>
                              <span style={{ fontWeight: 600 }}>{serviceCa.toLocaleString('fr-FR')} € / {toleranceTvaService.toLocaleString('fr-FR')} €</span>
                            </div>
                            <div style={{ height: '10px', backgroundColor: 'rgba(255,255,255,0.05)', borderRadius: '5px', overflow: 'hidden', marginBottom: '0.5rem' }}>
                              <div style={{ height: '100%', width: `${pctTvaService}%`, background: serviceCa > limitTvaService ? 'linear-gradient(90deg, #FF6B8B, #EF4444)' : 'linear-gradient(90deg, var(--color-gold), var(--color-gold-hover))', borderRadius: '5px' }}></div>
                            </div>
                            <span style={{ fontSize: '0.75rem', color: serviceCa > limitTvaService ? '#FF6B8B' : 'var(--text-muted)' }}>
                              {serviceCa > limitTvaService 
                                ? "⚠️ Vous avez dépassé le seuil de franchise. Vous devez facturer de la TVA."
                                : `Il vous reste ${(toleranceTvaService - serviceCa).toLocaleString('fr-FR')} € de marge avant d'assujettir vos services à la TVA.`}
                            </span>
                          </div>

                          {/* Plafond Régime Services */}
                          <div>
                            <div className="flex-between" style={{ marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                              <span>Plafond Régime Micro-Entreprise (Services)</span>
                              <span style={{ fontWeight: 600 }}>{serviceCa.toLocaleString('fr-FR')} € / {limitMicroService.toLocaleString('fr-FR')} €</span>
                            </div>
                            <div style={{ height: '10px', backgroundColor: 'rgba(255,255,255,0.05)', borderRadius: '5px', overflow: 'hidden', marginBottom: '0.5rem' }}>
                              <div style={{ height: '100%', width: `${pctMicroService}%`, background: 'linear-gradient(90deg, var(--color-blue), #38BDF8)', borderRadius: '5px' }}></div>
                            </div>
                            <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                              Plafond légal au-delà duquel vous devez basculer vers un régime réel d'imposition.
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* Section : Ventes */}
                      <div className="card-glass" style={{ padding: '1.5rem' }}>
                        <h4 style={{ fontFamily: 'var(--font-title)', color: 'var(--color-gold)', marginBottom: '1rem' }}>
                          Activités d'Achat / Vente de Marchandises (Plafonds pour {currentYear})
                        </h4>
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2rem' }}>
                          {/* TVA Ventes */}
                          <div>
                            <div className="flex-between" style={{ marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                              <span>Seuil Franchise de TVA (Ventes)</span>
                              <span style={{ fontWeight: 600 }}>{venteCa.toLocaleString('fr-FR')} € / {toleranceTvaVente.toLocaleString('fr-FR')} €</span>
                            </div>
                            <div style={{ height: '10px', backgroundColor: 'rgba(255,255,255,0.05)', borderRadius: '5px', overflow: 'hidden', marginBottom: '0.5rem' }}>
                              <div style={{ height: '100%', width: `${pctTvaVente}%`, background: venteCa > limitTvaVente ? 'linear-gradient(90deg, #FF6B8B, #EF4444)' : 'linear-gradient(90deg, var(--color-gold), var(--color-gold-hover))', borderRadius: '5px' }}></div>
                            </div>
                            <span style={{ fontSize: '0.75rem', color: venteCa > limitTvaVente ? '#FF6B8B' : 'var(--text-muted)' }}>
                              {venteCa > limitTvaVente 
                                ? "⚠️ Vous avez dépassé le seuil de franchise. Vous devez facturer de la TVA."
                                : `Il vous reste ${(toleranceTvaVente - venteCa).toLocaleString('fr-FR')} € de marge avant d'assujettir vos ventes à la TVA.`}
                            </span>
                          </div>

                          {/* Plafond Régime Ventes */}
                          <div>
                            <div className="flex-between" style={{ marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                              <span>Plafond Régime Micro-Entreprise (Ventes)</span>
                              <span style={{ fontWeight: 600 }}>{venteCa.toLocaleString('fr-FR')} € / {limitMicroVente.toLocaleString('fr-FR')} €</span>
                            </div>
                            <div style={{ height: '10px', backgroundColor: 'rgba(255,255,255,0.05)', borderRadius: '5px', overflow: 'hidden', marginBottom: '0.5rem' }}>
                              <div style={{ height: '100%', width: `${pctMicroVente}%`, background: 'linear-gradient(90deg, var(--color-blue), #38BDF8)', borderRadius: '5px' }}></div>
                            </div>
                            <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                              Plafond légal d'activité pour l'achat / revente de marchandises.
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })()}
              </div>
            )}
          </div>
        )}
      </main>

      {/* --- MODAL: CLIENT ADD/EDIT --- */}
      {clientModalOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>{clientForm.id ? 'Modifier le client' : 'Ajouter un client'}</h2>
              <button 
                className="btn btn-secondary btn-icon-only" 
                style={{ borderRadius: '50%' }}
                onClick={() => setClientModalOpen(false)}
              >
                ✕
              </button>
            </div>
            
            <form onSubmit={handleSaveClient}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Nom de l'entreprise</label>
                  <input 
                    type="text" 
                    className={`form-input ${clientFormErrors.companyName ? 'error' : ''}`}
                    value={clientForm.companyName}
                    onChange={(e) => setClientForm({ ...clientForm, companyName: e.target.value })}
                    placeholder="Ex: Stark Industries"
                  />
                  {clientFormErrors.companyName && <span className="error-text">{clientFormErrors.companyName}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Nom du contact</label>
                  <input 
                    type="text" 
                    className={`form-input ${clientFormErrors.contactName ? 'error' : ''}`}
                    value={clientForm.contactName}
                    onChange={(e) => setClientForm({ ...clientForm, contactName: e.target.value })}
                    placeholder="Ex: Pepper Potts"
                  />
                  {clientFormErrors.contactName && <span className="error-text">{clientFormErrors.contactName}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Email</label>
                  <input 
                    type="text" 
                    className={`form-input ${clientFormErrors.email ? 'error' : ''}`}
                    value={clientForm.email}
                    onChange={(e) => setClientForm({ ...clientForm, email: e.target.value })}
                    placeholder="Ex: contact@entreprise.fr"
                  />
                  {clientFormErrors.email && <span className="error-text">{clientFormErrors.email}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Numéro de téléphone</label>
                  <input 
                    type="text" 
                    className={`form-input ${clientFormErrors.phone ? 'error' : ''}`}
                    value={clientForm.phone}
                    onChange={handlePhoneInput}
                    placeholder="Ex: 06 12 34 56 78"
                  />
                  {clientFormErrors.phone && <span className="error-text">{clientFormErrors.phone}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Adresse de l'entreprise</label>
                  <input 
                    type="text" 
                    className={`form-input ${clientFormErrors.address ? 'error' : ''}`}
                    value={clientForm.address}
                    onChange={(e) => setClientForm({ ...clientForm, address: e.target.value })}
                    placeholder="Ex: 12 Rue de la Paix, 75002 Paris"
                  />
                  {clientFormErrors.address && <span className="error-text">{clientFormErrors.address}</span>}
                </div>
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setClientModalOpen(false)}>
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  {clientForm.id ? 'Modifier' : 'Ajouter'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* --- MODAL: INVOICE ADD --- */}
      {invoiceModalOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Générer une facture</h2>
              <button 
                className="btn btn-secondary btn-icon-only" 
                style={{ borderRadius: '50%' }}
                onClick={() => setInvoiceModalOpen(false)}
              >
                ✕
              </button>
            </div>
            
            <form onSubmit={handleSaveInvoice}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Client facturé</label>
                  <select 
                    className="form-input"
                    value={invoiceForm.clientId}
                    onChange={(e) => setInvoiceForm({ ...invoiceForm, clientId: e.target.value })}
                  >
                    {clients.map(c => (
                      <option key={c.id} value={c.id}>{c.companyName} ({c.contactName})</option>
                    ))}
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Type d'activité</label>
                  <select 
                    className="form-input"
                    value={invoiceForm.serviceType}
                    onChange={(e) => setInvoiceForm({ ...invoiceForm, serviceType: e.target.value })}
                  >
                    <option value="service_bnc">Prestation de service - Profession Libérale (BNC)</option>
                    <option value="service_bic">Prestation de service - Artisanale / Commerciale (BIC)</option>
                    <option value="vente">Achat / Vente de marchandises (BIC)</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Taux de TVA (%)</label>
                  <select 
                    className="form-input"
                    value={invoiceForm.tvaRate}
                    onChange={(e) => setInvoiceForm({ ...invoiceForm, tvaRate: e.target.value })}
                  >
                    <option value="0">0% (Franchise en base de TVA)</option>
                    <option value="5.5">5.5% (Taux réduit)</option>
                    <option value="10">10% (Taux intermédiaire)</option>
                    <option value="20">20% (Taux standard)</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Montant Hors Taxes (HT) en €</label>
                  <input 
                    type="number" 
                    step="0.01"
                    className={`form-input ${invoiceFormErrors.amountHt ? 'error' : ''}`}
                    value={invoiceForm.amountHt}
                    onChange={(e) => setInvoiceForm({ ...invoiceForm, amountHt: e.target.value })}
                    placeholder="Ex: 1500.00"
                  />
                  {invoiceFormErrors.amountHt && <span className="error-text">{invoiceFormErrors.amountHt}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Description de la prestation</label>
                  <textarea 
                    className={`form-input ${invoiceFormErrors.description ? 'error' : ''}`}
                    style={{ minHeight: '80px', resize: 'vertical' }}
                    value={invoiceForm.description}
                    onChange={(e) => setInvoiceForm({ ...invoiceForm, description: e.target.value })}
                    placeholder="Détaillez le travail effectué..."
                  />
                  {invoiceFormErrors.description && <span className="error-text">{invoiceFormErrors.description}</span>}
                </div>

                {/* Live calculation preview */}
                {invoiceForm.amountHt && !isNaN(parseFloat(invoiceForm.amountHt)) && (
                  <div style={{ marginTop: '1rem', padding: '1rem', backgroundColor: 'rgba(255,255,255,0.02)', borderRadius: '8px', border: '1px solid var(--border-glass)' }}>
                    <div style={{ fontWeight: 600, fontSize: '0.9rem', marginBottom: '0.5rem', color: 'var(--color-gold)' }}>Aperçu du calcul :</div>
                    <div className="flex-between" style={{ fontSize: '0.85rem', marginVertical: '0.25rem' }}>
                      <span>Montant HT :</span>
                      <span>{parseFloat(invoiceForm.amountHt).toFixed(2)} €</span>
                    </div>
                    <div className="flex-between" style={{ fontSize: '0.85rem', marginVertical: '0.25rem' }}>
                      <span>Montant TVA ({invoiceForm.tvaRate}%) :</span>
                      <span>{((parseFloat(invoiceForm.amountHt) * parseFloat(invoiceForm.tvaRate)) / 100).toFixed(2)} €</span>
                    </div>
                    <div className="flex-between" style={{ fontSize: '0.9rem', fontWeight: 700, borderTop: '1px solid var(--border-glass)', paddingTop: '0.5rem', marginTop: '0.5rem' }}>
                      <span>Total TTC :</span>
                      <span>{(parseFloat(invoiceForm.amountHt) + (parseFloat(invoiceForm.amountHt) * parseFloat(invoiceForm.tvaRate)) / 100).toFixed(2)} €</span>
                    </div>
                  </div>
                )}
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setInvoiceModalOpen(false)}>
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Générer & Enregistrer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* --- MODAL: ESTIMATE ADD/EDIT --- */}
      {estimateModalOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>{estimateForm.id ? 'Modifier le devis' : 'Générer un devis'}</h2>
              <button 
                className="btn btn-secondary btn-icon-only" 
                style={{ borderRadius: '50%' }}
                onClick={() => setEstimateModalOpen(false)}
              >
                ✕
              </button>
            </div>
            
            <form onSubmit={handleSaveEstimate}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Client</label>
                  <select 
                    className="form-input"
                    value={estimateForm.clientId}
                    onChange={(e) => setEstimateForm({ ...estimateForm, clientId: e.target.value })}
                  >
                    {clients.map(c => (
                      <option key={c.id} value={c.id}>{c.companyName} ({c.contactName})</option>
                    ))}
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Type d'activité</label>
                  <select 
                    className="form-input"
                    value={estimateForm.serviceType}
                    onChange={(e) => setEstimateForm({ ...estimateForm, serviceType: e.target.value })}
                  >
                    <option value="service_bnc">Prestation de service - Profession Libérale (BNC)</option>
                    <option value="service_bic">Prestation de service - Artisanale / Commerciale (BIC)</option>
                    <option value="vente">Achat / Vente de marchandises (BIC)</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Taux de TVA (%)</label>
                  <select 
                    className="form-input"
                    value={estimateForm.tvaRate}
                    onChange={(e) => setEstimateForm({ ...estimateForm, tvaRate: e.target.value })}
                  >
                    <option value="0">0% (Franchise en base de TVA)</option>
                    <option value="5.5">5.5% (Taux réduit)</option>
                    <option value="10">10% (Taux intermédiaire)</option>
                    <option value="20">20% (Taux standard)</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Montant HT (€)</label>
                  <input 
                    type="number" 
                    step="0.01"
                    className={`form-input ${estimateFormErrors.amountHt ? 'error' : ''}`}
                    value={estimateForm.amountHt}
                    onChange={(e) => setEstimateForm({ ...estimateForm, amountHt: e.target.value })}
                    placeholder="Ex: 1500.00"
                  />
                  {estimateFormErrors.amountHt && <span className="error-text">{estimateFormErrors.amountHt}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Date du devis</label>
                  <input 
                    type="date" 
                    className="form-input"
                    value={estimateForm.date}
                    onChange={(e) => setEstimateForm({ ...estimateForm, date: e.target.value })}
                  />
                </div>

                {estimateForm.id && (
                  <div className="form-group">
                    <label className="form-label">Statut du devis</label>
                    <select 
                      className="form-input"
                      value={estimateForm.status}
                      onChange={(e) => setEstimateForm({ ...estimateForm, status: e.target.value })}
                    >
                      <option value="brouillon">Brouillon</option>
                      <option value="envoye">Envoyé</option>
                      <option value="accepte">Accepté</option>
                      <option value="refuse">Refusé</option>
                    </select>
                  </div>
                )}

                <div className="form-group">
                  <label className="form-label">Description des prestations</label>
                  <textarea 
                    className={`form-input ${estimateFormErrors.description ? 'error' : ''}`}
                    style={{ minHeight: '80px', resize: 'vertical' }}
                    value={estimateForm.description}
                    onChange={(e) => setEstimateForm({ ...estimateForm, description: e.target.value })}
                    placeholder="Détaillez les travaux prévus..."
                  />
                  {estimateFormErrors.description && <span className="error-text">{estimateFormErrors.description}</span>}
                </div>
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setEstimateModalOpen(false)}>
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Enregistrer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* --- MODAL: EXPENSE ADD/EDIT --- */}
      {expenseModalOpen && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>{expenseForm.id ? 'Modifier la dépense' : 'Enregistrer une dépense'}</h2>
              <button 
                className="btn btn-secondary btn-icon-only" 
                style={{ borderRadius: '50%' }}
                onClick={() => setExpenseModalOpen(false)}
              >
                ✕
              </button>
            </div>
            
            <form onSubmit={handleSaveExpense}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Fournisseur</label>
                  <input 
                    type="text" 
                    className={`form-input ${expenseFormErrors.merchant ? 'error' : ''}`}
                    value={expenseForm.merchant}
                    onChange={(e) => setExpenseForm({ ...expenseForm, merchant: e.target.value })}
                    placeholder="Ex: OVHcloud, SNCF..."
                  />
                  {expenseFormErrors.merchant && <span className="error-text">{expenseFormErrors.merchant}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Montant (€)</label>
                  <input 
                    type="number" 
                    step="0.01"
                    className={`form-input ${expenseFormErrors.amount ? 'error' : ''}`}
                    value={expenseForm.amount}
                    onChange={(e) => setExpenseForm({ ...expenseForm, amount: e.target.value })}
                    placeholder="Ex: 50.00"
                  />
                  {expenseFormErrors.amount && <span className="error-text">{expenseFormErrors.amount}</span>}
                </div>

                <div className="form-group">
                  <label className="form-label">Date de la dépense</label>
                  <input 
                    type="date" 
                    className="form-input"
                    value={expenseForm.date}
                    onChange={(e) => setExpenseForm({ ...expenseForm, date: e.target.value })}
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Catégorie</label>
                  <select 
                    className="form-input"
                    value={expenseForm.category}
                    onChange={(e) => setExpenseForm({ ...expenseForm, category: e.target.value })}
                  >
                    <option value="Achats">Achats de marchandises</option>
                    <option value="Déplacements">Déplacements / Transport</option>
                    <option value="Logiciels">Abonnements logiciels / Cloud</option>
                    <option value="Télécoms">Télécommunications / Téléphone</option>
                    <option value="Bureautique">Fournitures de bureau</option>
                    <option value="Cotisations">Autres cotisations / Assurances</option>
                    <option value="Autre">Autre frais</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Moyen de paiement</label>
                  <select 
                    className="form-input"
                    value={expenseForm.paymentMethod}
                    onChange={(e) => setExpenseForm({ ...expenseForm, paymentMethod: e.target.value })}
                  >
                    <option value="carte">💳 Carte bancaire</option>
                    <option value="virement">🏦 Virement</option>
                    <option value="prelevement">🔄 Prélèvement automatique</option>
                    <option value="especes">💵 Espèces</option>
                    <option value="cheque">✉️ Chèque</option>
                  </select>
                </div>

                <div className="form-group">
                  <label className="form-label">Description (facultative)</label>
                  <input 
                    type="text" 
                    className="form-input"
                    value={expenseForm.description}
                    onChange={(e) => setExpenseForm({ ...expenseForm, description: e.target.value })}
                    placeholder="Ex: VPS mois de juin..."
                  />
                </div>
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setExpenseModalOpen(false)}>
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary">
                  Enregistrer
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* --- MODAL: MARK AS PAID --- */}
      {paymentModalOpen && (
        <div className="modal-overlay" style={{ zIndex: 105 }}>
          <div className="modal-content" style={{ maxWidth: '450px' }}>
            <div className="modal-header">
              <h2>Enregistrer le règlement</h2>
              <button 
                className="btn btn-secondary btn-icon-only" 
                style={{ borderRadius: '50%' }}
                onClick={() => setPaymentModalOpen(false)}
              >
                ✕
              </button>
            </div>
            
            <form onSubmit={handleSavePayment}>
              <div className="modal-body">
                <p style={{ marginBottom: '1.25rem', fontSize: '0.9rem', color: 'var(--text-secondary)' }}>
                  Veuillez renseigner la date et le moyen de paiement pour marquer la facture <strong>{paymentForm.invoiceNumber}</strong> comme payée.
                </p>

                <div className="form-group">
                  <label className="form-label">Date d'encaissement</label>
                  <input 
                    type="date" 
                    className="form-input"
                    value={paymentForm.paymentDate}
                    onChange={(e) => setPaymentForm({ ...paymentForm, paymentDate: e.target.value })}
                    required
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Moyen de règlement</label>
                  <select 
                    className="form-input"
                    value={paymentForm.paymentMethod}
                    onChange={(e) => setPaymentForm({ ...paymentForm, paymentMethod: e.target.value })}
                  >
                    <option value="virement">🏦 Virement bancaire</option>
                    <option value="carte">💳 Carte bancaire</option>
                    <option value="especes">💵 Espèces</option>
                    <option value="cheque">✉️ Chèque</option>
                  </select>
                </div>
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setPaymentModalOpen(false)}>
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary" style={{ background: 'linear-gradient(135deg, var(--color-gold), var(--color-gold-hover))', color: 'var(--bg-primary)' }}>
                  Valider le paiement
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* --- MODAL: EMAIL SEND --- */}
      {emailModalOpen && (
        <div className="modal-overlay">
          <div className="modal-content" style={{ maxWidth: '600px' }}>
            <div className="modal-header">
              <h2>Envoyer la facture par e-mail</h2>
              <button 
                type="button"
                className="btn btn-secondary btn-icon-only" 
                style={{ borderRadius: '50%' }}
                onClick={() => setEmailModalOpen(false)}
              >
                ✕
              </button>
            </div>
            
            <form onSubmit={handleSendEmail}>
              <div className="modal-body">
                <div className="form-group">
                  <label className="form-label">Destinataire (E-mail)</label>
                  <input 
                    type="email" 
                    className="form-input"
                    value={emailForm.to}
                    onChange={(e) => setEmailForm({ ...emailForm, to: e.target.value })}
                    required
                    placeholder="client@entreprise.fr"
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Objet du message</label>
                  <input 
                    type="text" 
                    className="form-input"
                    value={emailForm.subject}
                    onChange={(e) => setEmailForm({ ...emailForm, subject: e.target.value })}
                    required
                  />
                </div>

                <div className="form-group">
                  <label className="form-label">Corps du message</label>
                  <textarea 
                    className="form-input"
                    style={{ minHeight: '180px', resize: 'vertical', fontFamily: 'inherit', lineHeight: '1.5' }}
                    value={emailForm.text}
                    onChange={(e) => setEmailForm({ ...emailForm, text: e.target.value })}
                    required
                  />
                </div>

                {/* Attachment Pill */}
                {emailForm.invoice && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.75rem 1rem', backgroundColor: 'rgba(229, 169, 60, 0.05)', borderRadius: '8px', border: '1px solid rgba(229, 169, 60, 0.15)', fontSize: '0.85rem' }}>
                    <span style={{ fontSize: '1.15rem' }}>📎</span>
                    <div style={{ flexGrow: 1 }}>
                      <div style={{ fontWeight: 600, color: 'var(--color-gold)' }}>{emailForm.invoice.invoiceNumber}.pdf</div>
                      <div style={{ color: 'var(--text-muted)', fontSize: '0.75rem' }}>Document PDF généré automatiquement et joint au message</div>
                    </div>
                  </div>
                )}
              </div>

              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" onClick={() => setEmailModalOpen(false)} disabled={sendingEmail}>
                  Annuler
                </button>
                <button type="submit" className="btn btn-primary" disabled={sendingEmail} style={{ minWidth: '120px' }}>
                  {sendingEmail ? "Envoi en cours..." : "Envoyer"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* --- CUSTOM DIALOGS --- */}
      {customAlert.open && (
        <div className="modal-overlay" style={{ zIndex: 110 }}>
          <div className="modal-content" style={{ maxWidth: '400px' }}>
            <div className="modal-header">
              <h2>{customAlert.title}</h2>
              <button type="button" className="btn btn-secondary btn-icon-only" style={{ borderRadius: '50%' }} onClick={closeAlert}>✕</button>
            </div>
            <div className="modal-body">
              <p style={{ fontSize: '0.95rem', color: 'var(--text-secondary)', lineHeight: '1.5' }}>{customAlert.message}</p>
            </div>
            <div className="modal-footer">
              <button type="button" className="btn btn-primary" onClick={closeAlert}>OK</button>
            </div>
          </div>
        </div>
      )}

      {customConfirm.open && (
        <div className="modal-overlay" style={{ zIndex: 110 }}>
          <div className="modal-content" style={{ maxWidth: '450px' }}>
            <div className="modal-header">
              <h2>{customConfirm.title}</h2>
              <button type="button" className="btn btn-secondary btn-icon-only" style={{ borderRadius: '50%' }} onClick={closeConfirm}>✕</button>
            </div>
            <div className="modal-body">
              <p style={{ fontSize: '0.95rem', color: 'var(--text-secondary)', lineHeight: '1.5' }}>{customConfirm.message}</p>
            </div>
            <div className="modal-footer">
              <button type="button" className="btn btn-secondary" onClick={closeConfirm}>Annuler</button>
              <button type="button" className="btn btn-danger" onClick={() => { customConfirm.onConfirm(); closeConfirm(); }}>Confirmer</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
