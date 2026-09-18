# Migration Asgard CRM vers Rust

> Analyse du 15/09/2026 — branche `claude/rust-migration-analysis-a1527e`
> Cible retenue : **Tauri 2 + noyau métier Rust**, frontend React conservé jusqu'en phase 5.

---

## 1. État des lieux

### Stack actuelle

| Couche | Technologie | Version |
|---|---|---|
| Shell bureau | Electron | 43.1 |
| UI | React + Vite | 19.2 / 8.1 |
| Données | Dexie (IndexedDB) | 4.4 |
| PDF | jsPDF | 4.2 |
| E-mail | Nodemailer (via IPC) | 9.0 |
| Lint | Oxlint | 1.71 |
| Tests | *aucun* | — |
| Typage | *aucun* | — |

### Inventaire du code

| Fichier | LOC | Nature | Migre vers |
|---|---|---|---|
| `src/App.jsx` | 2 060 | God component : 39 hooks, état global, logique métier + 12 modales JSX | Éclaté : logique vers `asgard-core`, JSX conservé |
| `src/components/*.jsx` | 1 636 | Vues (dashboard, clients, factures, devis, dépenses, compta, réglages) | Conservé (phases 1-4) |
| `src/index.css` + `App.css` | 807 | Thème glassmorphism sombre | Conservé tel quel |
| `src/pdfGenerator.js` | 253 | Génération facture/devis jsPDF | `asgard-pdf` |
| `src/db.js` | 137 | Schéma Dexie, numérotation, export/import | `asgard-db` + `asgard-core` |
| `main.js` | 130 | Fenêtre Electron + 2 handlers IPC SMTP | `src-tauri` + `asgard-mail` |
| **Total** | **~5 223** | | |

**Ce qui migre bien** (~1 200 LOC de logique pure, testable) : calculs URSSAF, seuils TVA et plafonds micro-entreprise, agrégats du tableau de bord, livre des recettes, export CSV, numérotation des pièces, génération PDF, envoi SMTP.

**Ce qui migre mal** (~3 700 LOC) : le JSX et le CSS. Les réécrire en Leptos/Dioxus est du travail sans valeur utilisateur à court terme — d'où le report en phase 5 optionnelle.

---

## 2. Défauts à corriger pendant la migration

Classés par gravité. La migration est l'occasion de les traiter : plusieurs disparaissent « gratuitement » du fait du changement de plateforme.

### Critique

**D1 — Isolation Electron désactivée.** `main.js:12-13` : `nodeIntegration: true` + `contextIsolation: false`, sans script de preload. Le renderer a un accès direct à `require('fs')` et `require('child_process')`. Conjugué au rendu de contenus tiers (`logoBase64`, sauvegardes JSON importées), c'est une surface d'exécution de code arbitraire.
→ *Résolu par construction en Tauri* : pas de Node dans le webview, IPC typée et explicitement déclarée.

**D2 — Mot de passe SMTP en clair.** `smtpPass` est stocké dans `localStorage` (`App.jsx:335`) et réexporté en clair dans le fichier de sauvegarde JSON.
→ Trousseau de l'OS via le crate `keyring` ; exclusion du secret de l'export.

**D3 — Numérotation des factures non conforme.** `db.js:47-53` calcule le numéro par `count(factures de l'année) + 1`. Supprimer une facture fait réutiliser un numéro déjà émis, et deux créations concurrentes produisent le même. L'article 242 nonies A du CGI impose une séquence chronologique **continue et sans rupture**.
→ Table `document_sequences(kind, year, next_value)` incrémentée dans la même transaction SQL que l'insertion.

**D4 — Suppression physique des pièces comptables.** `handleDeleteInvoice` appelle `db.invoices.delete()`. Une facture émise ne peut pas être supprimée (conservation 10 ans, art. L123-22 C. com.) — seulement annulée par un avoir.
→ Colonne `deleted_at` (suppression logique) + pièce d'annulation ; suppression réelle réservée aux brouillons.

### Majeur

