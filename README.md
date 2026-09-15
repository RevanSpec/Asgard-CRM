# Asgard CRM 🛡️

**Asgard CRM** est une application de bureau légère, moderne et performante conçue spécifiquement pour les micro-entrepreneurs et auto-entrepreneurs français. Construite avec **React**, **Vite** et **Tauri**, elle permet de gérer l'intégralité de votre activité commerciale et comptable en local, garantissant une confidentialité totale de vos données.

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

* **Framework de bureau** : [Tauri 2](https://tauri.app/) (hôte Rust, webview du système)
* **Frontend** : [React](https://react.dev/) + [Vite](https://vite.dev/)
* **Base de données** : [Dexie.js](https://dexie.org/) (IndexedDB wrapper)
* **Design** : Interface responsive moderne avec thème sombre *glassmorphism* haut de gamme.
* **Génération PDF** : [jspdf](https://artskydj.github.io/jsPDF/docs/index.html)
* **Envoi d'emails** : [lettre](https://lettre.rs/) (commande Tauri, côté Rust)
* **Secrets** : trousseau du système d'exploitation via [keyring](https://crates.io/crates/keyring)
* **Linter** : [Oxlint](https://oxc.rs/docs/guide/usage/linter.html) (ultra rapide)
* **Tests** : [Vitest](https://vitest.dev/) pour la logique métier, `cargo test` pour l'hôte

> 🦀 Le projet migre progressivement vers Rust — voir [`docs/MIGRATION_RUST.md`](docs/MIGRATION_RUST.md).
> Phases livrées : **0** (logique métier sous tests), **1** (coquille Tauri).

---

## ⚙️ Installation & Démarrage local

### Prérequis

* [Node.js](https://nodejs.org/) 18 ou supérieur
* [Rust](https://rustup.rs/) 1.77 ou supérieur (`rustup default stable`)
* Sous Windows : **WebView2**, présent d'origine depuis Windows 10 21H2, et les *Build Tools* de Visual Studio (composant C++)
* Sous Linux : `libwebkit2gtk-4.1-dev`, `build-essential`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

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
   *Tauri lance le serveur Vite (port 5173) puis compile et ouvre l'application. La première compilation Rust prend plusieurs minutes ; les suivantes sont quasi instantanées.*

   Pour travailler l'interface seule, sans recompiler Rust :
   ```bash
   npm run dev:vite
   ```
   *Les fonctionnalités qui dépendent de l'hôte — envoi d'e-mails, trousseau — sont alors désactivées proprement.*

4. **Construire l'installeur** :
   ```bash
   npm run dist
   ```

### Tests

```bash
npm test                      # logique métier (151 tests)
cd src-tauri && cargo test    # hôte Rust
```

---

## 💡 Astuce de Configuration (Proton Mail Bridge)

Si vous utilisez **Proton Mail Bridge** pour envoyer vos e-mails professionnels de manière sécurisée en local :
1. Allez dans les **Paramètres de l'entreprise** dans l'application.
2. Dans la section **SMTP** :
   - **Hôte** : `127.0.0.1` ou `localhost`
   - **Port** : Le port fourni par votre application Bridge (généralement `1025`)
   - **Utilisateur / Mot de passe** : Les identifiants générés par Proton Mail Bridge
   - **Sécurité** : Choisissez `Aucune (STARTTLS automatique / Proton Mail Bridge)` car le certificat local auto-signé de Proton est automatiquement accepté.

> 🔐 Depuis la phase 1, le mot de passe SMTP est conservé dans le **trousseau de votre système** (Gestionnaire d'identifiants Windows, Trousseau macOS, Secret Service Linux) et non plus dans les réglages de l'application. Il ne figure donc plus dans les fichiers de sauvegarde. Un mot de passe déjà enregistré par une version précédente est déplacé automatiquement au premier lancement.
>
> La tolérance au certificat auto-signé ne vaut que pour `127.0.0.1` et `localhost` — jamais pour un serveur distant.
