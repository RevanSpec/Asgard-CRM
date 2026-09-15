# `src-tauri` — processus hôte

Phase 1 du [plan de migration vers Rust](../docs/MIGRATION_RUST.md). Remplace
`main.js`, qui créait la fenêtre Electron et hébergeait les deux handlers IPC.

## Ce qui a changé sur le fond

### Le webview n'a plus accès à Node (défaut D1)

L'ancienne fenêtre était créée ainsi :

```js
webPreferences: {
  nodeIntegration: true,
  contextIsolation: false,
}
```

Sans script de preload, cette configuration donnait à tout contenu rendu un
accès direct à `require('fs')` et `require('child_process')` — y compris aux
logos chargés par l'utilisateur et aux sauvegardes JSON importées.

Tauri n'expose aucun runtime au webview. L'interface ne peut appeler que les
cinq commandes déclarées dans [`src/lib.rs`](src/lib.rs), et
[`capabilities/default.json`](capabilities/default.json) ne concède que
`core:default`.

### Le mot de passe SMTP ne traverse plus la frontière (défaut D2)

Il vivait en clair dans `localStorage` et repartait tel quel dans les fichiers
de sauvegarde. Il est désormais dans le trousseau du système
([`src/secrets.rs`](src/secrets.rs)) et lu par l'hôte au moment de l'envoi.
L'interface peut l'écrire et savoir s'il existe, jamais le relire.

Le champ `pass` de `SmtpConfig` reste accepté pour un seul cas : le bouton
« Tester la connexion », qui doit pouvoir valider des identifiants avant
qu'ils ne soient enregistrés.

## Fidélité au comportement JavaScript

`nodemailer` et `lettre` ne font pas les mêmes choix par défaut. Les
correspondances retenues, testées dans [`src/mail.rs`](src/mail.rs) :

| Réglage | nodemailer | lettre |
|---|---|---|
| `secure: 'ssl'` | `secure: true`, TLS implicite (465) | `Tls::Wrapper` |
| toute autre valeur | `secure: false`, STARTTLS si annoncé | `Tls::Opportunistic` |
| `127.0.0.1` / `localhost` | `rejectUnauthorized: false` | `dangerous_accept_invalid_certs(true)` |
| tout autre hôte | vérification normale | vérification normale |

`Tls::Opportunistic` et non `Tls::Required` : nodemailer avec `secure: false`
tentait STARTTLS sans l'exiger, et `Required` ferait échouer un serveur qui ne
l'annonce pas.

La règle sur la boucle locale est une **égalité stricte**, comme en JavaScript.
`::1`, `LOCALHOST` et `localhost.evil.com` ne sont pas exemptés — un test le
vérifie explicitement.

## Commandes exposées

| Commande | Rôle |
|---|---|
| `send_email` | Envoie un document, pièce jointe PDF comprise |
| `test_smtp` | Vérifie la connexion sans envoyer |
| `set_smtp_password` | Écrit le secret dans le trousseau (chaîne vide = efface) |
| `has_smtp_password` | Dit si un secret existe, sans le divulguer |
| `clear_smtp_password` | Supprime le secret |

Les deux premières renvoient `{ success, messageId?, error? }` — la forme que
le JSX attendait déjà des handlers Electron, pour ne pas toucher aux vues.

L'envoi est bloquant : il part sur `spawn_blocking` afin que la négociation TLS
ne fige pas l'interface.

## Icône

L'icône est générée par [`scripts/generate-app-icon.mjs`](../scripts/generate-app-icon.mjs)
— un écu doré sur bleu nuit, aux couleurs de l'interface — puis déclinée par
`tauri icon`. Le script est versionné pour que l'icône reste reproductible :

```bash
npm run icon
```

## Développer

```bash
npm run dev     # Vite + compilation Rust + fenêtre
npm run dist    # installeur NSIS
cargo test      # tests de l'hôte
```

### ⚠️ Ne pas compiler avec `cargo build --release`

Un binaire release doit être produit par la CLI Tauri, jamais par cargo seul.
La CLI fait deux choses que `cargo build` ignore :

1. elle exécute `beforeBuildCommand` (`npm run build`), qui régénère `dist/` ;
2. elle indique à `tauri::generate_context!` d'embarquer ces fichiers plutôt
   que de pointer sur `devUrl`.

Un binaire compilé sans elle se lance normalement mais n'affiche qu'un
**« localhost a refusé de se connecter »** — une panne silencieuse qui ressemble
à un problème de réseau. Le symptôme est reconnaissable à la taille du binaire :
3,88 Mo sans les assets contre 4,22 Mo avec.

`build.rs` refuse désormais ce cas de figure plutôt que de produire un
exécutable cassé : il détecte l'absence de `TAURI_CLI_VERBOSITY`, variable que
seule la CLI Tauri renseigne, et interrompt la compilation avec un message
indiquant la commande à utiliser.

Pour travailler l'interface sans recompiler Rust, `npm run dev:vite` suffit :
`isDesktop()` renvoie alors `false` et les fonctionnalités dépendant de l'hôte
sont désactivées proprement au lieu de lever.
