# Feuille de route — de la migration terminée au produit fini

> Analyse du 29/09/2026, après la fusion de #11 (intégration continue et parcours
> de l'application).
> La migration vers Rust est close — voir [`MIGRATION_RUST.md`](MIGRATION_RUST.md).
> Ce document ne traite que de ce qui sépare encore l'application d'un produit
> qu'on peut installer chez quelqu'un sans rien lui expliquer.

---

## 1. Ce qui est acquis

| | État |
|---|---|
| Migration Rust | Six phases livrées, plus aucun JavaScript applicatif |
| Tests | Espace de travail sous `cargo test`, clippy sans indulgence, hôte et cible WebAssembly |
| Parcours | L'application empaquetée est lancée et traversée à chaque proposition (19 étapes, échec à la moindre erreur de console) |
| Comptabilité | Numérotation continue, suppression logique, montants en centimes, livre des recettes, déclaration URSSAF |
| Facturation | Échéance calculée, mentions de retard, avoirs partiels ou totaux |
| Robustesse | Panne d'ouverture annoncée par une boîte de dialogue, paniques de l'interface affichées, journaux sur disque, copie de la base une fois par jour |

Le produit **fonctionne**. Ce qui suit relève de ce qu'un utilisateur découvre le
jour où il s'en sert pour de vrai.

---

## 2. Défauts recensés

Numérotation continuée depuis [`MIGRATION_RUST.md`](MIGRATION_RUST.md) (D1–D10).

### Critique

**D11 — Identité d'emprunt par défaut.** `crates/asgard-ui/src/settings.rs:50` :
les réglages par défaut ne sont pas vides, ils décrivent une société fictive —
« Asgard Solutions », « Thor Odinson », SIRET `839 204 123 00019`, IBAN
`FR76 3000 2000 0001 2345 6789 012`. Tant que l'utilisateur ne va pas dans les
réglages, ses factures partent au nom de cette société, avec un SIRET faux mais
plausible et un IBAN qui ne lui appartient pas. Le client ne peut pas payer, et
la pièce est irrégulière.

Le repli prévu côté hôte (`crates/asgard-pdf/src/lib.rs:309`, qui remplace un
IBAN vide par `FR76 0000 …`) n'est donc presque jamais atteint : il ne protège
de rien, et imprimerait de toute façon un compte inexistant.

### Majeur

**D12 — Jeu de démonstration semé dans la base de l'utilisateur.**
`src-tauri/src/lib.rs:396` appelle `seed_demo_data_if_empty` à chaque ouverture :
une base vierge reçoit trois clients fictifs, des factures, des devis et des
dépenses. Outre la confusion, **les factures de démonstration consomment la
séquence réglementaire** : la première facture réelle ne porte pas le numéro 1,
ce qui est exactement ce que la numérotation continue devait éviter.

**D13 — Les réglages échappent aux copies automatiques.** Les réglages vivent
dans le `localStorage` de la WebView (`crates/asgard-ui/src/settings.rs:12`), pas
en base. La copie quotidienne est un `VACUUM INTO` du fichier SQLite : elle ne
couvre donc ni l'identité, ni le logo, ni les gabarits d'e-mail, ni la
configuration SMTP. Un profil WebView nettoyé les efface en silence, et seule
l'exportation JSON manuelle les embarque (`crates/asgard-ui/src/backup.rs:26`).
*Constat postérieur à la première liste — il appartient au même lot que D11.*

**D14 — Fiche client sans identifiant, mentions 2026 absentes.** La table
`clients` (`src-tauri/migrations/0001_initial.sql:21`) ne porte ni SIREN ni
numéro de TVA. Or les mentions ajoutées par le décret n° 2022-1299, qui entrent
en vigueur avec l'obligation de facturation électronique, exigent le **SIREN du
client**, l'**adresse de livraison** si elle diffère, la **nature de l'opération**
(biens, services ou mixte) et, le cas échéant, l'option de paiement de la TVA
d'après les débits. Aucune de ces données n'est saisie aujourd'hui.

