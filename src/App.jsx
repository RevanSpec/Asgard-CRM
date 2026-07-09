import React, { useState, useEffect } from 'react';
import { db, generateInvoiceNumber } from './db';
import { exportInvoiceToPDF } from './pdfGenerator';

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
  acreEnabled: false
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
          date: new Date('2026-02-05').toISOString()
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
          date: new Date('2026-03-12').toISOString()
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
          date: new Date('2026-04-18').toISOString()
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
          date: new Date('2026-05-02').toISOString()
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
          date: new Date('2026-06-25').toISOString()
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
          date: new Date('2026-07-01').toISOString()
        }
      ];

      for (const inv of invoicesSeed) {
        await db.invoices.add(inv);
      }
    }
  };

  // Fetch all data
  const loadAllData = async () => {
    const clientsList = await db.clients.toArray();
    const invoicesList = await db.invoices.toArray();
    
    // Sort clients by name
    setClients(clientsList.sort((a, b) => a.companyName.localeCompare(b.companyName)));
    // Sort invoices by date descending
    setInvoices(invoicesList.sort((a, b) => new Date(b.date) - new Date(a.date)));
  };

  // Load Settings from LocalStorage
  useEffect(() => {
    const saved = localStorage.getItem('asgard_crm_settings');
    if (saved) {
      setBusinessSettings(JSON.parse(saved));
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

  // --- CALCULATION HELPERS FOR DASHBOARD ---

  const calculateCA = () => {
    let ht = 0;
    let ttc = 0;
    invoices.forEach(inv => {
      ht += inv.amountHt;
      ttc += inv.amountTotal;
    });
    return { ht, ttc };
  };

  const calculateUrssafCharges = () => {
    let charges = 0;
    invoices.forEach(inv => {
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

  const getServiceTypeBreakdown = () => {
    let bnc = 0;
    let bic = 0;
    let vente = 0;
    invoices.forEach(inv => {
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

  const { ht: totalCaHt, ttc: totalCaTtc } = calculateCA();
  const totalUrssaf = calculateUrssafCharges();
  const breakdown = getServiceTypeBreakdown();
  const monthlyCA = getMonthlyCAData();

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
            className={`nav-item ${activeTab === 'invoices' ? 'active' : ''}`}
            onClick={() => setActiveTab('invoices')}
          >
            <Icons.Invoices />
            <span>Factures</span>
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

            {/* Metric Grid */}
            <div className="dashboard-grid">
              <div className="card-glass">
                <div className="metric-label">CA Global (HT)</div>
                <div className="metric-value metric-highlight">{totalCaHt.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">Total net hors taxes encaissé</div>
              </div>
              <div className="card-glass">
                <div className="metric-label">CA Global (TTC)</div>
                <div className="metric-value">{totalCaTtc.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">Total avec taxe sur la valeur ajoutée</div>
              </div>
              <div className="card-glass" style={{ borderColor: 'rgba(244, 63, 94, 0.3)' }}>
                <div className="metric-label">Charges URSSAF</div>
                <div className="metric-value" style={{ color: '#FF6B8B' }}>{totalUrssaf.toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">
                  Estimées à {businessSettings.acreEnabled ? 'taux réduit ACRE' : 'taux plein'}
                </div>
              </div>
              <div className="card-glass" style={{ borderColor: 'rgba(56, 189, 248, 0.3)' }}>
                <div className="metric-label">Reste Après Charges</div>
                <div className="metric-value" style={{ color: 'var(--color-blue)' }}>{(totalCaHt - totalUrssaf).toLocaleString('fr-FR', { minimumFractionDigits: 2 })} €</div>
                <div className="metric-subtext">Trésorerie nette après cotisations</div>
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
                          <th>Montant HT</th>
                          <th>Montant TTC</th>
                          <th className="text-right">PDF</th>
                        </tr>
                      </thead>
                      <tbody>
                        {top10Invoices.map((inv) => (
                          <tr key={inv.id}>
                            <td style={{ fontWeight: 600, color: 'var(--color-gold)' }}>{inv.invoiceNumber}</td>
                            <td>{inv.companyName}</td>
                            <td>{inv.amountHt.toFixed(2)} €</td>
                            <td>{inv.amountTotal.toFixed(2)} €</td>
                            <td className="text-right">
                              <button 
                                className="btn btn-secondary btn-icon-only" 
                                title="Exporter en PDF"
                                onClick={() => handleExportPDF(inv)}
                              >
                                <Icons.Download />
                              </button>
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
                        <th>Description</th>
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
                            {inv.serviceType === 'service_bnc' && <span className="badge badge-blue">Service BNC</span>}
                            {inv.serviceType === 'service_bic' && <span className="badge badge-blue">Service BIC</span>}
                            {inv.serviceType === 'vente' && <span className="badge badge-success">Vente (BIC)</span>}
                          </td>
                          <td style={{ maxWidth: '180px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={inv.description}>
                            {inv.description}
                          </td>
                          <td>{inv.amountHt.toFixed(2)} €</td>
                          <td>{inv.tvaRate}%</td>
                          <td style={{ fontWeight: 600 }}>{inv.amountTotal.toFixed(2)} €</td>
                          <td className="text-right">
                            <div className="flex-gap-2" style={{ justifyContent: 'flex-end' }}>
                              <button 
                                className="btn btn-primary btn-icon-only" 
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
              </form>
            </div>
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
