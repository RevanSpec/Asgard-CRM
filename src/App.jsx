import React, { useState, useEffect } from 'react';
import {
  loadSnapshot,
  saveClient,
  deleteClient,
  createInvoice,
  setInvoiceStatus,
  recordPayment,
  deleteInvoices,
  saveEstimate,
  deleteEstimate,
  convertEstimate,
  saveExpense,
  deleteExpense,
  exportBackup,
  importBackup,
} from './db';
import {
  exportInvoiceToPDF,
  generateInvoicePDF,
  exportEstimateToPDF,
  generateEstimatePDF,
} from './pdfGenerator';
import { computeAmounts } from './domain/money';
import { totalUrssafCharges } from './domain/urssaf';
import { buildCaAlerts } from './domain/thresholds';
import {
  calculateCA,
  calculateTotalExpenses,
  getServiceTypeBreakdown,
  getMonthlyFinancialsData,
  getExpensesCategoryData,
} from './domain/reporting';
import {
  formatPhoneInput,
  validateClientForm,
  validateInvoiceForm,
  validateEstimateForm,
  validateExpenseForm,
  isValid,
} from './domain/validation';
import { buildEmailDraft, isInvoiceKind, DOCUMENT_KINDS } from './domain/templates';
import { isDesktop, sendEmail, testSmtp, setSmtpPassword, hasSmtpPassword } from './ipc';
import Sidebar from './components/Sidebar';
import DashboardTab from './components/DashboardTab';
import ClientsTab from './components/ClientsTab';
import InvoicesTab from './components/InvoicesTab';
import EstimatesTab from './components/EstimatesTab';
import ExpensesTab from './components/ExpensesTab';
import ComptaTab from './components/ComptaTab';
import SettingsTab from './components/SettingsTab';

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
  smtpSecure: 'none',
  customColor: '#E5A93C',
  logoBase64: '',
  emailTemplateInvoice: "Bonjour {clientName},\n\nVeuillez trouver ci-joint la facture {documentNumber} pour la prestation : {description}.\n\nLe montant total est de {amountTotal} €.\n\nCordialement,\n\n{senderName}\n{senderCompany}",
  emailTemplateEstimate: "Bonjour {clientName},\n\nVeuillez trouver ci-joint le devis {documentNumber} pour la prestation : {description}.\n\nLe montant total est de {amountTotal} €.\n\nCordialement,\n\n{senderName}\n{senderCompany}",
  emailTemplateReminder: "Bonjour {clientName},\n\nSauf erreur ou omission de notre part, nous n'avons pas reçu le règlement de la facture {documentNumber} d'un montant de {amountTotal} € envoyée le {documentDate}.\n\nNous vous prions de bien vouloir régulariser cette situation dans les plus brefs délais. Vous trouverez la facture en pièce jointe.\n\nCordialement,\n\n{senderName}\n{senderCompany}"
};

/**
 * Clé du mot de passe SMTP telle qu'elle existait dans localStorage avant la
 * phase 1. Conservée uniquement pour pouvoir la retirer.
 */
const LEGACY_SMTP_PASS_KEY = 'smtpPass';

/**
 * Retire le mot de passe d'un objet de réglages.
 *
 * Appliqué au chargement, à chaque enregistrement et à l'import de sauvegarde :
 * sans cela, un ancien fichier de sauvegarde réintroduirait le secret en clair
 * dans localStorage, et le défaut D2 reviendrait par la porte de derrière.
 */
function stripSecret(settings) {
  if (!settings || !(LEGACY_SMTP_PASS_KEY in settings)) return settings;
  const { [LEGACY_SMTP_PASS_KEY]: _discarded, ...rest } = settings;
  return rest;
}

/**
 * Déplace vers le trousseau de l'OS un mot de passe encore stocké en clair.
 *
 * Les installations existantes ont le secret dans localStorage. S'en tenir à
 * « les nouvelles écritures vont ailleurs » le laisserait sur le disque
 * indéfiniment : la migration doit aussi effacer l'ancien emplacement.
 */
async function migrateLegacySmtpPassword() {
  const raw = localStorage.getItem('asgard_crm_settings');
  if (!raw) return;

  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return;
  }

  const legacy = parsed?.[LEGACY_SMTP_PASS_KEY];
  if (!(LEGACY_SMTP_PASS_KEY in (parsed || {}))) return;

  if (legacy && isDesktop()) {
    await setSmtpPassword(legacy);
  }

  localStorage.setItem('asgard_crm_settings', JSON.stringify(stripSecret(parsed)));
}

