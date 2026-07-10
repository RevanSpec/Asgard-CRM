import React, { useState, useEffect } from 'react';
import { db, generateInvoiceNumber, exportDatabaseData, importDatabaseData } from './db';
import { exportInvoiceToPDF, generateInvoicePDF, exportEstimateToPDF } from './pdfGenerator';
import Sidebar from './components/Sidebar';
import DashboardTab from './components/DashboardTab';
import ClientsTab from './components/ClientsTab';
import InvoicesTab from './components/InvoicesTab';
import EstimatesTab from './components/EstimatesTab';
import ExpensesTab from './components/ExpensesTab';
import ComptaTab from './components/ComptaTab';
import SettingsTab from './components/SettingsTab';

// Electron IPC renderer (safe for web testing too)
const ipcRenderer = window.require ? window.require('electron').ipcRenderer : null;

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
  logoBase64: '',
  emailTemplateInvoice: "Bonjour {clientName},\n\nVeuillez trouver ci-joint la facture {documentNumber} pour la prestation : {description}.\n\nLe montant total est de {amountTotal} €.\n\nCordialement,\n\n{senderName}\n{senderCompany}",
  emailTemplateEstimate: "Bonjour {clientName},\n\nVeuillez trouver ci-joint le devis {documentNumber} pour la prestation : {description}.\n\nLe montant total est de {amountTotal} €.\n\nCordialement,\n\n{senderName}\n{senderCompany}",
  emailTemplateReminder: "Bonjour {clientName},\n\nSauf erreur ou omission de notre part, nous n'avons pas reçu le règlement de la facture {documentNumber} d'un montant de {amountTotal} € envoyée le {documentDate}.\n\nNous vous prions de bien vouloir régulariser cette situation dans les plus brefs délais. Vous trouverez la facture en pièce jointe.\n\nCordialement,\n\n{senderName}\n{senderCompany}"
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

  const resolveTemplate = (template, data) => {
    if (!template) return '';
    return template
      .replace(/{clientName}/g, data.clientName || '')
      .replace(/{documentNumber}/g, data.documentNumber || '')
      .replace(/{description}/g, data.description || '')
      .replace(/{amountTotal}/g, data.amountTotal || '')
      .replace(/{dueDate}/g, data.dueDate || '')
      .replace(/{documentDate}/g, data.documentDate || '')
      .replace(/{senderName}/g, data.senderName || '')
      .replace(/{senderCompany}/g, data.senderCompany || '');
  };

  const handleOpenSendEmail = async (invoice, type = 'invoice') => {
    // Get client email
    let client = await db.clients.get(invoice.clientId);
    const toEmail = client ? client.email : '';
    const isInvoice = type === 'invoice';
    const isReminder = type === 'reminder';
    
    const num = isInvoice || isReminder ? invoice.invoiceNumber : invoice.estimateNumber;
    const clientName = client ? client.companyName : '';
    
    // Select template
    let template = '';
    let subject = '';
    
    if (isInvoice) {
      template = businessSettings.emailTemplateInvoice || defaultSettings.emailTemplateInvoice;
      subject = `Facture ${num} - ${businessSettings.companyName}`;
    } else if (isReminder) {
      template = businessSettings.emailTemplateReminder || defaultSettings.emailTemplateReminder;
      subject = `Rappel : Facture impayée ${num} - ${businessSettings.companyName}`;
    } else {
      template = businessSettings.emailTemplateEstimate || defaultSettings.emailTemplateEstimate;
      subject = `Devis ${num} - ${businessSettings.companyName}`;
    }
    
    const data = {
      clientName: clientName,
      documentNumber: num,
      description: invoice.description || '',
      amountTotal: invoice.amountTotal.toFixed(2),
      dueDate: invoice.dueDate ? new Date(invoice.dueDate).toLocaleDateString('fr-FR') : '',
      documentDate: invoice.date ? new Date(invoice.date).toLocaleDateString('fr-FR') : '',
      senderName: businessSettings.contactName,
      senderCompany: businessSettings.companyName
    };
    
    const text = resolveTemplate(template, data);
    
    setEmailForm({
      to: toEmail,
      subject: subject,
      text: text,
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
      const isInvoice = emailForm.type === 'invoice' || emailForm.type === 'reminder';
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

  const getMonthlyFinancialsData = () => {
    const months = ['Jan', 'Fév', 'Mar', 'Avr', 'Mai', 'Jun', 'Jul', 'Aoû', 'Sep', 'Oct', 'Nov', 'Déc'];
    const currentYear = new Date().getFullYear();
    
    const caPerMonth = Array(12).fill(0);
    const expensesPerMonth = Array(12).fill(0);
    const chargesPerMonth = Array(12).fill(0);
    const profitPerMonth = Array(12).fill(0);
    
    // 1. Calculate CA & theoretical URSSAF charges per month
    invoices.forEach(inv => {
      if (inv.status === 'brouillon') return;
      const invDate = new Date(inv.date);
      if (invDate.getFullYear() === currentYear) {
        const monthIndex = invDate.getMonth();
        caPerMonth[monthIndex] += inv.amountHt;
        
        // Calculate URSSAF charge rate for this invoice
        let rate = 0;
        if (inv.serviceType === 'service_bnc') {
          rate = businessSettings.urssafServiceBnc;
        } else if (inv.serviceType === 'service_bic') {
          rate = businessSettings.urssafServiceBic;
        } else if (inv.serviceType === 'vente') {
          rate = businessSettings.urssafVente;
        }
        if (businessSettings.acreEnabled) {
          rate = rate / 2;
        }
        chargesPerMonth[monthIndex] += (inv.amountHt * rate) / 100;
      }
    });
    
    // 2. Calculate expenses per month
    expenses.forEach(exp => {
      const expDate = new Date(exp.date);
      if (expDate.getFullYear() === currentYear) {
        const monthIndex = expDate.getMonth();
        expensesPerMonth[monthIndex] += exp.amount;
      }
    });
    
    // 3. Calculate profit per month
    for (let i = 0; i < 12; i++) {
      profitPerMonth[i] = caPerMonth[i] - expensesPerMonth[i] - chargesPerMonth[i];
    }
    
    const allValues = [...caPerMonth, ...profitPerMonth];
    const maxVal = Math.max(...allValues, 1000) * 1.15;
    const minVal = Math.min(...allValues, 0) * 1.15; // Support negative profit
    
    return {
      labels: months,
      caValues: caPerMonth,
      profitValues: profitPerMonth,
      maxVal,
      minVal
    };
  };

  const getExpensesCategoryData = () => {
    const categories = {
      "Achats": 0,
      "Déplacements": 0,
      "Logiciels": 0,
      "Télécoms": 0,
      "Bureautique": 0,
      "Cotisations": 0,
      "Autre": 0
    };
    
    const categoryLabels = {
      "Achats": "Achats",
      "Déplacements": "Déplacements",
      "Logiciels": "Logiciels",
      "Télécoms": "Télécoms",
      "Bureautique": "Bureautique",
      "Cotisations": "Cotisations",
      "Autre": "Autre"
    };

    let total = 0;
    expenses.forEach(exp => {
      const cat = exp.category || 'Autre';
      if (categories[cat] !== undefined) {
        categories[cat] += exp.amount;
      } else {
        categories["Autre"] += exp.amount;
      }
      total += exp.amount;
    });

    const list = Object.keys(categories).map(key => {
      const amount = categories[key];
      const pct = total > 0 ? (amount / total) * 100 : 0;
      return {
        key,
        label: categoryLabels[key] || key,
        amount,
        pct
      };
    }).filter(item => item.amount > 0); // Only keep categories with expenses

    return {
      total,
      list
    };
  };

  const { ht: totalCaHt, ttc: totalCaTtc, htFacture, ttcFacture } = calculateCA();
  const totalUrssaf = calculateUrssafCharges();
  const totalExpenses = calculateTotalExpenses();
  const netProfit = totalCaHt - totalUrssaf - totalExpenses;
  const breakdown = getServiceTypeBreakdown();
  const monthlyFinancials = getMonthlyFinancialsData();
  const expensesCategoryData = getExpensesCategoryData();

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
      <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />

      {/* --- MAIN CONTENT AREA --- */}
      <main className="main-content">
        {activeTab === 'dashboard' && (
          <DashboardTab
            businessSettings={businessSettings}
            caAlerts={caAlerts}
            totalCaHt={totalCaHt}
            htFacture={htFacture}
            totalCaTtc={totalCaTtc}
            totalExpenses={totalExpenses}
            totalUrssaf={totalUrssaf}
            netProfit={netProfit}
            monthlyFinancials={monthlyFinancials}
            expensesCategoryData={expensesCategoryData}
            breakdown={breakdown}
            top10Invoices={top10Invoices}
            top5Clients={top5Clients}
            handleOpenAddInvoice={handleOpenAddInvoice}
            handleOpenPaymentModal={handleOpenPaymentModal}
            handleOpenSendEmail={handleOpenSendEmail}
            handleExportPDF={handleExportPDF}
          />
        )}

        {activeTab === 'clients' && (
          <ClientsTab
            clientSearch={clientSearch}
            setClientSearch={setClientSearch}
            filteredClients={filteredClients}
            handleOpenAddClient={handleOpenAddClient}
            handleOpenEditClient={handleOpenEditClient}
            handleDeleteClient={handleDeleteClient}
          />
        )}

        {activeTab === 'invoices' && (
          <InvoicesTab
            selectedInvoiceIds={selectedInvoiceIds}
            setSelectedInvoiceIds={setSelectedInvoiceIds}
            invoiceSearch={invoiceSearch}
            setInvoiceSearch={setInvoiceSearch}
            filteredInvoices={filteredInvoices}
            handleDeleteSelectedInvoices={handleDeleteSelectedInvoices}
            handleOpenAddInvoice={handleOpenAddInvoice}
            handleOpenPaymentModal={handleOpenPaymentModal}
            handleOpenSendEmail={handleOpenSendEmail}
            handleExportPDF={handleExportPDF}
            handleDeleteInvoice={handleDeleteInvoice}
          />
        )}

        {activeTab === 'settings' && (
          <SettingsTab
            businessSettings={businessSettings}
            saveSettings={saveSettings}
            handleTestSMTP={handleTestSMTP}
            smtpTesting={smtpTesting}
            handleExportBackup={handleExportBackup}
            handleImportBackup={handleImportBackup}
            showAlert={showAlert}
          />
        )}

        {activeTab === 'estimates' && (
          <EstimatesTab
            estimatesSearch={estimatesSearch}
            setEstimatesSearch={setEstimatesSearch}
            estimates={estimates}
            handleCreateEstimateClick={handleCreateEstimateClick}
            handleConvertEstimateToInvoice={handleConvertEstimateToInvoice}
            handleOpenSendEmail={handleOpenSendEmail}
            handleExportEstimatePDF={handleExportEstimatePDF}
            handleEditEstimateClick={handleEditEstimateClick}
            handleDeleteEstimate={handleDeleteEstimate}
          />
        )}

        {activeTab === 'expenses' && (
          <ExpensesTab
            expensesSearch={expensesSearch}
            setExpensesSearch={setExpensesSearch}
            expenses={expenses}
            handleCreateExpenseClick={handleCreateExpenseClick}
            handleEditExpenseClick={handleEditExpenseClick}
            handleDeleteExpense={handleDeleteExpense}
          />
        )}

        {activeTab === 'compta' && (
          <ComptaTab
            comptaActiveTab={comptaActiveTab}
            setComptaActiveTab={setComptaActiveTab}
            invoices={invoices}
            periodType={periodType}
            setPeriodType={setPeriodType}
            selectedYear={selectedYear}
            setSelectedYear={setSelectedYear}
            selectedMonth={selectedMonth}
            setSelectedMonth={setSelectedMonth}
            selectedQuarter={selectedQuarter}
            setSelectedQuarter={setSelectedQuarter}
            businessSettings={businessSettings}
            showAlert={showAlert}
          />
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
