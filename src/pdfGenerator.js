import { jsPDF } from 'jspdf';

/**
 * Generates and downloads a professional invoice PDF.
 * @param {object} invoice - The invoice data
 * @param {object} client - The client data
 * @param {object} businessSettings - The user's company/business settings
 */
export function exportInvoiceToPDF(invoice, client, businessSettings) {
  const doc = new jsPDF({
    orientation: 'portrait',
    unit: 'mm',
    format: 'a4',
  });

  // Color Palette
  const primaryColor = [11, 15, 25]; // Dark Blue #0B0F19
  const accentColor = [229, 169, 60]; // Gold #E5A93C
  const greyColor = [100, 116, 139]; // Muted grey

  // Page width and height
  const pageWidth = doc.internal.pageSize.getWidth();
  
  // 1. Header with Asgard Branding
  doc.setFillColor(...primaryColor);
  doc.rect(0, 0, pageWidth, 40, 'F');
  
  doc.setTextColor(...accentColor);
  doc.setFont('Helvetica', 'bold');
  doc.setFontSize(22);
  doc.text('ASGARD CRM', 20, 25);
  
  doc.setTextColor(255, 255, 255);
  doc.setFontSize(9);
  doc.setFont('Helvetica', 'normal');
  doc.text('Gestion & Facturation Auto-Entreprise', 20, 32);
  
  doc.setTextColor(255, 255, 255);
  doc.setFontSize(20);
  doc.setFont('Helvetica', 'bold');
  doc.text('FACTURE', pageWidth - 20, 27, { align: 'right' });
  
  // 2. Invoice Info Block
  doc.setTextColor(...primaryColor);
  doc.setFontSize(9);
  doc.setFont('Helvetica', 'normal');
  
  let y = 55;
  
  // Issuer Info (Left)
  doc.setFont('Helvetica', 'bold');
  doc.text('ÉMETTEUR :', 20, y);
  doc.setFont('Helvetica', 'normal');
  doc.text(businessSettings.companyName || 'Mon Auto-Entreprise', 20, y + 6);
  doc.text(businessSettings.contactName || 'Votre Nom', 20, y + 11);
  doc.text(businessSettings.address || 'Votre Adresse', 20, y + 16, { maxWidth: 75 });
  doc.text(`Tél : ${businessSettings.phone || '06 00 00 00 00'}`, 20, y + 27);
  doc.text(`Email : ${businessSettings.email || 'contact@monentreprise.fr'}`, 20, y + 32);
  if (businessSettings.siret) {
    doc.text(`SIRET : ${businessSettings.siret}`, 20, y + 37);
  }
  
  // Client Info (Right)
  doc.setFont('Helvetica', 'bold');
  doc.text('FACTURÉ À :', 110, y);
  doc.setFont('Helvetica', 'normal');
  doc.text(client.companyName, 110, y + 6);
  doc.text(client.contactName, 110, y + 11);
  doc.text(client.address, 110, y + 16, { maxWidth: 80 });
  doc.text(`Tél : ${client.phone}`, 110, y + 27);
  doc.text(`Email : ${client.email}`, 110, y + 32);

  // Invoice Metadata (Top Right)
  doc.setFont('Helvetica', 'bold');
  doc.text(`N° Facture :`, pageWidth - 20, y, { align: 'right' });
  doc.setFont('Helvetica', 'normal');
  doc.text(invoice.invoiceNumber, pageWidth - 20, y + 5, { align: 'right' });
  doc.text(`Date : ${new Date(invoice.date).toLocaleDateString('fr-FR')}`, pageWidth - 20, y + 12, { align: 'right' });
  
  y = 110;
  
  // 3. Table Header
  doc.setFillColor(248, 250, 252);
  doc.rect(20, y, pageWidth - 40, 10, 'F');
  doc.setDrawColor(229, 231, 235);
  doc.line(20, y, pageWidth - 20, y);
  doc.line(20, y + 10, pageWidth - 20, y + 10);
  
  doc.setTextColor(...primaryColor);
  doc.setFont('Helvetica', 'bold');
  doc.setFontSize(9);
  doc.text('Description de la prestation', 22, y + 6.5);
  doc.text('Type', 95, y + 6.5);
  doc.text('TVA', 125, y + 6.5, { align: 'right' });
  doc.text('Montant HT', 155, y + 6.5, { align: 'right' });
  doc.text('Total TTC', pageWidth - 22, y + 6.5, { align: 'right' });
  
  // Table Row
  y = 120;
  doc.setFont('Helvetica', 'normal');
  
  // Format Type for display
  let typeLabel = 'Service';
  if (invoice.serviceType === 'service_bic') typeLabel = 'Serv. Comm.';
  else if (invoice.serviceType === 'vente') typeLabel = 'Vente';
  
  doc.text(invoice.description || 'Prestation de service', 22, y + 7, { maxWidth: 65 });
  doc.text(typeLabel, 95, y + 7);
  doc.text(`${invoice.tvaRate}%`, 125, y + 7, { align: 'right' });
  doc.text(`${invoice.amountHt.toFixed(2)} €`, 155, y + 7, { align: 'right' });
  doc.text(`${invoice.amountTotal.toFixed(2)} €`, pageWidth - 22, y + 7, { align: 'right' });
  
  doc.line(20, y + 15, pageWidth - 20, y + 15);
  
  // 4. Totals Block
  y = 145;
  doc.setFont('Helvetica', 'normal');
  doc.text('Total Hors Taxes (HT) :', 135, y, { align: 'right' });
  doc.text(`${invoice.amountHt.toFixed(2)} €`, pageWidth - 22, y, { align: 'right' });
  
  doc.text(`TVA (${invoice.tvaRate}%) :`, 135, y + 6, { align: 'right' });
  doc.text(`${invoice.amountTva.toFixed(2)} €`, pageWidth - 22, y + 6, { align: 'right' });
  
  // Highlight Total Box
  doc.setFillColor(...primaryColor);
  doc.rect(100, y + 12, pageWidth - 120, 10, 'F');
  doc.setTextColor(255, 255, 255);
  doc.setFont('Helvetica', 'bold');
  doc.setFontSize(10);
  doc.text('TOTAL NET À PAYER (TTC) :', 135, y + 18.5, { align: 'right' });
  doc.setTextColor(...accentColor);
  doc.text(`${invoice.amountTotal.toFixed(2)} €`, pageWidth - 22, y + 18.5, { align: 'right' });
  
  // 5. Legal & Payment Info
  y = 190;
  doc.setTextColor(...primaryColor);
  doc.setFontSize(8);
  doc.setFont('Helvetica', 'bold');
  doc.text('MENTIONS LÉGALES & CONDITIONS DE PAIEMENT', 20, y);
  doc.setFont('Helvetica', 'normal');
  
  let legalNotice = '';
  if (invoice.tvaRate === 0) {
    legalNotice += 'TVA non applicable, article 293 B du CGI.\n';
  }
  legalNotice += 'Dispensé d\'immatriculation au registre du commerce et des sociétés (RCS) et au répertoire des métiers (RM).\n';
  legalNotice += `Mode de règlement : Virement bancaire sous 30 jours. IBAN : ${businessSettings.iban || 'FR76 0000 0000 0000 0000 0000 000'}`;
  
  doc.text(legalNotice, 20, y + 5, { maxWidth: pageWidth - 40 });
  
  // Footer Signature Line
  doc.line(20, 260, pageWidth - 20, 260);
  doc.setFontSize(7);
  doc.setTextColor(...greyColor);
  doc.text(`Facture générée automatiquement via Asgard CRM le ${new Date().toLocaleDateString('fr-FR')}`, pageWidth / 2, 267, { align: 'center' });
  
  // Download file
  doc.save(`${invoice.invoiceNumber}.pdf`);
}
