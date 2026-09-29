# `asgard-ui` — interface Leptos

Phase 5 du [plan de migration](../../docs/MIGRATION_RUST.md). Depuis sa
livraison, **c'est l'interface que l'application sert** : React, Vite et le
JavaScript applicatif ont été retirés du dépôt.

## Ce que cette phase apporte réellement

Un bénéfice net : **l'interface et l'hôte compilent le même crate de types**,
[`asgard-ipc`](../asgard-ipc/src/lib.rs). Un champ renommé d'un côté ne compile
plus de l'autre. Le JavaScript n'avait pas cette garantie : `src/db.js`
construisait des objets anonymes, et une divergence ne se voyait qu'à
l'exécution, sur un `undefined` inattendu.

L'interface s'appuie aussi sur `asgard-core` : l'aperçu du calcul de TVA, par
exemple, arrondit exactement comme l'hôte qui enregistrera la facture.

## Organisation

| Module | Rôle |
|---|---|
| `state.rs` | État partagé (remplace les 39 `useState` de `App.jsx`) |
| `actions.rs` | Appels à l'hôte : appeler, recharger, informer — toujours dans cet ordre |
| `backup.rs` | Sauvegarde et restauration |
| `validation.rs`, `templates.rs` | Validation des formulaires, gabarits d'e-mail |
| `views/widgets.rs` | Fenêtre, champs, boutons — partagés au lieu d'être recopiés par écran |
| `views/*.rs` | Un module par écran, plus les fenêtres communes (`modals.rs`) |

## Fidèle à l'original, sauf là où l'original se trompait

Chaque texte visible, chaque classe CSS, chaque style en ligne reprend la
version React. Le contrôle a été fait mécaniquement au moment de la bascule :
tous les textes des composants JSX se retrouvent dans le port, et toutes les
classes CSS utilisées existent dans le thème.

Les écarts sont délibérés, et chacun est commenté à l'endroit du code :

- **Badge et relance des factures envoyées.** L'original testait `'envoye'`
  (l'orthographe des devis) au lieu de `'envoyee'` : une facture envoyée
  n'avait aucun badge, et le bouton « Relancer » n'apparaissait jamais.
- **Envoi d'un devis par e-mail.** La fenêtre annonçait « la facture » et la
  pièce jointe s'affichait « undefined.pdf ». Elle nomme désormais la bonne pièce.
- **Client supprimé.** Le jeton `{clientName}` restait vide ; il reprend la raison
  sociale recopiée sur la pièce.
- **Montants saisis avec une virgule.** `1899,99` était lu `1899`. Il est lu
  correctement.
- **Suppression d'une facture émise.** Le message n'annonce plus une suppression
  « irréversible » : depuis la phase 2, une facture émise est archivée.
- **Taux URSSAF illisible.** Il conserve la valeur précédente au lieu de passer à
  zéro, ce qui annulait les cotisations sans prévenir.

## Le thème n'est pas réécrit

`style/index.css` et `style/App.css` sont les feuilles de la version React,
déplacées sans modification. Un écueil rencontré : la barre latérale avait
d'abord été écrite avec des `<button>`, plus corrects sémantiquement que les
`<li>` de l'original. Les styles par défaut du navigateur — fond blanc,
bordure — sont alors apparus, la feuille n'ayant jamais eu à les neutraliser.
Le balisage suit donc l'original, avec `role="button"` et `tabindex` ajoutés
pour l'accès au clavier.

## Développer

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked

npm run dev                  # trunk serve + hôte Tauri, recompilation à chaud
cargo test -p asgard-ui      # tests de l'interface
```

L'interface ne se sert pas seule : toutes ses données passent par les commandes
de l'hôte. Ouverte dans un navigateur, elle affiche sa coquille mais ne peut
rien charger.

## Poids

| | Taille |
|---|---|
| WebAssembly, profil release | **1,8 Mo** |
| Liaison JavaScript | 41 Ko |
| *Bundle React, pour comparaison* | *289 Ko* |

Mesuré sur `trunk build --release` avec le profil de l'espace de travail
(`opt-level = "s"`, LTO). Avec deux écrans portés, le WebAssembly pesait
695 Ko : les cinq autres ont plus que doublé le total. Chaque `view!` de
Leptos engendre son propre type, donc son propre code — le poids suit le nombre
d'écrans bien plus qu'en React. `wasm-opt` réduirait ce chiffre, mais trunk le
télécharge à la première compilation — une dépendance réseau que le build n'a
pas aujourd'hui.

Sur une application de bureau servie localement, la différence ne se remarque
pas au chargement ; l'argument « plus léger » ne tient simplement pas.
