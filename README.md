# Asgard CRM 🛡️

**Asgard CRM** est une application de bureau légère, moderne et performante conçue spécifiquement pour les micro-entrepreneurs et auto-entrepreneurs français. Construite avec **React**, **Vite** et **Electron**, elle permet de gérer l'intégralité de votre activité commerciale et comptable en local, garantissant une confidentialité totale de vos données.

---

## 🚀 Fonctionnalités clés

- 📊 **Tableau de bord dynamique** : Suivi en temps réel de votre chiffre d'affaires, statistiques mensuelles/annuelles, graphiques d'évolution et santé financière globale.
- 👥 **Gestion des clients** : Fiches clients complètes (coordonnées, SIRET, historique d'affaires) pour un suivi optimal.
- 📄 **Devis & Factures** :
  - Création et édition intuitive de devis et factures.
  - Génération automatique des numéros de pièces réglementaires (`DEV-CLIENT-ANNEE-XXXX` / `FAC-CLIENT-ANNEE-XXXX`).
  - Conversion d'un devis en facture en 1 clic.
  - Gestion du statut de paiement (Brouillon, Envoyé, Payé, En retard).
- 💸 **Gestion des dépenses** : Suivi des frais professionnels avec catégorisation et méthode de paiement.
- 📐 **Comptabilité & URSSAF** :
  - Registre réglementaire des recettes encaissées.
  - Calcul automatique des cotisations sociales URSSAF selon votre activité (Libérale BNC, Artisanale/Commerciale BIC, Vente de marchandises).
  - Prise en charge des taux réduits de l'**ACRE** (50%).
  - Suivi des seuils de franchise de TVA et des plafonds de chiffre d'affaires micro-entreprise avec alertes visuelles.
- ✉️ **Envoi d'e-mails & SMTP** : Exportez et envoyez directement vos factures et devis en PDF à vos clients depuis l'application (support de Proton Mail Bridge et des serveurs SMTP standards).
- 💾 **Confidentialité & Sauvegarde** :
  - Base de données locale intégrée avec **Dexie.js** (IndexedDB). Aucune donnée ne quitte votre machine.
  - Export et import de sauvegardes complètes pour ne jamais perdre vos données.

---

## 🛠️ Stack Technique

* **Framework de bureau** : [Electron](https://www.electronjs.org/)
* **Frontend** : [React](https://react.dev/) + [Vite](https://vite.dev/)
* **Base de données** : [Dexie.js](https://dexie.org/) (IndexedDB wrapper)
* **Design** : Interface responsive moderne avec thème sombre *glassmorphism* haut de gamme.
* **Génération PDF** : [html2canvas](https://html2canvas.hertzen.com/) & [jspdf](https://artskydj.github.io/jsPDF/docs/index.html)
* **Envoi d'emails** : [Nodemailer](https://nodemailer.com/) (géré via IPC Electron)
* **Linter** : [Oxlint](https://oxc.rs/docs/guide/usage/linter.html) (ultra rapide)

---

## ⚙️ Installation & Démarrage local

### Prérequis
Assurez-vous d'avoir [Node.js](https://nodejs.org/) (version 18 ou supérieure recommandée) installé sur votre machine.

### Étapes d'installation

1. **Cloner le projet** :
   ```bash
   git clone https://github.com/RevanSpec/Asgard-CRM.git
   cd Asgard-CRM
   ```

2. **Installer les dépendances** :
   ```bash
   npm install
   ```
   *(ou `npm ci` pour un build propre conforme au `package-lock.json`)*

3. **Démarrer en mode développement** :
   ```bash
   npm run dev
   ```
   *Cette commande lance simultanément le serveur de développement Vite (sur le port 5173) et l'application Electron.*

---

## 💡 Astuce de Configuration (Proton Mail Bridge)

Si vous utilisez **Proton Mail Bridge** pour envoyer vos e-mails professionnels de manière sécurisée en local :
1. Allez dans les **Paramètres de l'entreprise** dans l'application.
2. Dans la section **SMTP** :
   - **Hôte** : `127.0.0.1` ou `localhost`
   - **Port** : Le port fourni par votre application Bridge (généralement `1025`)
   - **Utilisateur / Mot de passe** : Les identifiants générés par Proton Mail Bridge
   - **Sécurité** : Choisissez `Aucune (STARTTLS automatique / Proton Mail Bridge)` car le certificat local auto-signé de Proton est automatiquement accepté.
