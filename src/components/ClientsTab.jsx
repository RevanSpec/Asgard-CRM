import React from 'react';
import { Icons } from './Icons';

export default function ClientsTab({
  clientSearch,
  setClientSearch,
  filteredClients,
  handleOpenAddClient,
  handleOpenEditClient,
  handleDeleteClient
}) {
  return (
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
  );
}