**D5 — Montants en `f64`.** Tous les montants (`amountHt`, `amountTva`, `amountTotal`) et toutes les cotisations sont des flottants. Les erreurs d'arrondi se cumulent sur le registre des recettes, qui est un document réglementaire.
→ `rust_decimal` en mémoire, `INTEGER` de centimes en base, règle d'arrondi explicite (demi-supérieur au centime).

**D6 — IndexedDB comme base comptable.** Pas de fichier inspectable ni sauvegardable par l'utilisateur, effaçable par un nettoyage du profil du moteur de rendu, sauvegarde uniquement par export JSON manuel.
→ Fichier SQLite dans le répertoire de données applicatif, sauvegarde atomique par `VACUUM INTO`.

**D7 — Bug : envoi de devis par e-mail impossible.** `App.jsx:703` appelle `generateEstimatePDF`, absent de la liste d'imports en `App.jsx:3`. L'envoi d'un devis lève un `ReferenceError`.
→ À corriger dès la phase 0 (sinon la migration reproduirait le bug).

**D8 — Logique métier triplée dans les vues.** Le calcul des cotisations URSSAF existe en trois exemplaires : `calculateUrssafCharges` (`App.jsx:1063`), `getMonthlyFinancialsData` (`App.jsx:1136`) et une IIFE dans `ComptaTab.jsx:186`. Les seuils TVA (36 800 / 39 100 / 91 900 / 101 000 €) et plafonds (77 700 / 188 700 €) sont des littéraux dans le JSX.
→ Un seul module `asgard-core::urssaf` + barèmes datés par année fiscale.

### Mineur

**D9 — Dépendance déclarée en double.** `html2canvas` (1.4.1) figure dans les dépendances directes alors qu'elle n'est importée nulle part dans `src/` ni dans `main.js`.

*Précision apportée par la phase 0* : la retirer du `package.json` ne la retire pas du bundle. `jspdf` la déclare comme sa propre dépendance et Vite continue de l'empaqueter (199 Ko). La déclaration directe était donc redondante, pas la bibliothèque : celle-ci ne disparaîtra qu'avec `jspdf`, en phase 4.

**D10 — Poids.** Installeur Electron ≈ 200-250 Mo, ~200 Mo de RSS au repos, ~2 s de démarrage à froid.

---

## 3. Décision d'architecture

| Option | Effort | Gain | Verdict |
|---|---|---|---|
| **A.** Tauri 2, frontend React conservé | 5-7 j | Résout D1, D2, D10 ; installeur divisé par 20 | **Retenu** — étape obligatoire de toute façon |
| **B.** A + noyau métier Rust (SQLite, URSSAF, PDF, SMTP) | +15-20 j | Résout D3-D9 ; logique testée | **Retenu** — phases 2 à 4 |
| **C.** B + frontend Rust (Leptos / Dioxus) | +4-6 sem. | Typage de bout en bout, un seul langage | Optionnel — phase 5, ROI faible |
| **D.** Natif pur (egui / iced), sans webview | +8-10 sem. | Perf maximale | **Écarté** — le thème glassmorphism et les graphes SVG seraient à refaire intégralement |

La migration se fait **par soustraction progressive de JavaScript**, jamais par réécriture simultanée. À chaque fin de phase l'application est livrable.

### Arborescence cible

```
asgard-crm/
├── crates/
│   ├── asgard-core/          # logique métier pure, sans I/O — 100 % testable
│   │   ├── money.rs          #   Decimal, arrondis, calcul TVA
│   │   ├── numbering.rs      #   séquences FAC-/DEV- conformes
│   │   ├── urssaf.rs         #   taux BNC/BIC/vente, ACRE, périodes
│   │   ├── thresholds.rs     #   seuils TVA + plafonds micro, barèmes par année
│   │   ├── reporting.rs      #   agrégats dashboard, livre des recettes, CSV
│   │   └── model.rs          #   Client, Invoice, Estimate, Expense, Settings
│   ├── asgard-db/            # sqlx + SQLite, migrations, repositories
│   ├── asgard-pdf/           # rendu facture / devis
│   └── asgard-mail/          # lettre, SMTP, résolution des gabarits
├── src-tauri/                # commandes Tauri, état applicatif, keyring
└── src/                      # React — inchangé jusqu'en phase 5
```

