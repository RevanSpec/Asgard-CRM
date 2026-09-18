# `asgard-ui` — interface Leptos

Phase 5 du [plan de migration](../../docs/MIGRATION_RUST.md), la seule que le
plan classait **optionnelle**. Son état est partiel, et volontairement : lisez
la section « Où en est le portage » avant de vous y fier.

## Ce que cette phase apporte réellement

Un seul bénéfice, mais il est net : **l'interface et l'hôte compilent le même
crate de types**, [`asgard-ipc`](../asgard-ipc/src/lib.rs). Un champ renommé
d'un côté ne compile plus de l'autre.

Le JavaScript n'avait pas cette garantie. `src/db.js` construisait des objets
anonymes et lisait ce que l'hôte voulait bien renvoyer : une divergence ne se
voyait qu'à l'exécution, sur un `undefined` inattendu, et parfois seulement dans
un cas d'usage rare.

Ce n'est pas rien, mais **l'utilisateur ne verra aucune différence** — ce que le
plan annonçait déjà en estimant le retour de cette phase faible.

## Où en est le portage

| Écran | État |
|---|---|
| Coquille, navigation, fenêtres de message | porté |
| Tableau de bord | porté |
| Clients | porté, avec création, édition et suppression |
| Devis, Factures, Dépenses, Comptabilité, Paramètres | **non portés** |

Les onglets non portés affichent un message explicite plutôt qu'un écran vide,
qui ressemblerait à une panne.

`crates/asgard-ui/src/views/clients.rs` sert de gabarit aux onglets restants :
recherche, tableau, formulaire, appel à l'hôte, rechargement. Le schéma se
répète pour les factures, devis et dépenses. Les deux gros morceaux sont
`ComptaTab` et `SettingsTab`, environ 400 lignes de JSX chacun.

## Le thème n'est pas réécrit

`index.html` charge `src/index.css` et `src/App.css` **tels quels**. Les classes
du balisage sont celles de l'original, au nom près. C'est la garantie la plus
simple que le rendu ne dérive pas, et cela évite de refaire un design qui
n'avait aucune raison de recommencer.

Un écueil rencontré : la barre latérale avait d'abord été écrite avec des
`<button>`, plus corrects sémantiquement que les `<li>` de l'original. Les
styles par défaut du navigateur — fond blanc, bordure — sont alors apparus, la
feuille de style n'ayant jamais eu à les neutraliser. Le balisage suit donc
l'original, avec `role="button"` et `tabindex` ajoutés pour l'accès au clavier,
que la version React n'offrait pas.

## Développer

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk

trunk serve --config crates/asgard-ui/Trunk.toml   # port 5174
trunk build --release --config crates/asgard-ui/Trunk.toml
```

Servi seul, sans l'hôte Tauri, l'interface s'affiche mais toute commande échoue
avec un message explicite : il n'y a pas de base à interroger. C'est utile pour
travailler la mise en page, pas pour manipuler des données.

## Poids

| | Taille |
|---|---|
| WebAssembly, profil release | **695 Ko** |
| Liaison JavaScript | 38 Ko |
| *Bundle React, pour comparaison* | *289 Ko, pour sept écrans sur sept* |

Le profil de développement produit 4 Mo : ne pas s'y fier pour juger. Porter les
cinq écrans restants ajoutera peu — l'essentiel du poids est le moteur Leptos et
la bibliothèque standard — mais le total restera au-dessus de React.

## Ce qui reste à décider

Le `Trunk.toml` existe, mais `tauri.conf.json` pointe toujours sur Vite. Tant
que le portage n'est pas complet, **c'est la version React que l'application
empaquetée sert** — basculer maintenant priverait l'utilisateur de cinq écrans
sur sept.