Accessoirement, [`README.md:10`](../README.md) annonce des fiches clients
« complètes (coordonnées, SIRET, historique d'affaires) » : le SIRET client
n'existe pas. À corriger dans le même mouvement.

### Mineur

**D15 — Mentions liées à l'activité.** Rien n'est prévu pour le **médiateur de
la consommation**, obligatoire dès qu'on facture des particuliers (art. L616-1
du code de la consommation), ni pour l'**assurance professionnelle**, obligatoire
pour les activités qui y sont soumises (art. L112-11 du code des assurances).
Ces mentions dépendent de l'activité : elles ne peuvent pas être écrites en dur.

**D16 — Les polices ne se chargent jamais.** `crates/asgard-ui/style/index.css:1`
importe Inter et Outfit depuis Google Fonts ; la CSP
(`src-tauri/tauri.conf.json:25`, `font-src 'self' data:`) bloque la requête.
L'application s'affiche avec les polices du système, jamais avec celles pour
lesquelles le thème a été dessiné. Le parcours tolère explicitement l'erreur de
console correspondante (`scripts/smoke.mjs:29`) : la tolérance disparaîtra avec
la cause.

**D17 — Ni licence, ni installeur publié.** Le dépôt n'a aucun fichier de
licence et aucun `license` dans les manifestes, alors que la diffusion en source
ouverte est envisagée. Rien ne construit ni ne publie l'installeur : il faut
compiler soi-même pour essayer le produit.

**D18 — `.oxlintrc.json`, reliquat de l'ère JavaScript.** Plus aucun linter JS
n'est configuré dans `package.json`, qui ne garde que la CLI Tauri.

---

## 3. Plan d'intégration

Quatre lots. A et B touchent les mêmes écrans et le même chemin d'émission :
les enchaîner évite de payer deux fois la relecture. C et D sont indépendants et
peuvent glisser.

### Lot A — Un premier lancement honnête *(2-3 j)* — D11, D12, D13 ✅ **livré**

> **Livré.** `Settings::default()` n'a plus d'identité, `Issuer::missing_fields`
> refuse l'édition d'une pièce incomplète, l'écran d'accueil
> (`crates/asgard-ui/src/views/onboarding.rs`) demande les quatre champs exigés et
> propose le jeu d'exemple au lieu de le semer, et les réglages vivent dans la
> table `app_settings` — donc dans les copies quotidiennes. Le parcours de
> l'application couvre l'accueil, son refus sans identité, et le refus d'éditer
> une facture sans SIRET.

L'objectif : sur une machine vierge, **aucune facture ne peut sortir avec une
identité qui n'est pas celle de l'utilisateur**.