---

## 4. Plan de migration

### Phase 0 — Filet de sécurité *(3-4 j, aucun Rust)* ✅ **livrée**

Sans tests, toute migration est une réécriture à l'aveugle. Cette phase produit l'oracle.

> **Livré** — 7 modules dans `src/domain/`, 151 golden tests, jeu de référence et
> 22 PDF archivés. `App.jsx` passe de 2 060 à 1 748 lignes, `ComptaTab.jsx` de
> 418 à 361. D7 et D9 corrigés. Voir [`src/domain/README.md`](../src/domain/README.md).

1. Extraire la logique de `App.jsx` et `ComptaTab.jsx` vers `src/domain/*.js` (fonctions pures, sans React).
2. Écrire des tests Vitest sur ces fonctions — ce sont les **golden tests** qui seront rejoués contre l'implémentation Rust en phase 3.
3. Figer un jeu de données de référence (`fixtures/*.json`) couvrant : les 3 types d'activité, ACRE activé/désactivé, factures à cheval sur deux exercices, tous les statuts.
4. Archiver les PDF produits par jsPDF sur ces fixtures — référence visuelle pour la phase 4.
5. Corriger **D7**, supprimer **D9**.

*Critère de sortie* : `npm test` vert, comportement actuel figé et documenté. **Atteint.**

Un point non anticipé est apparu : `pdfGenerator.js` imprimait `new Date()` en pied de page, et jsPDF horodate le PDF puis tire un identifiant de document aléatoire. Les PDF de référence changeaient donc à chaque génération. La date d'édition est devenue un paramètre injectable, l'horodatage est figé et l'identifiant normalisé — sans quoi la référence n'aurait été comparable à rien.

### Phase 1 — Remplacer la coquille Electron par Tauri *(5-7 j)* ✅ **livrée**

Le frontend n'est pas touché. Seuls le shell et les deux IPC changent.

> **Livré** — `src-tauri/` (hôte Rust), SMTP porté sur `lettre`, secrets dans le
> trousseau de l'OS, `src/ipc.js` comme frontière unique. `main.js` supprimé ;
> `electron`, `electron-builder`, `nodemailer` et `concurrently` retirés du
> `package.json`.

1. `cargo tauri init` ; `tauri.conf.json` pointe sur le serveur Vite en dev, sur `dist/` en production.
2. Réimplémenter `send-email` et `test-smtp` en commandes Tauri avec **lettre 0.11**. Reproduire exactement la logique TLS de `main.js` :
   - `secure: 'ssl'` → `SmtpTransport::relay(host)` (TLS implicite, port 465) ;
   - `secure: 'none'` → `SmtpTransport::starttls_relay(host)` ;
   - certificat auto-signé accepté **uniquement** si `host` vaut `127.0.0.1` ou `localhost` (Proton Mail Bridge), via `TlsParameters::builder(...).dangerous_accept_invalid_certs(true)`.
3. Introduire `src/ipc.js` qui encapsule `invoke()` — point de changement unique pour toutes les phases suivantes. Remplacer les 2 appels `window.require('electron').ipcRenderer`.
4. Sortir `smtpPass` de `localStorage` vers `keyring` (**D2**).
5. `electron-builder` → `tauri build` ; cible NSIS conservée sur Windows.
6. Supprimer `electron`, `electron-builder`, `nodemailer`, `concurrently` du `package.json`.

*Critère de sortie* : application fonctionnellement identique, installeur < 15 Mo, plus aucun module Node dans le renderer. **D1, D2, D10 résolus.**

**Précisions apportées par l'exécution :**

