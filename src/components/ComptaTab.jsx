import React from 'react';
import { loadRecettesCsv } from '../db';

/**
 * Tri d'affichage du registre.
 *
 * Purement visuel : le livre des recettes réglementaire est produit par l'hôte
 * (`loadRecettesCsv`), dans l'ordre chronologique que la loi impose. Ici on
 * montre le plus récent en premier, ce qui est plus commode à l'écran.
 */
function collectedInvoices(invoices, newestFirst = true) {
  const sign = newestFirst ? -1 : 1;

  return invoices
    .filter((invoice) => invoice.status === 'payee')
    .slice()
    .sort((a, b) => sign * (
      new Date(a.paymentDate || a.date) - new Date(b.paymentDate || b.date)
    ));
}

export default function ComptaTab({
  comptaActiveTab,
  setComptaActiveTab,
  invoices,
  periodType,
  setPeriodType,
  selectedYear,
  setSelectedYear,
  selectedMonth,
  setSelectedMonth,
  selectedQuarter,
  setSelectedQuarter,
  showAlert,
  declaration,
  gauges,
}) {
  return (
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
              onClick={async () => {
                if (collectedInvoices(invoices).length === 0) {
                  showAlert("Erreur", "Aucune recette encaissée à exporter.");
                  return;
                }

                // Le CSV réglementaire est produit par l'hôte : séparateur,
                // décimale, BOM et ordre chronologique y sont testés.
                const csv = await loadRecettesCsv();
                const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
                const url = URL.createObjectURL(blob);
                const link = document.createElement("a");
                link.setAttribute("href", url);
                link.setAttribute("download", `Livre_des_recettes_${new Date().getFullYear()}.csv`);
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
                URL.revokeObjectURL(url);
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
                {collectedInvoices(invoices)
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
                {collectedInvoices(invoices).length === 0 && (
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
            const { bnc, bic, vente, totalCa: totalPeriodCa, totalCharges: totalPeriodCharges } =
              declaration ?? {
                bnc: { ht: 0, rate: 0, charges: 0 },
                bic: { ht: 0, rate: 0, charges: 0 },
                vente: { ht: 0, rate: 0, charges: 0 },
                totalCa: 0,
                totalCharges: 0,
              };

            const { ht: bncHt, rate: bncRate, charges: bncCharges } = bnc;
            const { ht: bicHt, rate: bicRate, charges: bicCharges } = bic;
            const { ht: venteHt, rate: venteRate, charges: venteCharges } = vente;

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
            const currentYear = gauges?.year ?? new Date().getFullYear();
            if (!gauges) return null;

            const serviceCa = gauges.service.ca;
            const venteCa = gauges.vente.ca;
            const { tvaLimit: limitTvaService, tvaTolerance: toleranceTvaService, microLimit: limitMicroService } = gauges.service;
            const { tvaLimit: limitTvaVente, tvaTolerance: toleranceTvaVente, microLimit: limitMicroVente } = gauges.vente;
            const { pctTva: pctTvaService, pctMicro: pctMicroService } = gauges.service;
            const { pctTva: pctTvaVente, pctMicro: pctMicroVente } = gauges.vente;

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
  );
}
