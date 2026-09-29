# Asgard CRM 🛡️

**Asgard CRM** est une application de bureau légère, moderne et performante conçue spécifiquement pour les micro-entrepreneurs et auto-entrepreneurs français. Écrite en **Rust** de bout en bout — hôte **Tauri**, interface **Leptos** compilée en WebAssembly — elle permet de gérer l'intégralité de votre activité commerciale et comptable en local, garantissant une confidentialité totale de vos données.

---

## 🚀 Fonctionnalités clés

- 📊 **Tableau de bord dynamique** : Suivi en temps réel de votre chiffre d'affaires, statistiques mensuelles/annuelles, graphiques d'évolution et santé financière globale.
- 🚀 **Premier lancement guidé** : L'application demande les informations qui figurent sur vos factures — raison sociale, adresse, SIRET, IBAN — et refuse d'éditer une pièce tant qu'elles manquent, plutôt que d'imprimer une identité qui n'est pas la vôtre. Le jeu de données d'exemple est proposé, jamais imposé.
- 👥 **Gestion des clients** : Fiches clients (coordonnées, historique d'affaires) pour un suivi optimal.
- 📄 **Devis & Factures** :
  - Création et édition intuitive de devis et factures.
  - Génération automatique des numéros de pièces réglementaires (`DEV-CLIENT-ANNEE-XXXX` / `FAC-CLIENT-ANNEE-XXXX`).
  - Conversion d'un devis en facture en 1 clic.
  - Gestion du statut de paiement (Brouillon, Envoyé, Payé, En retard).
  - **Échéance de règlement** calculée au délai que vous fixez, et mentions obligatoires imprimées (pénalités de retard, indemnité de recouvrement de 40 €, escompte).
  - **Avoirs** : une facture émise ne se modifie pas — elle s'annule ou se corrige par un avoir, partiel ou total, avec sa propre série de numéros (`AVO-CLIENT-ANNEE-XXXX`).
- 💸 **Gestion des dépenses** : Suivi des frais professionnels avec catégorisation et méthode de paiement.
- 📐 **Comptabilité & URSSAF** :
  - Registre réglementaire des recettes encaissées.
  - Calcul automatique des cotisations sociales URSSAF selon votre activité (Libérale BNC, Artisanale/Commerciale BIC, Vente de marchandises).
  - Prise en charge des taux réduits de l'**ACRE** (50%).
  - Suivi des seuils de franchise de TVA et des plafonds de chiffre d'affaires micro-entreprise avec alertes visuelles.
- ✉️ **Envoi d'e-mails & SMTP** : Exportez et envoyez directement vos factures et devis en PDF à vos clients depuis l'application (support de Proton Mail Bridge et des serveurs SMTP standards).
- 💾 **Confidentialité & Sauvegarde** :
  - Base de données locale **SQLite**, montants stockés en centimes. Aucune donnée ne quitte votre machine.
  - Export et import de sauvegardes complètes pour ne jamais perdre vos données.
  - **Copie automatique** de la base au premier lancement de chaque journée, les sept dernières conservées à côté de la base.
  - Vos réglages — identité, logo, gabarits d'e-mail, configuration SMTP — vivent **dans la base**, et sont donc emportés par ces copies comme par vos exportations.

---

## 🛠️ Stack Technique

* **Framework de bureau** : [Tauri 2](https://tauri.app/) (hôte Rust, webview du système)
* **Interface** : [Leptos](https://leptos.dev/) compilé en WebAssembly avec [trunk](https://trunkrs.dev/)
* **Noyau métier** : crate `asgard-core` (cotisations URSSAF, seuils, agrégats), partagé par l'hôte et l'interface
* **Base de données** : SQLite via [sqlx](https://github.com/launchbadge/sqlx)
* **Design** : Interface responsive moderne avec thème sombre *glassmorphism* haut de gamme.
* **Génération PDF** : [printpdf](https://github.com/fschutt/printpdf) (côté Rust)
* **Envoi d'emails** : [lettre](https://lettre.rs/) (commande Tauri, côté Rust)
* **Secrets** : trousseau du système d'exploitation via [keyring](https://crates.io/crates/keyring)
* **Tests** : `cargo test`, pour l'ensemble de l'espace de travail

> 🦀 La migration vers Rust est terminée — voir [`docs/MIGRATION_RUST.md`](docs/MIGRATION_RUST.md).
> Phases livrées : **0** (logique métier sous tests), **1** (coquille Tauri), **2** (SQLite), **3** (noyau métier Rust), **4** (PDF et e-mail Rust), **5** (interface Leptos).

---

## ⚙️ Installation & Démarrage local

### Prérequis

* [Rust](https://rustup.rs/) 1.77 ou supérieur (`rustup default stable`), avec la cible WebAssembly :
  `rustup target add wasm32-unknown-unknown`
* [trunk](https://trunkrs.dev/), qui compile l'interface : `cargo install trunk --locked`
* [Node.js](https://nodejs.org/) 18 ou supérieur — uniquement pour la CLI Tauri
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
   *Tauri lance `trunk serve` (port 5174), qui compile l'interface en WebAssembly et la recompile à chaque modification, puis compile et ouvre l'application. La première compilation prend plusieurs minutes ; les suivantes sont bien plus rapides.*

   L'interface ne fonctionne qu'à l'intérieur de l'application : toutes ses données passent par l'hôte. Ouverte seule dans un navigateur, elle ne peut rien charger.

4. **Construire l'installeur** :
   ```bash
   npm run dist
   ```
   *À utiliser systématiquement : `cargo build --release` ne compile pas l'interface et produit un binaire qui cherche le serveur de développement. La compilation est refusée dans ce cas, avec un message explicite.*

### Tests

```bash
cargo test --workspace        # noyau, PDF, hôte et interface
cargo clippy --workspace --all-targets -- -D warnings
```

Ces commandes tournent sur chaque proposition de modification
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)).

**Parcours de l'application.** Les tests unitaires ne voient pas une interface
qui se fige. Un second script lance l'application, s'attache à sa WebView et
joue un parcours complet — accueil du premier lancement, sept écrans, création
d'un client, facturation avec virgule décimale, encaissement, avoir, envoi par
e-mail, conversion d'un devis, suppressions, et le refus d'éditer une facture
sans SIRET — en échouant à la moindre erreur de console :

```bash
node scripts/smoke.mjs target/release/asgard-crm.exe
```

> ⚠️ Il écrit dans votre base. Sauvegardez `%APPDATA%/com.asgard.crm` avant de
> le lancer en local ; en intégration continue, la base part vierge.

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