- La migration du secret ne se limite pas à « les nouvelles écritures vont dans le trousseau ». Les installations existantes ont le mot de passe en clair dans `localStorage`, et une sauvegarde d'avant la phase 1 le contient aussi. `stripSecret` est donc appliqué au chargement, à chaque enregistrement **et à l'import de sauvegarde**, et `migrateLegacySmtpPassword` déplace puis efface l'ancienne entrée au premier lancement. Sans ces trois points, D2 revenait par la porte de derrière.
- Le mot de passe ne traverse plus la frontière IPC : `send_email` et `test_smtp` le lisent eux-mêmes dans le trousseau. Le champ `pass` reste accepté pour le seul bouton « Tester la connexion », qui doit pouvoir valider des identifiants avant enregistrement.
- **À vérifier manuellement sur l'application empaquetée** : l'export CSV du livre des recettes et l'export de sauvegarde passent par un `<a download>` pointant sur un `blob:` et un `data:` URI. La CSP de Tauri est plus stricte que celle d'Electron et ces chemins n'ont pas pu être testés dans l'environnement de développement. Ils deviennent de toute façon des opérations de fichier côté Rust en phase 2, où la gestion de fichiers a sa place.

**Piège rencontré après coup, corrigé :** un binaire compilé avec `cargo build --release` au lieu de `npm run dist` sert le frontend depuis `devUrl` et n'affiche qu'un « localhost a refusé de se connecter ». L'application se lance normalement, rien n'apparaît dans les journaux, et la seule différence visible est la taille — 3,88 Mo sans les fichiers embarqués contre 4,22 Mo avec. `build.rs` interrompt désormais la compilation dans ce cas, en détectant l'absence de `TAURI_CLI_VERBOSITY` (variable que seule la CLI renseigne) et en indiquant la commande à utiliser.

**Leçon pour les phases suivantes :** vérifier qu'un processus démarre ne prouve rien sur ce qu'il affiche. Les binaires de bureau se vérifient en capturant la fenêtre.

**Mesure après la phase 2 :** le binaire passe de 4,22 à 5,66 Mo — `sqlx` et le moteur SQLite embarqué. La base elle-même vit à part, dans `%APPDATA%\com.asgard.crmsgard-crm.sqlite`.

**Mesures relevées sur la compilation release (phase 1) :**

| | Avant (estimé) | Après (mesuré) |
|---|---|---|
| Installeur | ~220 Mo | **1,82 Mo** |
| Binaire applicatif | — | 4,22 Mo |
| Processus hôte au repos | ~200 Mo | 30 Mo |

L'installeur est aussi léger parce que Tauri n'embarque pas WebView2 : il utilise le mode `downloadBootstrapper`, qui le télécharge à l'installation s'il est absent. WebView2 est présent d'origine depuis Windows 10 21H2, mais **une machine plus ancienne et hors ligne ne pourra pas installer l'application**. Passer `webviewInstallMode` en `embedBootstrapper` ou `offlineInstaller` corrige cela au prix de la taille — décision à prendre selon le public visé.

Les 30 Mo ne couvrent que le processus hôte : les processus WebView2 enfants n'ont pas pu être attribués proprement sur la machine de test, où d'autres applications utilisent le même moteur. Le total réel est supérieur, tout en restant très en deçà d'Electron, puisque le moteur est partagé avec le système au lieu d'être embarqué.

### Phase 2 — IndexedDB vers SQLite *(5-7 j)* OK **livree**

> **Livre** — `src-tauri/src/db/` (schema, depot, numerotation, reprise, jeu de
> demonstration), 26 tests Rust supplementaires. `src/db.js` n'est plus un schema
> Dexie mais un adaptateur vers les commandes de l'hote ; `dexie` est retire du
> `package.json`. `App.jsx` passe de 1 748 a 1 656 lignes : le jeu de demonstration (182 lignes) s'en va, mais le compte rendu de reprise et les messages d'archivage s'ajoutent.

1. Schéma SQL : montants en **centimes (`INTEGER`)**, `deleted_at TEXT NULL`, table `document_sequences(kind, year, next_value)`, index sur `(payment_date)` et `(client_id)`. Migrations versionnées `sqlx::migrate!`.
2. Commandes Tauri CRUD renvoyant **la même forme d'objets que Dexie** (montants reconvertis en nombre décimal à la frontière) : aucun composant React n'est modifié, seul `src/db.js` devient un adaptateur vers `invoke()`.
3. Reprise des données : commande `import_legacy_backup` acceptant le format d'export actuel (`{ db: {...}, settings: {...} }`). Au premier lancement, si IndexedDB contient des enregistrements, proposer la reprise automatique ; **exiger un export JSON préalable**.
4. Sauvegarde : copie atomique par `VACUUM INTO`, l'export JSON restant disponible pour la compatibilité ascendante.
5. Suppression logique des factures et devis émis (**D4**).

