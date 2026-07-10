import React from 'react';
import { Icons } from './Icons';

export default function EstimatesTab({
  estimatesSearch,
  setEstimatesSearch,
  estimates,
  handleCreateEstimateClick,
  handleConvertEstimateToInvoice,
  handleOpenSendEmail,
  handleExportEstimatePDF,
  handleEditEstimateClick,
  handleDeleteEstimate
}) {
  return (
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
  );
}
