import React from 'react';

export default function SettingsTab({
  businessSettings,
  saveSettings,
  handleTestSMTP,
  smtpTesting,
  handleExportBackup,
  handleImportBackup,
  showAlert
}) {
  return (
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

          {/* Section : Identité Visuelle & PDF */}
          <div className="settings-section" style={{ borderTop: '1px solid var(--border-glass)', paddingTop: '1.5rem', marginTop: '1.5rem' }}>
            <div className="settings-section-title">Identité Visuelle & PDF</div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem' }}>
              <div className="form-group">
                <label className="form-label">Couleur d'accentuation des PDF</label>
                <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                  <input 
                    type="color" 
                    className="form-input"
                    style={{ width: '50px', height: '40px', padding: '2px', cursor: 'pointer', border: '1px solid var(--border-glass)' }}
                    value={businessSettings.customColor || '#E5A93C'}
                    onChange={(e) => saveSettings({ ...businessSettings, customColor: e.target.value })}
                  />
                  <input 
                    type="text" 
                    className="form-input"
                    style={{ fontFamily: 'monospace' }}
                    value={businessSettings.customColor || '#E5A93C'}
                    onChange={(e) => saveSettings({ ...businessSettings, customColor: e.target.value })}
                  />
                  <button 
                    type="button" 
                    className="btn btn-secondary" 
                    onClick={() => saveSettings({ ...businessSettings, customColor: '#E5A93C' })}
                    style={{ padding: '0.5rem 1rem', fontSize: '0.8rem' }}
                  >
                    Réinitialiser
                  </button>
                </div>
                <span className="metric-subtext">Couleur utilisée pour les titres et totaux sur les devis/factures exportés.</span>
              </div>

              <div className="form-group">
                <label className="form-label">Logo de l'entreprise (Format image)</label>
                <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
                  {businessSettings.logoBase64 ? (
                    <div style={{ position: 'relative', border: '1px solid var(--border-glass)', borderRadius: '4px', padding: '4px', background: 'rgba(255,255,255,0.05)', display: 'flex', alignItems: 'center', justifyContent: 'center', height: '40px', width: '60px' }}>
                      <img src={businessSettings.logoBase64} alt="Logo" style={{ maxHeight: '100%', maxWidth: '100%', objectFit: 'contain' }} />
                      <button 
                        type="button" 
                        style={{ position: 'absolute', top: '-5px', right: '-5px', background: '#EF4444', color: 'white', border: 'none', borderRadius: '50%', width: '16px', height: '16px', cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '10px', fontWeight: 'bold' }}
                        onClick={() => saveSettings({ ...businessSettings, logoBase64: '' })}
                        title="Supprimer le logo"
                      >
                        ×
                      </button>
                    </div>
                  ) : (
                    <div style={{ border: '1px dashed var(--border-glass)', borderRadius: '4px', height: '40px', width: '60px', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '12px', color: 'var(--text-muted)' }}>
                      Aucun
                    </div>
                  )}
                  <input 
                    type="file" 
                    accept="image/*"
                    style={{ display: 'none' }}
                    id="logoUploadInput"
                    onChange={(e) => {
                      const file = e.target.files[0];
                      if (file) {
                        const reader = new FileReader();
                        reader.onload = (event) => {
                          saveSettings({ ...businessSettings, logoBase64: event.target.result });
                        };
                        reader.readAsDataURL(file);
                      }
                    }}
                  />
                  <label htmlFor="logoUploadInput" className="btn btn-secondary" style={{ padding: '0.5rem 1rem', fontSize: '0.8rem', cursor: 'pointer', margin: 0 }}>
                    Choisir un logo
                  </label>
                </div>
                <span className="metric-subtext">Recommandé : PNG transparent, format paysage (hauteur max 60px).</span>
              </div>
            </div>
          </div>

          {/* Section : SMTP config */}
          <div className="settings-section" style={{ borderTop: '1px solid var(--border-glass)', paddingTop: '1.5rem', marginTop: '1.5rem' }}>
            <div className="flex-between" style={{ marginBottom: '1.25rem' }}>
              <div className="settings-section-title" style={{ marginBottom: '0' }}>Configuration de messagerie (SMTP)</div>
              <button 
                type="button" 
                className="btn btn-secondary" 
                onClick={handleTestSMTP}
                disabled={smtpTesting}
              >
                {smtpTesting ? "Vérification en cours..." : "Tester la connexion SMTP"}
              </button>
            </div>
            
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.25rem', marginBottom: '1rem' }}>
              <div className="form-group">
                <label className="form-label">Hôte SMTP</label>
                <input 
                  type="text" 
                  className="form-input" 
                  placeholder="ex: 127.0.0.1 (Proton Mail Bridge)"
                  value={businessSettings.smtpHost}
                  onChange={(e) => saveSettings({ ...businessSettings, smtpHost: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Port SMTP</label>
                <input 
                  type="text" 
                  className="form-input" 
                  placeholder="ex: 1025"
                  value={businessSettings.smtpPort}
                  onChange={(e) => saveSettings({ ...businessSettings, smtpPort: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Utilisateur SMTP / Adresse Mail</label>
                <input 
                  type="text" 
                  className="form-input" 
                  placeholder="Votre adresse e-mail"
                  value={businessSettings.smtpUser}
                  onChange={(e) => saveSettings({ ...businessSettings, smtpUser: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Mot de passe SMTP</label>
                <input 
                  type="password" 
                  className="form-input" 
                  placeholder="Mot de passe ou clé générée"
                  value={businessSettings.smtpPass}
                  onChange={(e) => saveSettings({ ...businessSettings, smtpPass: e.target.value })}
                />
              </div>
              <div className="form-group">
                <label className="form-label">Sécurité Connexion</label>
                <select 
                  className="form-input"
                  value={businessSettings.smtpSecure}
                  onChange={(e) => saveSettings({ ...businessSettings, smtpSecure: e.target.value })}
                >
                  <option value="none">Aucune (STARTTLS automatique / Proton Mail Bridge)</option>
                  <option value="ssl">SSL Strict (Port 465)</option>
                </select>
              </div>
            </div>
            
            <span className="metric-subtext" style={{ display: 'block', color: 'var(--color-gold)', lineHeight: '1.5' }}>
              💡 Pour Proton Mail Bridge, laissez l'Hôte sur <strong>127.0.0.1</strong>, le port sur celui indiqué par votre application Bridge (souvent 1025), et choisissez "Aucune" pour la sécurité.
            </span>
          </div>

          {/* Section : Sauvegarde et Restauration */}
          <div className="settings-section" style={{ borderTop: '1px solid var(--border-glass)', paddingTop: '1.5rem', marginTop: '1.5rem' }}>
            <div className="settings-section-title">Sécurité & Sauvegarde des données</div>
            <p className="metric-subtext" style={{ marginBottom: '1.25rem', lineHeight: '1.5' }}>
              Vos données sont stockées localement dans votre base de données locale. Exportez régulièrement des sauvegardes pour éviter toute perte de données en cas de panne de votre ordinateur.
            </p>
            
            <div style={{ display: 'flex', gap: '1rem' }}>
              <button 
                type="button" 
                className="btn btn-primary"
                onClick={handleExportBackup}
              >
                Exporter les données (.json)
              </button>
              
              <input 
                type="file" 
                accept=".json"
                id="dbBackupImportInput"
                style={{ display: 'none' }}
                onChange={handleImportBackup}
              />
              <button 
                type="button" 
                className="btn btn-secondary"
                onClick={() => document.getElementById('dbBackupImportInput').click()}
              >
                Restaurer une sauvegarde
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