*Critère de sortie* : base fichier inspectable, reprise vérifiée sur une sauvegarde réelle, golden tests toujours verts. **D4, D6 résolus.**

### Phase 3 — Noyau métier Rust *(5-7 j)* ✅ **livrée**

> **Livré** — crate `crates/asgard-core`, sans dépendance à Tauri ni à SQLite :
> `money` (rust_decimal), `urssaf` (une seule implémentation), `thresholds`
> (barèmes datés), `reporting` (agrégats, livre des recettes, CSV). 87 tests,
> tests de propriété compris. Cinq des sept modules de `src/domain` disparaissent.

1. `asgard-core::money` : `rust_decimal`, conversion centimes ↔ décimal, arrondi au centime explicite (**D5**).
2. `asgard-core::urssaf` : une seule implémentation, taux par type d'activité, abattement ACRE 50 %, découpage mensuel/trimestriel — remplace les trois copies (**D8**).
3. `asgard-core::thresholds` : barèmes TVA et plafonds micro-entreprise **paramétrés par année fiscale** plutôt qu'écrits en dur dans le JSX.
4. ~~`asgard-core::numbering`~~ — *fait en phase 2*, la numérotation appartenait au schéma de la base (**D3**).
5. `asgard-core::reporting` : agrégats du tableau de bord, livre des recettes, export CSV (séparateur `;`, décimale `,`, BOM UTF-8 — comme l'existant).
6. Rejouer les golden tests de la phase 0 contre Rust sur les mêmes fixtures JSON. Ajouter des `proptest` sur les arrondis (invariant : `ht + tva == ttc` au centime près).
7. **Produire un rapport d'écarts** : sur les données réelles de l'utilisateur, comparer les totaux `f64` et `Decimal`. Les chiffres vont bouger de quelques centimes — il faut pouvoir l'expliquer, pas le découvrir.

*Critère de sortie* : `App.jsx` réduit à l'état d'UI ; parité numérique documentée. **Atteint — D5 et D8 résolus** (D3 l'avait été en phase 2).

**Précisions apportées par l'exécution :**

- **La parité se démontre sur le même fichier.** `asgard-core::fixtures` embarque `fixtures/reference-dataset.json`, celui que les golden tests JavaScript utilisent depuis la phase 0. Les deux implémentations sont donc éprouvées sur des données identiques, et non sur deux jeux supposés équivalents. Le module `parity` classe chaque valeur en trois catégories : identique, artefact flottant supprimé, ou montant réellement modifié par l'arrondi au centime.
- **D8 est tranché, pas seulement unifié.** Les deux bases qui se contredisaient — factures émises contre encaissées, 1 002,25 € d'écart sur le jeu de référence — existent toujours mais sont nommées : `Basis::Collected` est la seule déclarable à l'URSSAF, `Basis::Issued` est une projection de trésorerie. Conséquence : sur la base encaissée, la somme des mois se recoupe enfin avec le total annuel, ce que l'ancienne implémentation ne permettait pas.
- **Un contrat de sérialisation a failli casser en silence.** `rust_decimal` sérialise par défaut en **chaîne** (`"19650.34"`). L'interface appelle `.toFixed(2)` dessus, ce qui aurait échoué sans aucune erreur côté Rust. Un test de contrat l'a attrapé avant tout travail sur le frontend ; la fonctionnalité `serde-float` corrige le tir.
- **Les montants sont dérivés à l'écriture.** L'interface n'envoie plus que le HT et le taux ; la TVA et le total sont calculés côté Rust. L'invariant `HT + TVA = TTC` est ainsi garanti en base plutôt que de dépendre de l'appelant.
- **Piège de l'espace de travail Cargo.** Déclarer un workspace déplace la sortie de compilation vers `/target` à la racine — que le `.gitignore` de `src-tauri` ne couvre pas — et fait **ignorer** le `[profile.release]` des packages membres. Le binaire est passé de 5,7 à 14,2 Mo sans autre signe qu'un avertissement noyé dans le log de compilation. Profil déplacé à la racine, `/target` ajouté au `.gitignore`.
- **Les tests JavaScript passent de 151 à 49.** Les 102 autres n'ont pas été supprimés : ils ont changé de camp avec le code qu'ils éprouvaient.

### Phase 4 — PDF et e-mail entièrement côté Rust *(4-6 j)* ✅ **livrée**

> **Livré** — crate `crates/asgard-pdf` (printpdf), 22 tests dont la comparaison
> avec les PDF archivés en phase 0. `jspdf` retiré : le bundle passe de 1,19 Mo
> à 296 Ko, `html2canvas` disparaissant avec lui.

1. Réimplémenter `pdfGenerator.js` dans `asgard-pdf`. Deux approches possibles : `printpdf` (impératif, proche du code jsPDF actuel, portage direct) ou **Typst embarqué** (gabarit déclaratif, bien plus maintenable pour les évolutions de mentions légales). Recommandation : `printpdf` pour un portage à l'identique, Typst si les gabarits doivent devenir personnalisables.
   - ~~Attention : il faut embarquer une police, d'où des métriques différentes.~~ **Cette crainte était infondée** : Helvetica fait partie des quatorze polices standard du format PDF, et `printpdf` les expose comme jsPDF. Les métriques sont identiques, sans rien embarquer.
2. L'envoi devient une commande unique `send_document(id, kind)` : le PDF est généré et attaché côté Rust, il ne transite plus en base64 à travers l'IPC.
3. Supprimer `jspdf` du `package.json`.

*Critère de sortie* : plus aucune dépendance JS hors React/Vite ; PDF visuellement conformes. **Atteint.**

**Précisions apportées par l'exécution :**

- **Aucune police à embarquer.** Le plan annonçait des métriques divergentes et une validation page par page. Helvetica étant une police standard du format PDF, `printpdf` l'expose comme jsPDF : les métriques sont les mêmes. Les largeurs de caractères sont reprises des fichiers AFM de la spécification pour calculer l'alignement à droite, que `printpdf` ne fait pas lui-même.
- **La comparaison porte sur le texte, pas sur les octets.** Deux générateurs ne produisent jamais les mêmes octets. Ce qui doit être identique, c'est ce que le lecteur voit : les tests extraient le texte imprimé des deux côtés et vérifient que rien ne manque. Le positionnement, lui, se lit dans le code — les coordonnées sont reprises telles quelles, et un module de mise en page retourne l'axe vertical, jsPDF comptant depuis le haut et le PDF depuis le bas.
- **Deux encodages à gérer.** jsPDF écrit ses chaînes en littéraux `(texte) Tj`, printpdf en hexadécimal `<4153…> Tj`. Les deux sont du WinAnsi, qui coïncide avec le latin-1 **sauf** entre 0x80 et 0x9F — où se trouve le symbole euro. Décoder naïvement en latin-1 transformait chaque « € » en caractère de contrôle et faisait échouer la comparaison des montants.
- **Les exports quittent enfin la page.** Le point resté ouvert depuis la phase 1 est traité : PDF, livre des recettes et sauvegarde JSON s'écrivent désormais par le sélecteur de fichiers du système, côté Rust, au lieu d'un `<a download>` sur une URL `blob:` dont le comportement sous la CSP n'avait jamais pu être vérifié.
- **`templates.js` reste en JavaScript.** Le plan prévoyait de le déplacer. Le message d'e-mail est composé puis **relu et modifié par l'utilisateur** avant envoi : le faire côté hôte imposerait un aller-retour pour afficher un brouillon destiné à être retouché. Le PDF, lui, n'est jamais relu avant envoi.

### Phase 5 — Frontend Rust *(optionnelle, 4-6 semaines)*

Leptos 0.7 (CSR) ou Dioxus 0.6, CSS conservé à l'identique, types partagés depuis `asgard-core` — plus de désynchronisation possible entre les formes d'objets du front et du back.

**À ne lancer que si** le besoin de typage de bout en bout devient tangible. Après la phase 4, React n'est plus qu'une couche de présentation : le garder est un choix parfaitement défendable.

---

## 5. Correspondance des dépendances

| JavaScript | Rust | Note |
|---|---|---|
| `electron` | `tauri` 2 | WebView2 sur Windows, WebKitGTK sur Linux |
| `dexie` / IndexedDB | `sqlx` + SQLite | ou `rusqlite` si le mode synchrone suffit |
| `nodemailer` | `lettre` 0.11 | STARTTLS + TLS implicite + certificats auto-signés |
| `jspdf` | `printpdf` 0.7 | ou `typst` pour des gabarits déclaratifs |
| `html2canvas` | — | dépendance morte, à supprimer |
| `localStorage` (réglages) | `tauri-plugin-store` | ou fichier TOML + `serde` |
| `localStorage` (`smtpPass`) | `keyring` 3 | trousseau Windows / Keychain / Secret Service |
| `new Date()` | `chrono` | `NaiveDate` pour les dates comptables, pas d'heure |
| `f64` (montants) | `rust_decimal` | `i64` centimes en base |
| `JSON.parse` / `stringify` | `serde_json` | |
| — | `cargo test` + `proptest` + `insta` | il n'existe aucun test aujourd'hui |

---

## 6. Risques

| Risque | Gravité | Mitigation |
|---|---|---|
| Perte de données au passage IndexedDB vers SQLite | **Critique** | Export JSON obligatoire avant migration ; import idempotent ; IndexedDB conservée en lecture seule pendant une version |
| Écarts d'arrondi `f64` vers `Decimal` sur des chiffres déjà déclarés | Majeur | Rapport d'écarts en phase 3 ; aucune modification rétroactive des exercices clos |
| Rendu PDF différent (métriques de police) | Majeur | Comparaison page par page contre les PDF archivés en phase 0 ; jsPDF gardé en secours une version |
| Régression Proton Mail Bridge (certificat auto-signé) | Majeur | Reproduire exactement la règle `host === '127.0.0.1'` ou `host === 'localhost'` ; test manuel avec un Bridge réel |
| Séquence de numérotation cassée à la reprise | Majeur | `next_value` initialisé à `MAX(numéro existant) + 1`, jamais au nombre de lignes |
| WebView2 absent sur un poste Windows ancien | Mineur | Présent depuis Windows 10 21H2 ; sinon bootstrapper embarqué dans l'installeur NSIS |
| Montée en compétence Rust | Variable | Les phases 1 et 2 demandent peu de Rust ; la difficulté réelle arrive en phase 3 |

---

## 7. Effort et gains

| Phase | Charge | Résout |
|---|---|---|
| 0 — Filet de sécurité ✅ | 3-4 j | D7, D9 |
| 1 — Coquille Tauri ✅ | 5-7 j | D1, D2, D10 |
| 2 — SQLite OK | 5-7 j | D3, D4, D6 (+ D5 amorce) |
| 3 — Noyau métier ✅ | 5-7 j | D5 (fin), D8 |
| 4 — PDF & e-mail ✅ | 4-6 j | — |
| **Total phases 0-4** | **22-31 j** (≈ 5-6 semaines) | **les 10 défauts** |
| 5 — Frontend Rust *(option)* | +4-6 semaines | — |

**Gains attendus à l'issue de la phase 4**

| | Avant | Après |
|---|---|---|
| Installeur | ~220 Mo | ~10 Mo |
| RSS au repos | ~200 Mo | ~80 Mo |
| Démarrage à froid | ~2 s | < 0,5 s |
| Couverture de tests de la logique comptable | 0 % | logique métier intégralement couverte |
| Node dans le processus de rendu | oui | non |
| Secrets en clair sur le disque | oui | non |

---

## 8. Premier pas concret

La phase 0 ne contient pas une ligne de Rust et a de la valeur même si la migration est ensuite abandonnée : elle sort la logique comptable du composant React et la met sous tests. C'est par là qu'il faut commencer.
