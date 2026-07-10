import React, { useState } from 'react';
import { Icons } from './Icons';

export default function DashboardTab({
  businessSettings,
  caAlerts,
  totalCaHt,
  htFacture,
  totalCaTtc,
  totalExpenses,
  totalUrssaf,
  netProfit,
  monthlyFinancials,
  expensesCategoryData,
  breakdown,
  top10Invoices,
  top5Clients,
  handleOpenAddInvoice,
  handleOpenPaymentModal,
  handleOpenSendEmail,
  handleExportPDF
}) {
  const [rightTab, setRightTab] = useState('activity'); // 'activity' or 'expenses'

  // Scale Y coordinates helper
  const getY = (val) => {
    const range = monthlyFinancials.maxVal - monthlyFinancials.minVal;
    if (range === 0) return 230; // Fallback
    const ratio = (val - monthlyFinancials.minVal) / range;
    return 230 - (ratio * 200);
  };

  return (
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
        {/* SVG CA & Profit Evolution Chart */}
        <div className="card-glass">
          <h3 style={{ fontFamily: 'var(--font-title)', marginBottom: '0.75rem' }}>Suivi CA vs Bénéfice Net ({new Date().getFullYear()})</h3>
          
          <div style={{ display: 'flex', gap: '1.5rem', justifyContent: 'center', marginBottom: '1rem', fontSize: '0.8rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
              <span style={{ display: 'inline-block', width: '12px', height: '12px', backgroundColor: 'var(--color-gold)', borderRadius: '2px' }}></span>
              <span>Chiffre d'Affaires HT</span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
              <span style={{ display: 'inline-block', width: '12px', height: '12px', backgroundColor: '#10B981', borderRadius: '2px' }}></span>
              <span>Bénéfice Net Réel</span>
            </div>
          </div>

          <div style={{ position: 'relative', width: '100%', height: '320px' }}>
            <svg width="100%" height="100%" viewBox="0 0 600 280" preserveAspectRatio="none">
              <defs>
                <linearGradient id="ca-gradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="var(--color-gold)" stopOpacity="0.25" />
                  <stop offset="100%" stopColor="var(--color-gold)" stopOpacity="0.0" />
                </linearGradient>
                <linearGradient id="profit-gradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#10B981" stopOpacity="0.25" />
                  <stop offset="100%" stopColor="#10B981" stopOpacity="0.0" />
                </linearGradient>
              </defs>

              {/* Horizontal Grid Lines */}
              {[0, 0.25, 0.5, 0.75, 1].map((ratio, idx) => {
                const val = monthlyFinancials.minVal + ratio * (monthlyFinancials.maxVal - monthlyFinancials.minVal);
                const y = getY(val);
                return (
                  <g key={idx}>
                    <line
                      x1="45"
                      y1={y}
                      x2="570"
                      y2={y}
                      className="chart-grid-line"
                    />
                    <text
                      x="40"
                      y={y + 3}
                      textAnchor="end"
                      fill="var(--text-muted)"
                      fontSize="8"
                      fontFamily="monospace"
                    >
                      {Math.round(val)}
                    </text>
                  </g>
                );
              })}

              {/* Chart Lines and Areas */}
              {(() => {
                const caPoints = monthlyFinancials.caValues.map((val, idx) => {
                  const x = 60 + idx * 45;
                  const y = getY(val);
                  return { x, y };
                });

                const profitPoints = monthlyFinancials.profitValues.map((val, idx) => {
                  const x = 60 + idx * 45;
                  const y = getY(val);
                  return { x, y };
                });

                const caPath = caPoints.reduce((acc, p, idx) => 
                  idx === 0 ? `M ${p.x} ${p.y}` : `${acc} L ${p.x} ${p.y}`, ''
                );

                const profitPath = profitPoints.reduce((acc, p, idx) => 
                  idx === 0 ? `M ${p.x} ${p.y}` : `${acc} L ${p.x} ${p.y}`, ''
                );

                const caArea = caPoints.length > 0 
                  ? `${caPath} L ${caPoints[caPoints.length-1].x} ${getY(0)} L ${caPoints[0].x} ${getY(0)} Z` 
                  : '';

                const profitArea = profitPoints.length > 0 
                  ? `${profitPath} L ${profitPoints[profitPoints.length-1].x} ${getY(0)} L ${profitPoints[0].x} ${getY(0)} Z` 
                  : '';

                return (
                  <>
                    {/* Areas */}
                    <path d={caArea} fill="url(#ca-gradient)" />
                    <path d={profitArea} fill="url(#profit-gradient)" />

                    {/* Zero Line */}
                    {monthlyFinancials.minVal < 0 && (
                      <line
                        x1="45"
                        y1={getY(0)}
                        x2="570"
                        y2={getY(0)}
                        stroke="#EF4444"
                        strokeDasharray="4 4"
                        strokeWidth="1.5"
                        opacity="0.6"
                      />
                    )}

                    {/* Lines */}
                    <path d={caPath} fill="none" stroke="var(--color-gold)" strokeWidth="2" />
                    <path d={profitPath} fill="none" stroke="#10B981" strokeWidth="2" />
                    
                    {/* CA Dots */}
                    {caPoints.map((p, idx) => (
                      <g key={`ca-${idx}`}>
                        <circle
                          cx={p.x}
                          cy={p.y}
                          r="3.5"
                          fill="var(--bg-primary)"
                          stroke="var(--color-gold)"
                          strokeWidth="1.5"
                        />
                        {monthlyFinancials.caValues[idx] > 0 && (
                          <text
                            x={p.x}
                            y={p.y - 8}
                            textAnchor="middle"
                            fill="var(--text-primary)"
                            fontSize="8"
                            fontWeight="600"
                          >
                            {Math.round(monthlyFinancials.caValues[idx])}
                          </text>
                        )}
                      </g>
                    ))}

                    {/* Profit Dots */}
                    {profitPoints.map((p, idx) => (
                      <g key={`profit-${idx}`}>
                        <circle
                          cx={p.x}
                          cy={p.y}
                          r="3.5"
                          fill="var(--bg-primary)"
                          stroke="#10B981"
                          strokeWidth="1.5"
                        />
                        {Math.abs(monthlyFinancials.profitValues[idx]) > 0 && (
                          <text
                            x={p.x}
                            y={monthlyFinancials.profitValues[idx] >= 0 ? p.y - 8 : p.y + 12}
                            textAnchor="middle"
                            fill="#10B981"
                            fontSize="8"
                            fontWeight="600"
                          >
                            {Math.round(monthlyFinancials.profitValues[idx])}
                          </text>
                        )}
                      </g>
                    ))}
                  </>
                );
              })()}

              {/* X-Axis Labels */}
              {monthlyFinancials.labels.map((label, idx) => (
                <text
                  key={idx}
                  x={60 + idx * 45}
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

        {/* Breakdown Panel (Tabs between Activity Types and Expenses Category Breakdown) */}
        <div className="card-glass flex-between" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
          <div style={{ display: 'flex', borderBottom: '1px solid var(--border-glass)', marginBottom: '1.25rem', paddingBottom: '0.5rem', gap: '0.5rem' }}>
            <button 
              type="button"
              style={{ 
                background: 'none', 
                border: 'none', 
                color: rightTab === 'activity' ? 'var(--color-gold)' : 'var(--text-muted)', 
                fontWeight: 600, 
                fontSize: '0.9rem',
                cursor: 'pointer',
                padding: '0.25rem 0.5rem',
                borderBottom: rightTab === 'activity' ? '2px solid var(--color-gold)' : 'none',
                outline: 'none'
              }}
              onClick={() => setRightTab('activity')}
            >
              Activités
            </button>
            <button 
              type="button"
              style={{ 
                background: 'none', 
                border: 'none', 
                color: rightTab === 'expenses' ? 'var(--color-gold)' : 'var(--text-muted)', 
                fontWeight: 600, 
                fontSize: '0.9rem',
                cursor: 'pointer',
                padding: '0.25rem 0.5rem',
                borderBottom: rightTab === 'expenses' ? '2px solid var(--color-gold)' : 'none',
                outline: 'none'
              }}
              onClick={() => setRightTab('expenses')}
            >
              Dépenses par catégorie
            </button>
          </div>

          {rightTab === 'activity' ? (
            <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: '1.25rem', flexGrow: 1 }}>
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
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.85rem', flexGrow: 1, overflowY: 'auto', maxHeight: '200px', paddingRight: '4px' }}>
              {expensesCategoryData.list.length === 0 ? (
                <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.85rem', padding: '2rem 0' }}>
                  Aucune dépense enregistrée pour le moment.
                </div>
              ) : (
                expensesCategoryData.list.map(item => (
                  <div key={item.key}>
                    <div className="flex-between" style={{ fontSize: '0.85rem', marginBottom: '0.3rem' }}>
                      <span style={{ fontWeight: 500 }}>{item.label}</span>
                      <span style={{ color: 'var(--color-gold)', fontWeight: 600 }}>{item.amount.toFixed(2)} € ({item.pct.toFixed(1)}%)</span>
                    </div>
                    <div style={{ height: '6px', backgroundColor: 'var(--bg-tertiary)', borderRadius: '3px', overflow: 'hidden' }}>
                      <div style={{ width: `${item.pct}%`, height: '100%', backgroundColor: 'var(--color-gold)', borderRadius: '3px' }}></div>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}
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
  );
}
