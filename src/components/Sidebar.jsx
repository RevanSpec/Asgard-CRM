import React from 'react';
import { Icons } from './Icons';

export default function Sidebar({ activeTab, setActiveTab }) {
  return (
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
          className={`nav-item ${activeTab === 'estimates' ? 'active' : ''}`}
          onClick={() => setActiveTab('estimates')}
        >
          <Icons.Estimates />
          <span>Devis</span>
        </li>
        <li 
          className={`nav-item ${activeTab === 'invoices' ? 'active' : ''}`}
          onClick={() => setActiveTab('invoices')}
        >
          <Icons.Invoices />
          <span>Factures</span>
        </li>
        <li 
          className={`nav-item ${activeTab === 'expenses' ? 'active' : ''}`}
          onClick={() => setActiveTab('expenses')}
        >
          <Icons.Expenses />
          <span>Dépenses</span>
        </li>
        <li 
          className={`nav-item ${activeTab === 'compta' ? 'active' : ''}`}
          onClick={() => setActiveTab('compta')}
        >
          <Icons.Accounting />
          <span>Comptabilité</span>
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
  );
}
