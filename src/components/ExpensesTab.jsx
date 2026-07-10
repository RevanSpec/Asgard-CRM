import React from 'react';
import { Icons } from './Icons';

export default function ExpensesTab({
  expensesSearch,
  setExpensesSearch,
  expenses,
  handleCreateExpenseClick,
  handleEditExpenseClick,
  handleDeleteExpense
}) {
  return (
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
  );
}
