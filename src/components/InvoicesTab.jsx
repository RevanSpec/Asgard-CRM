import React from 'react';
import { Icons } from './Icons';

export default function InvoicesTab({
  selectedInvoiceIds,
  setSelectedInvoiceIds,
  invoiceSearch,
  setInvoiceSearch,
  filteredInvoices,
  handleDeleteSelectedInvoices,
  handleOpenAddInvoice,
  handleOpenPaymentModal,
  handleOpenSendEmail,
  handleExportPDF,
  handleDeleteInvoice
}) {
  return (
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
  );
}