1. **Vider l'identité par défaut.** `Settings::default()` ne garde que ce qui est
   un réglage (délai de règlement, taux URSSAF, gabarits d'e-mail, couleur) et
   laisse vides raison sociale, contact, adresse, SIRET, IBAN, e-mail,
   téléphone. Les installations existantes ne bougent pas : `#[serde(default)]`
   ne s'applique qu'aux champs absents.
2. **Un garde à l'émission.** Une fonction `Issuer::missing_fields()` dans
   `asgard-pdf`, appelée par `src-tauri/src/documents.rs` avant tout rendu :
   raison sociale, adresse, SIRET et IBAN manquants font échouer la commande
   avec la liste des champs à remplir, que l'interface affiche en renvoyant vers
   les réglages. Le repli `FR76 0000 …` disparaît — un IBAN absent devient une
   erreur, pas une invention.
3. **Un écran d'accueil au premier lancement**, qui demande ces quatre champs et
   propose, en une case à cocher, de **partir à vide ou avec un jeu d'exemple**.
   Le semis (`seed_demo_data_if_empty`) sort du démarrage et devient une commande
   appelée par ce choix.
4. **Les réglages en base.** Migration `0004_settings.sql` : une table
   `app_settings(key TEXT PRIMARY KEY, value TEXT NOT NULL)`, la reprise du
   `localStorage` au premier lancement suivant la mise à jour — exactement le
   schéma déjà employé pour le mot de passe SMTP en phase 1 : lire l'ancien
   emplacement, écrire le nouveau, effacer l'ancien. Les copies quotidiennes
   couvrent alors l'identité, le logo et la configuration SMTP.

*Fichiers* : `crates/asgard-ui/src/{settings.rs,state.rs,views/settings_view.rs}`,
un nouveau `views/onboarding.rs`, `src-tauri/src/{lib.rs,documents.rs,db/seed.rs}`,
`crates/asgard-pdf/src/lib.rs`, nouvelle migration.

*Tests* : un test d'`asgard-pdf` par champ obligatoire manquant ; un test de
reprise des réglages depuis l'ancien format ; le test existant du semis devient
un test de la commande.

*Parcours* : il dépend aujourd'hui du jeu de démonstration pour avoir un contenu
prévisible. Il gagne une première étape — remplir l'accueil, choisir le jeu
d'exemple — avant les dix-neuf actuelles, et une étape négative : une facture
refusée tant que l'identité est incomplète.

*Critère de sortie* : sur une base vierge, la première facture porte le numéro 1
et l'identité saisie ; une restauration de la copie du jour rend aussi les
réglages.

*Piège* : toute nouvelle migration est contrôlée par somme de contrôle sur le
contenu du fichier. `.gitattributes` fixe déjà `*.sql` en fins de ligne LF —
ne pas y toucher.

### Lot B — Conformité de la facture *(2-3 j)* — D14, D15 ✅ **livré**

> **Livré.** Migration `0005_client_identifiers` : la fiche client porte SIREN,
> numéro de TVA et adresse de livraison ; les factures et devis portent la nature
> de l'opération, proposée d'après le type d'activité et modifiable. Le PDF
> imprime les quatre mentions du décret, et deux champs de réglages — médiateur,
> assurance — s'impriment s'ils sont remplis. Le README annonçait un SIRET client
> qui n'existait pas : il est désormais exact.

L'objectif : **saisir et stocker dès maintenant tout ce que la facturation
électronique exigera**, pour n'avoir plus qu'un format à produire le jour venu.

1. **Migration `0005_client_identifiers.sql`** : `clients` gagne `siren`,
   `vat_number` et `delivery_address` ; `invoices` et `estimates` gagnent
   `operation_kind` (`biens` / `services` / `mixte`), déduit par défaut du type
   de prestation déjà saisi.
2. **Saisie** : les champs apparaissent dans la fiche client
   (`crates/asgard-ui/src/views/clients.rs`) et dans le formulaire de facture
   pour la nature de l'opération, qui reste modifiable.
3. **Impression** : le bloc « FACTURÉ À » porte le SIREN et, s'il existe, le
   numéro de TVA ; le pied de page porte la nature de l'opération et l'adresse
   de livraison quand elle diffère du siège.
4. **Mentions d'activité (D15)** : deux champs libres dans les réglages —
   médiateur de la consommation, assurance professionnelle — imprimés seulement
   s'ils sont remplis, sous les mentions de retard existantes.
5. **README corrigé** sur les fiches clients.

*Tests* : les tests de parité PDF comparent au rendu de référence ; comme pour
l'échéance, ils doivent porter une **exception explicite et nommée** pour chaque
ligne nouvelle, plus des assertions positives sur le libellé ajouté. Un rendu qui
change sans exception déclarée doit rester un échec.

*Critère de sortie* : aucune donnée exigée par le profil minimum de Factur-X ne
manque en base. L'émission du format lui-même reste hors périmètre, mais elle ne
demandera plus de migration de données ni de ressaisie client.

### Lot C — Finition visible *(0,5 j)* — D16, D18 ✅ **livré**

> **Livré.** Inter et Outfit vivent dans `crates/asgard-ui/assets/fonts`,
> sous-ensembles latins seulement (~180 ko), avec le texte de leur licence OFL.
> `scripts/fetch-fonts.mjs` les récupère et produit `style/fonts.css` ; l'import
> distant a disparu. Le parcours vérifie que les deux faces sont chargées et
> qu'aucune requête ne sort de la machine, et sa liste de messages tolérés est
> désormais vide. `.oxlintrc.json` est supprimé.

1. Récupérer Inter et Outfit en `woff2` dans `crates/asgard-ui/assets/fonts/`,
   les déclarer en `@font-face` local, supprimer l'`@import` distant. Trunk copie
   le dossier (`<link data-trunk rel="copy-dir" href="assets/fonts" />`).
2. Retirer `fonts.googleapis.com` des messages tolérés du parcours
   (`scripts/smoke.mjs:29`) et ajouter une assertion :
   `document.fonts.check('16px Outfit')` doit être vrai. La CI attrape alors
   toute régression typographique.
3. Supprimer `.oxlintrc.json`.

*Critère de sortie* : l'application ne fait plus aucune requête réseau au
démarrage, et s'affiche avec les polices prévues.

### Lot D — Diffusion en source ouverte *(1 j)* — D17

1. **Choisir une licence** et l'écrire : fichier `LICENSE`, champ `license` dans
   les manifestes, mention dans le README. MIT si l'objectif est que le code
   circule ; AGPL-3.0 si les versions dérivées doivent rester ouvertes.
2. **Workflow `release.yml`**, déclenché sur une étiquette `v*` : `npm run dist`,
   puis publication de l'installeur NSIS en pièce jointe d'une *release* GitHub.
   La version publiée est celle de `src-tauri/tauri.conf.json` (`0.1.0`) — à
   aligner avec `package.json`, resté à `0.0.0`.
3. **README** : avertissement SmartScreen (l'installeur n'est pas signé, décision
   assumée) et rappel que l'installeur télécharge WebView2 s'il est absent, donc
   qu'une machine hors ligne et ancienne ne pourra pas l'installer.

*Critère de sortie* : `git tag v0.2.0 && git push --tags` produit une release
téléchargeable et installable sur une machine neuve.

---

## 4. Ordre, effort, dépendances

| Lot | Défauts | Effort | Dépend de | Pourquoi ce rang |
|---|---|---|---|---|
| **A** — Premier lancement honnête | D11, D12, D13 | 2-3 j | — | Seul lot dont l'absence produit une facture fausse |
| **B** — Conformité de la facture | D14, D15 | 2-3 j | A (mêmes écrans) | Échéance réglementaire connue, migration de données à faire tôt |
| **C** — Finition visible | D16, D18 | 0,5 j | — | Peu coûteux, visible immédiatement |
| **D** — Diffusion | D17 | 1 j | C (on publie ce qu'on montre) | Dernier pas avant de rendre le dépôt public |

**Total : 6 à 8 jours.**

---

## 5. Ce que l'intégration continue doit gagner

| Lot | Vérification ajoutée |
|---|---|
| A | Étape « premier lancement » dans le parcours ; refus d'émettre sans identité ; test de reprise des réglages |
| B | Tests de parité PDF avec exceptions nommées ; test de migration sur une base d'avant |
| C | `document.fonts.check` dans le parcours ; tolérance Google Fonts retirée |
| D | Construction de l'installeur sur étiquette — un échec de publication ne doit pas rendre la CI rouge |

---

## 6. Hors périmètre, et pourquoi

- **Factur-X et plateforme de dématérialisation.** L'obligation arrive par
  paliers à partir de 2026-2027 et le format n'est pas stabilisé pour les
  micro-entreprises en franchise. Le lot B suffit à ne pas avoir à migrer de
  données le jour venu ; produire le XML sera un travail de rendu, pas de modèle.
- **Signature de code et mise à jour automatique.** Écartées : le projet reste
  personnel, au mieux diffusé en source ouverte, sans objectif de
  commercialisation. Un certificat coûte plusieurs centaines d'euros par an pour
  supprimer un avertissement que le README peut expliquer.
- **Mode hors Windows.** L'hôte compile ailleurs, mais rien n'est vérifié : ni
  parcours, ni installeur, ni trousseau. À ouvrir seulement si le besoin existe.