/**
 * Met en mots le compte rendu d'une reprise de sauvegarde.
 *
 * La partie qui compte est `adjustments` : le passage des montants flottants
 * aux centimes change réellement certaines valeurs — `1899,99 € × 20 %` valait
 * `379,998 €` et vaut désormais `380,00 €`. C'est plus juste, une facture ne se
 * libellant pas en fractions de centime, mais cela modifie des documents déjà
 * émis. L'utilisateur doit pouvoir l'expliquer, pas le découvrir dans une
 * déclaration.
 */
function describeImport(report) {
  const parts = [
    `${report.clients} client(s), ${report.invoices} facture(s), ` +
    `${report.estimates} devis et ${report.expenses} dépense(s) repris.`,
  ];

  if (report.adjustments.length > 0) {
    const shown = report.adjustments.slice(0, 5)
      .map((a) => `• ${a.document} — ${a.field} : ${a.before} → ${a.after} €`)
      .join('\n');
    const rest = report.adjustments.length > 5
      ? `\n… et ${report.adjustments.length - 5} autre(s).`
      : '';

    parts.push(
      `\n${report.adjustments.length} montant(s) ajusté(s) au centime. ` +
      `Vos données étaient stockées en virgule flottante ; elles le sont ` +
      `désormais en centimes, ce qui supprime les écarts d'arrondi :\n${shown}${rest}`
    );
  }

  if (report.skipped.length > 0) {
    parts.push(
      `\n${report.skipped.length} pièce(s) écartée(s) :\n• ` +
      report.skipped.slice(0, 5).join('\n• ')
    );
  }

  return parts.join('\n');
}

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

  // Le mot de passe SMTP vit dans le trousseau de l'OS, pas dans les réglages.
  // L'interface n'en garde qu'un brouillon de saisie, et un booléen disant
  // si un secret est déjà enregistré — elle ne peut jamais le relire.
  const [smtpPassDraft, setSmtpPassDraft] = useState('');
  const [smtpPassStored, setSmtpPassStored] = useState(false);

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

  // Fetch all data
  const loadAllData = async () => {
    // Un seul aller-retour : l'hôte renvoie les quatre tables déjà triées.
    const snapshot = await loadSnapshot();

    setClients(snapshot.clients);
    setInvoices(snapshot.invoices);
    setEstimates(snapshot.estimates);
    setExpenses(snapshot.expenses);
  };

  // Load Settings from LocalStorage
  useEffect(() => {
    const saved = localStorage.getItem('asgard_crm_settings');
    if (saved) {
      setBusinessSettings({ ...defaultSettings, ...stripSecret(JSON.parse(saved)) });
    } else {
      localStorage.setItem('asgard_crm_settings', JSON.stringify(defaultSettings));
    }

    const init = async () => {
      await migrateLegacySmtpPassword();
      setSmtpPassStored(await hasSmtpPassword());
      await loadAllData();
    };
    init();
  }, []);

  // Save Settings to LocalStorage & DB helper
  const saveSettings = (newSettings) => {
    const safe = stripSecret(newSettings);
    setBusinessSettings(safe);
    localStorage.setItem('asgard_crm_settings', JSON.stringify(safe));
  };

  // Confie le mot de passe saisi au trousseau de l'OS.
  const handleSaveSmtpPassword = async () => {
    if (!isDesktop()) {
      showAlert("Erreur", "Le trousseau n'est disponible que dans la version de bureau de l'application.");
      return;
    }

    await setSmtpPassword(smtpPassDraft);
    setSmtpPassStored(smtpPassDraft !== '');
    setSmtpPassDraft('');
    showAlert(
      "Enregistré",
      smtpPassDraft
        ? "Le mot de passe SMTP a été placé dans le trousseau de votre système. Il ne figure plus dans les réglages ni dans les sauvegardes."
        : "Le mot de passe SMTP a été supprimé du trousseau."
    );
  };

  // Export all DB tables and settings as a JSON file backup
  const handleExportBackup = async () => {
    try {
      const dbData = await exportBackup();
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

          const settingsData = backup.settings || null;

          // 1. Reprise des tables. L'hôte rend compte de ce qu'il a fait : les
          //    montants que l'arrondi au centime modifie, et les pièces
          //    écartées. Le rapport est affiché plus bas.
          const report = await importBackup(backup);

          // 2. Restore settings — sans le mot de passe SMTP : une sauvegarde
          //    d'avant la phase 1 le contient en clair et le réinjecterait.
          if (settingsData) {
            saveSettings(stripSecret(settingsData));
          }

          // 3. Reload state
          await loadAllData();

          showAlert("Sauvegarde restaurée", describeImport(report));
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
    setClientForm({ ...clientForm, phone: formatPhoneInput(e.target.value) });
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

  const runClientValidation = () => {
    const errors = validateClientForm(clientForm);
    setClientFormErrors(errors);
    return isValid(errors);
  };

  const handleSaveClient = async (e) => {
    e.preventDefault();
    if (!runClientValidation()) return;

    try {
      // La raison sociale est recopiée sur les pièces côté hôte, dans la même
      // transaction que la mise à jour du client.
      await saveClient({
        id: clientForm.id ?? undefined,
        companyName: clientForm.companyName,
        contactName: clientForm.contactName,
        email: clientForm.email,
        phone: clientForm.phone,
        address: clientForm.address,
      });

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
        await deleteClient(id);
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

  const runInvoiceValidation = () => {
    const errors = validateInvoiceForm(invoiceForm);
    setInvoiceFormErrors(errors);
    return isValid(errors);
  };

  const handleSaveInvoice = async (e) => {
    e.preventDefault();
    if (!runInvoiceValidation()) return;

    const selectedClient = clients.find(c => c.id === parseInt(invoiceForm.clientId));
    const { amountHt, tvaRate, amountTva, amountTotal } = computeAmounts(
      invoiceForm.amountHt,
      invoiceForm.tvaRate,
    );
    const date = new Date().toISOString();

    try {
      // Le numéro est attribué par la base, dans la transaction d'insertion
      // (défaut D3) : l'interface ne peut plus en proposer un.
      await createInvoice({
        clientId: selectedClient.id,
        companyName: selectedClient.companyName,
        serviceType: invoiceForm.serviceType,
        description: invoiceForm.description,
        amountHt,
        tvaRate,
        amountTva,
        amountTotal,
        date,
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
        const outcome = await deleteInvoices([id]);
        setSelectedInvoiceIds(prev => prev.filter(item => item !== id));
        await loadAllData();
        if (outcome.archived > 0) {
          showAlert(
            "Facture archivée",
            "Cette facture a été émise : elle est retirée de la liste mais conservée. " +
            "Le code de commerce impose dix ans de conservation — une facture émise " +
            "s'annule par un avoir, elle ne se supprime pas."
          );
        }
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
          const outcome = await deleteInvoices(selectedInvoiceIds);
          setSelectedInvoiceIds([]);
          await loadAllData();
          if (outcome.archived > 0) {
            showAlert(
              "Factures archivées",
              `${outcome.archived} facture(s) émise(s) ont été retirées de la liste mais conservées, ` +
              `comme l'impose le code de commerce. ${outcome.discarded} brouillon(s) supprimé(s).`
            );
          }
        } catch (err) {
          console.error(err);
        }
      }
    );
  };

  const handleOpenSendEmail = async (invoice, type = DOCUMENT_KINDS.INVOICE) => {
    const client = clients.find(c => c.id === invoice.clientId);
    const draft = buildEmailDraft(invoice, type, client, businessSettings, defaultSettings);

    setEmailForm({ ...draft, invoice, type });
    setEmailModalOpen(true);
  };

  const handleSendEmail = async (e) => {
    e.preventDefault();
    if (!isDesktop()) {
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
      const isInvoice = isInvoiceKind(emailForm.type);
      let client = clients.find(c => c.id === invoice.clientId);
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

      // 2. Configuration SMTP. Pas de mot de passe : l'hôte le lit dans le
      //    trousseau de l'OS au moment de l'envoi (défaut D2).
      const smtpConfig = {
        host: businessSettings.smtpHost,
        port: businessSettings.smtpPort,
        user: businessSettings.smtpUser,
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

      // 4. Envoi via la commande Tauri
      const result = await sendEmail(smtpConfig, emailData);

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
    if (!isDesktop()) {
      showAlert("Erreur", "Les fonctions de messagerie ne sont pas disponibles hors de l'application de bureau.");
      return;
    }

    setSmtpTesting(true);

    try {
      // Le test accepte un mot de passe saisi mais pas encore enregistré, pour
      // qu'on puisse valider des identifiants avant de les confier au trousseau.
      const smtpConfig = {
        host: businessSettings.smtpHost,
        port: businessSettings.smtpPort,
        user: businessSettings.smtpUser,
        pass: smtpPassDraft,
        secure: businessSettings.smtpSecure
      };

      const result = await testSmtp(smtpConfig);

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
    let client = clients.find(c => c.id === invoice.clientId);
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

  const runEstimateValidation = () => {
    const errors = validateEstimateForm(estimateForm);
    setEstimateFormErrors(errors);
    return isValid(errors);
  };

  const handleSaveEstimate = async (e) => {
    e.preventDefault();
    if (!runEstimateValidation()) return;

    const selectedClient = clients.find(c => c.id === parseInt(estimateForm.clientId));
    const { amountHt, tvaRate, amountTva, amountTotal } = computeAmounts(
      estimateForm.amountHt,
      estimateForm.tvaRate,
    );
    const date = new Date(estimateForm.date).toISOString();

    try {
      await saveEstimate({
        id: estimateForm.id ?? undefined,
        clientId: selectedClient.id,
        companyName: selectedClient.companyName,
        serviceType: estimateForm.serviceType,
        description: estimateForm.description,
        amountHt,
        tvaRate,
        amountTva,
        amountTotal,
        date,
        status: estimateForm.id ? estimateForm.status : 'brouillon',
      });

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
        const outcome = await deleteEstimate(id);
        await loadAllData();
        if (outcome.archived > 0) {
          showAlert(
            "Devis archivé",
            "Ce devis a déjà été envoyé ou accepté : il est retiré de la liste mais conservé."
          );
        }
      }
    );
  };

  const handleExportEstimatePDF = async (est) => {
    let client = clients.find(c => c.id === est.clientId);
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
          // Facture créée et devis marqué accepté dans la même transaction :
          // un échec ne peut pas laisser un devis accepté sans facture.
          const invoice = await convertEstimate(est.id);

          await loadAllData();
          setActiveTab('invoices');
          showAlert("Conversion réussie !", `Le devis a été converti en facture ${invoice.invoiceNumber} et enregistré en brouillon.`);
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

  const runExpenseValidation = () => {
    const errors = validateExpenseForm(expenseForm);
    setExpenseFormErrors(errors);
    return isValid(errors);
  };

  const handleSaveExpense = async (e) => {
    e.preventDefault();
    if (!runExpenseValidation()) return;

    const amount = parseFloat(expenseForm.amount);
    const date = new Date(expenseForm.date).toISOString();

    try {
      await saveExpense({
        id: expenseForm.id ?? undefined,
        merchant: expenseForm.merchant,
        category: expenseForm.category,
        amount,
        description: expenseForm.description,
        paymentMethod: expenseForm.paymentMethod,
        date,
      });

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
        await deleteExpense(id);
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
      await recordPayment({
        invoiceId: paymentForm.invoiceId,
        paymentDate: new Date(paymentForm.paymentDate).toISOString(),
        paymentMethod: paymentForm.paymentMethod,
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
      await setInvoiceStatus(id, 'envoyee');
      await loadAllData();
    } catch (err) {
      console.error(err);
    }
  };

  // --- AGRÉGATS DU TABLEAU DE BORD ---
  // La logique vit dans src/domain : ces appels ne font que la brancher sur l'état.
  const currentYear = new Date().getFullYear();

  const { ht: totalCaHt, ttc: totalCaTtc, htFacture, ttcFacture } = calculateCA(invoices);
  const totalUrssaf = totalUrssafCharges(invoices, businessSettings);
  const totalExpenses = calculateTotalExpenses(expenses);
  const netProfit = totalCaHt - totalUrssaf - totalExpenses;
  const breakdown = getServiceTypeBreakdown(invoices);
  const monthlyFinancials = getMonthlyFinancialsData(invoices, expenses, businessSettings, currentYear);
  const expensesCategoryData = getExpensesCategoryData(expenses);
  const caAlerts = buildCaAlerts(invoices, currentYear);

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
            smtpPassDraft={smtpPassDraft}
            setSmtpPassDraft={setSmtpPassDraft}
            smtpPassStored={smtpPassStored}
            handleSaveSmtpPassword={handleSaveSmtpPassword}
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
