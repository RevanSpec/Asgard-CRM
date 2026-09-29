// Parcours de l'application empaquetée, de bout en bout.
//
//     node scripts/smoke.mjs target/debug/asgard-crm.exe
//
// Le script lance l'application, s'attache à sa WebView par le protocole
// DevTools, visite les sept écrans et joue les opérations d'écriture. Il ne
// simule **aucune entrée souris ou clavier** : tout passe par la page, donc
// rien ne peut atterrir dans une autre fenêtre du bureau.
//
// Sur une base vierge — le cas en intégration continue — l'application s'ouvre
// sur l'accueil du premier lancement : le parcours le remplit, après avoir
// vérifié qu'il refuse de s'effacer sans identité. Sur une base déjà en service,
// ces deux étapes n'ont pas lieu d'être et sont passées.
//
// Pourquoi ce script existe : ni le compilateur ni les tests unitaires ne
// voient une interface qui se fige. Un appel de contexte Leptos hors rendu a
// paniqué au clic sur « Nouvelle Facture », et seule l'exécution de
// l'application l'a montré. Toute panique WebAssembly fait donc échouer ce
// parcours.
//
// ⚠️ Il écrit dans la base de l'utilisateur courant. En intégration continue
// la base est vierge ; en local, sauvegardez `%APPDATA%/com.asgard.crm`
// avant de le lancer.

import { spawn, execSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const EXE = process.argv[2] ?? 'target/release/asgard-crm.exe';
const PORT = process.argv[3] ?? '9333';
const SHOTS = resolve('target/smoke');

/** Messages de console connus, qui ne font pas échouer le parcours. */
const TOLERATED = [
  // Les polices Google sont bloquées par la CSP : l'application retombe sur
  // les polices du système. C'était déjà le cas avec l'interface React.
  'fonts.googleapis.com',
];

const wait = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------------------------------------------------------------- lancement

mkdirSync(SHOTS, { recursive: true });

const app = spawn(resolve(EXE), {
  env: {
    ...process.env,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT} --disable-features=CalculateNativeWinOcclusion`,
  },
  stdio: ['ignore', 'pipe', 'pipe'],
});

let appOutput = '';
let stopping = false;
app.stdout.on('data', (d) => (appOutput += d));
app.stderr.on('data', (d) => (appOutput += d));
app.on('exit', (code) => {
  // Après `stop()`, la sortie est voulue : c'est nous qui l'avons arrêtée.
  if (stopping || code === 0 || code === null) return;
  console.error(`L'application s'est arrêtée d'elle-même (code ${code}) :\n${appOutput}`);
  process.exit(1);
});

function stop() {
  stopping = true;
  try {
    execSync(`taskkill /pid ${app.pid} /t /f`, { stdio: 'ignore' });
  } catch {
    app.kill('SIGKILL');
  }
}

/// Ce qu'on peut dire de la machine quand la WebView ne répond pas.
///
/// Sans cela, l'échec se résume à « pas de page de débogage », ce qui
/// n'oriente vers rien : runtime absent, fenêtre jamais créée, port filtré,
/// tout se ressemble.
function diagnose() {
  const report = [];
  const run = (label, command) => {
    try {
      report.push(`${label} :\n${execSync(command, { encoding: 'utf8', stdio: 'pipe' }).trim()}`);
    } catch (error) {
      report.push(`${label} : indisponible (${String(error.message).split(String.fromCharCode(10))[0]})`);
    }
  };

  report.push(`processus lancé : ${app.pid}, encore vivant : ${app.exitCode === null}`);
  run(
    'version du runtime WebView2',
    'reg query "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" /v pv',
  );
  run('processus de WebView', 'tasklist /fi "imagename eq msedgewebview2.exe"');
  // La ligne de commande dit si l'argument de débogage a été transmis : c'est
  // la différence entre « la WebView l'ignore » et « on ne le lui a pas donné ».
  run(
    'arguments de la WebView',
    `wmic process where "name='msedgewebview2.exe'" get commandline /format:list`,
  );
  run('ports en écoute', `netstat -ano | findstr LISTENING | findstr ${PORT}`);
  return report.join(String.fromCharCode(10).repeat(2));
}

/// Le point de débogage n'écoute pas toujours sur la même pile : selon la
/// machine, il se lie à IPv4, à IPv6, ou aux deux.
const HOSTS = ['127.0.0.1', '[::1]', 'localhost'];

async function findPage() {
  // Un runner froid met plus de temps qu'un poste de travail : la WebView
  // s'initialise à son premier lancement.
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    for (const host of HOSTS) {
      try {
        const targets = await (await fetch(`http://${host}:${PORT}/json`)).json();
        const page = targets.find((t) => t.type === 'page' && t.url.includes('tauri'));
        // L'adresse du WebSocket vient de la cible elle-même : pas besoin de
        // retenir la pile qui a répondu.
        if (page) return page;
      } catch {
        // Cette pile-là ne répond pas (encore).
      }
    }
    await wait(500);
  }

  throw new Error(
    `aucune page de débogage sur le port ${PORT}.\n\nSortie de l'application :\n${appOutput}\n\n${diagnose()}`,
  );
}

const page = await findPage();
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r, j) => {
  ws.addEventListener('open', r, { once: true });
  ws.addEventListener('error', j, { once: true });
});

// ------------------------------------------------------------- protocole

let nextId = 1;
const pending = new Map();
const noticed = [];

ws.addEventListener('message', (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) {
    pending.get(message.id)(message);
    pending.delete(message.id);
    return;
  }
  const text =
    message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error'
      ? message.params.args.map((a) => a.value ?? a.description ?? '').join(' ')
      : message.method === 'Runtime.exceptionThrown'
        ? message.params.exceptionDetails.exception?.description ?? message.params.exceptionDetails.text
        : message.method === 'Log.entryAdded' && message.params.entry.level === 'error'
          ? message.params.entry.text
          : null;
  if (text && !TOLERATED.some((known) => text.includes(known))) noticed.push(text.slice(0, 400));
});

function send(method, params = {}) {
  const id = nextId++;
  ws.send(JSON.stringify({ id, method, params }));
  return new Promise((res, rej) =>
    pending.set(id, (m) => (m.error ? rej(new Error(`${method} : ${m.error.message}`)) : res(m.result))),
  );
}

async function ev(expression) {
  const { result, exceptionDetails } = await send('Runtime.evaluate', {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (exceptionDetails) {
    const detail = exceptionDetails.exception?.description ?? exceptionDetails.text;
    throw new Error(String(detail).split(String.fromCharCode(10))[0]);
  }
  return result.value;
}

/** Attend qu'une condition de la page devienne vraie. */
async function until(expression, what, timeout = 8000) {
  const deadline = Date.now() + timeout;
  let last;
  while (Date.now() < deadline) {
    last = await ev(expression);
    if (last) return true;
    await wait(200);
  }
  throw new Error(`attente dépassée : ${what}`);
}

async function shot(name) {
  const { data } = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(`${SHOTS}/${name}.png`, Buffer.from(data, 'base64'));
}

// Outils injectés dans la page. Leptos écoute `input` et `change` : une valeur
// posée sans événement ne remonterait pas jusqu'au signal.
const HELPERS = `
  window.__t = {
    click(selector, text) {
      const nodes = [...document.querySelectorAll(selector)];
      const node = text ? nodes.find((n) => n.textContent.trim().includes(text)) : nodes[0];
      if (!node) throw new Error('introuvable : ' + selector + ' « ' + (text ?? '') + ' »');
      node.click();
      return true;
    },
    clickInRow(needle, label) {
      const row = [...document.querySelectorAll('tbody tr')].find((r) => r.innerText.includes(needle));
      if (!row) throw new Error('ligne introuvable : ' + needle);
      const button = [...row.querySelectorAll('button')].find((b) => (b.textContent + (b.title ?? '')).includes(label));
      if (!button) throw new Error('bouton « ' + label + ' » absent de la ligne ' + needle);
      button.click();
      return true;
    },
    fill(label, value) {
      const group = [...document.querySelectorAll('.form-group')]
        .find((g) => g.querySelector('.form-label')?.textContent.trim() === label);
      if (!group) throw new Error('champ introuvable : ' + label);
      const field = group.querySelector('input, textarea, select');
      const proto = field.tagName === 'SELECT' ? HTMLSelectElement
        : field.tagName === 'TEXTAREA' ? HTMLTextAreaElement : HTMLInputElement;
      Object.getOwnPropertyDescriptor(proto.prototype, 'value').set.call(field, value);
      field.dispatchEvent(new Event(field.tagName === 'SELECT' ? 'change' : 'input', { bubbles: true }));
      return field.value;
    },
    value(label) {
      const group = [...document.querySelectorAll('.form-group')]
        .find((g) => g.querySelector('.form-label')?.textContent.trim() === label);
      if (!group) throw new Error('champ introuvable : ' + label);
      return group.querySelector('input, textarea, select').value;
    },
    optionContaining(label, needle) {
      const group = [...document.querySelectorAll('.form-group')]
        .find((g) => g.querySelector('.form-label')?.textContent.trim() === label);
      const option = [...group.querySelector('select').options].find((o) => o.textContent.includes(needle));
      return option ? option.value : null;
    },
    heading() { return document.querySelector('.main-content h1')?.textContent ?? ''; },
    modal() { const m = document.querySelector('.modal-content'); return m ? m.innerText : null; },
    rows() { return [...document.querySelectorAll('.main-content tbody tr')].map((r) => r.innerText.split(String.fromCharCode(9)).join(' | ')); },
    errors() { return [...document.querySelectorAll('.modal-content .error-text')].map((e) => e.textContent); },
    // Les montants portent une espace fine insécable entre les milliers et une
    // espace insécable avant « € » : on les ramène à des espaces ordinaires,
    // sans quoi la moindre comparaison de texte échouerait pour rien.
    flat(text) {
      return (text ?? '')
        .split(String.fromCharCode(10)).join(' ')
        .split(String.fromCharCode(0x202f)).join(' ')
        .split(String.fromCharCode(0xa0)).join(' ')
        .trim();
    },
  };
  'prêt'
`;

// ------------------------------------------------------------------ étapes

const results = [];

/** Referme ce qui serait resté ouvert : un échec ne doit pas en entraîner d'autres. */
async function closeAnyModal() {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    const open = await ev('__t.modal() !== null').catch(() => false);
    if (!open) return;
    await ev(`
      (() => {
        const footer = document.querySelector('.modal-footer');
        const escape = [...(footer?.querySelectorAll('button') ?? [])]
          .find((b) => ['OK', 'Annuler'].includes(b.textContent.trim()));
        (escape ?? document.querySelector('.modal-header button')).click();
        return true;
      })()
    `).catch(() => false);
    await wait(300);
  }
}

async function step(name, body) {
  try {
    await closeAnyModal();
    const detail = await body();
    results.push({ name, ok: true, detail });
    const shown = typeof detail === 'object' && detail !== null ? JSON.stringify(detail) : String(detail ?? '');
    console.log(`  ok   ${name}${shown ? ' — ' + shown.slice(0, 110) : ''}`);
  } catch (error) {
    results.push({ name, ok: false, detail: String(error.message ?? error) });
    console.error(`  ÉCHEC ${name} — ${error.message ?? error}`);
    try {
      await shot(`echec-${results.length}`);
    } catch {
      // Une page morte ne se capture pas : l'échec suffit.
    }
  }
}

const goto = async (tab) => {
  await ev(`__t.click('.nav-item', ${JSON.stringify(tab)})`);
  await wait(700);
  await ev(HELPERS);
};

await send('Page.enable');
await send('Runtime.enable');
await send('Log.enable');

// Deux ouvertures possibles. Une base vierge — le cas en intégration continue —
// s'ouvre sur l'accueil du premier lancement ; une base déjà en service s'ouvre
// directement sur l'application.
//
// L'attente porte donc sur les deux, **et** sur la fin du chargement : tant que
// l'hôte n'a pas répondu, ni l'un ni l'autre n'est affiché, et trancher avant
// reviendrait à jouer à pile ou face — un runner froid répondant plus lentement
// qu'un poste de travail.
const WELCOME = 'Bienvenue dans Asgard CRM';
await until(
  `document.querySelector('.sidebar') !== null || document.body.innerText.includes(${JSON.stringify(WELCOME)})`,
  'démarrage de l’interface',
  30_000,
);
await ev(HELPERS);

const welcoming = await ev(`document.body.innerText.includes(${JSON.stringify(WELCOME)})`);

console.log('Parcours :');

const IDENTITY = {
  "Nom de l'entreprise": 'Forge du Valhalla',
  'Adresse professionnelle': '1 rue du Bifrost, 75011 Paris',
  SIRET: '839 204 123 00019',
  'IBAN bancaire (Règlement)': 'FR76 3000 2000 0001 2345 6789 012',
};

if (welcoming) {
  // Le défaut D11 tenait à ce que rien n'exigeait cette identité : les réglages
  // en livraient une, fictive et crédible, et la facture sortait au nom d'une
  // société inventée. L'accueil doit donc refuser de s'effacer tant qu'elle
  // manque.
  await step("l'accueil refuse de commencer sans identité", async () => {
    for (const label of Object.keys(IDENTITY)) await ev(`__t.fill(${JSON.stringify(label)}, '')`);
    await ev(`__t.click('button', 'Commencer')`);
    await until(`document.querySelector('.error-text') !== null`, 'refus affiché', 3000);
    const refusal = await ev(`__t.flat(document.querySelector('.error-text').textContent)`);
    if (!refusal.includes('le SIRET')) throw new Error(`refus peu explicite : ${refusal}`);
    if (await ev(`document.querySelector('.sidebar') !== null`)) {
      throw new Error("l'application s'est ouverte malgré une identité absente");
    }
    return refusal;
  });

  await step("remplir l'accueil, avec le jeu d'exemple", async () => {
    for (const [label, value] of Object.entries(IDENTITY)) {
      await ev(`__t.fill(${JSON.stringify(label)}, ${JSON.stringify(value)})`);
    }
    await shot('accueil');
    await ev(`__t.click('button', 'exemple')`);
    await until(`document.querySelector('.sidebar') !== null`, "ouverture de l'application", 20_000);
    await ev(HELPERS);
    // Le jeu d'exemple est semé par l'hôte pendant que l'écran bascule : les
    // étapes suivantes en dépendent, donc on attend de le voir plutôt que de
    // supposer qu'il est arrivé.
    await goto('Clients');
    await until(`__t.rows().length >= 3`, "jeu d'exemple chargé", 15_000);
    return `${IDENTITY.SIRET}, ${await ev('__t.rows().length')} clients d'exemple`;
  });
}

await until(`!document.body.innerText.includes('Chargement')`, 'chargement des données', 20_000);

const SCREENS = [
  ['Dashboard', 'Tableau de bord'],
  ['Clients', 'Fichiers Clients'],
  ['Devis', 'Gestion des Devis'],
  ['Factures', 'Factures'],
  ['Dépenses', 'Registre des Dépenses'],
  ['Comptabilité', 'Comptabilité & Déclarations'],
  ['Paramètres', "Paramètres de l'entreprise"],
];

for (const [tab, heading] of SCREENS) {
  await step(`écran ${tab}`, async () => {
    await goto(tab);
    const seen = await ev('__t.heading()');
    if (seen !== heading) throw new Error(`titre « ${seen} » au lieu de « ${heading} »`);
    await shot(`ecran-${tab.normalize('NFD').replace(/[^a-zA-Z]/g, '')}`);
    return seen;
  });
}

const CLIENT = 'Gjallarhorn SARL';

await step('créer un client', async () => {
  await goto('Clients');
  await ev(`__t.click('.page-header .btn-primary', 'Nouveau Client')`);
  await until(`__t.modal()?.includes('Ajouter un client')`, 'fenêtre de création');
  await ev(`__t.fill("Nom de l'entreprise", ${JSON.stringify(CLIENT)})`);
  await ev(`__t.fill('Nom du contact', 'Heimdall Gardien')`);
  await ev(`__t.fill('Email', 'heimdall@gjallarhorn.as')`);
  await ev(`__t.fill('Numéro de téléphone', '0611aa22b33.44')`);
  await ev(`__t.fill("Adresse de l'entreprise", '1 rue du Pont')`);
  await ev(`__t.click('.modal-footer .btn-primary', 'Ajouter')`);
  await until(`__t.modal() === null`, 'fermeture de la fenêtre');
  await until(`__t.rows().some(r => r.includes(${JSON.stringify(CLIENT)}))`, 'client ajouté au tableau');
  const row = (await ev('__t.rows()')).find((r) => r.includes(CLIENT));
  // Le téléphone est mis en forme à la frappe, comme dans la version d'origine.
  if (!row.includes('06 11 22 33 44')) throw new Error(`téléphone non formaté : ${row}`);
  return row;
});

await step('refuser une saisie invalide', async () => {
  await ev(`__t.click('.page-header .btn-primary', 'Nouveau Client')`);
  await until(`__t.modal()?.includes('Ajouter un client')`, 'fenêtre de création');
  await ev(`__t.fill("Nom de l'entreprise", 'Test')`);
  await ev(`__t.fill('Nom du contact', 'Test')`);
  await ev(`__t.fill('Email', 'pas-un-email')`);
  await ev(`__t.fill('Numéro de téléphone', '0611223344')`);
  await ev(`__t.fill("Adresse de l'entreprise", '2 rue de Paris')`);
  await ev(`__t.click('.modal-footer .btn-primary', 'Ajouter')`);
  await wait(500);
  const errors = await ev('__t.errors()');
  const open = await ev('__t.modal() !== null');
  await ev(`__t.click('.modal-footer .btn-secondary', 'Annuler')`);
  await until(`__t.modal() === null`, 'fermeture');
  if (!open) throw new Error('la fenêtre s’est fermée malgré une saisie invalide');
  if (!errors.some((e) => e.includes('Format email invalide'))) throw new Error(`erreurs : ${JSON.stringify(errors)}`);
  return errors.join(' / ');
});

await step('modifier un client', async () => {
  await ev(`__t.clickInRow(${JSON.stringify(CLIENT)}, 'Modifier')`);
  await until(`__t.modal()?.includes('Modifier le client')`, 'fenêtre de modification');
  await ev(`__t.fill('Nom du contact', 'Heimdall Veilleur')`);
  await ev(`__t.click('.modal-footer .btn-primary', 'Modifier')`);
  await until(`__t.modal() === null`, 'fermeture');
  // Une ligne dont seul le contenu change doit être redessinée.
  await until(`__t.rows().some(r => r.includes('Heimdall Veilleur'))`, 'ligne redessinée');
  return 'ligne à jour';
});

await step('facturer un montant à virgule décimale', async () => {
  await goto('Factures');
  const before = (await ev('__t.rows()')).length;
  await ev(`__t.click('.page-header .btn-primary', 'Nouvelle Facture')`);
  await until(`__t.modal()?.includes('Générer une facture')`, 'fenêtre de facturation');
  const client = await ev(`__t.optionContaining('Client facturé', ${JSON.stringify(CLIENT)})`);
  if (!client) throw new Error('client absent de la liste déroulante');
  await ev(`__t.fill('Client facturé', ${JSON.stringify(client)})`);
  await ev(`__t.fill('Montant Hors Taxes (HT) en €', '1899,99')`);
  await ev(`__t.fill('Description de la prestation', 'Veille du pont')`);
  await wait(400);
  const preview = await ev(`__t.flat(__t.modal().split('Aperçu du calcul :')[1] ?? '')`);
  // 1899,99 € à 20 % : la virgule décimale doit être lue, pas tronquée.
  if (!preview.includes('380,00') || !preview.includes('2 279,99')) {
    throw new Error(`aperçu inattendu : ${preview.slice(0, 120)}`);
  }
  await ev(`__t.click('.modal-footer .btn-primary', 'Générer')`);
  await until(`__t.modal() === null`, 'fermeture');
  await until(`__t.rows().length === ${before + 1}`, 'facture ajoutée');
  return preview.slice(0, 80);
});

await step('encaisser une facture', async () => {
  await ev(`__t.clickInRow(${JSON.stringify(CLIENT)}, 'Régler')`);
  await until(`__t.modal()?.includes('Enregistrer le règlement')`, 'fenêtre de règlement');
  await ev(`__t.click('.modal-footer .btn-primary', 'Valider le paiement')`);
  await until(`__t.modal()?.includes('marquée comme payée')`, 'confirmation');
  await ev(`__t.click('.modal-footer .btn-primary', 'OK')`);
  await until(`__t.modal() === null`, 'fermeture');
  await until(`__t.rows().find(r => r.includes(${JSON.stringify(CLIENT)})).includes('Payée')`, 'statut Payée');
  return 'facture payée';
});

await step('émettre un avoir sur la facture', async () => {
  // L'avoir n'est possible que sur une facture émise : le bouton n'apparaît
  // pas sur un brouillon.
  await ev(`__t.clickInRow(${JSON.stringify(CLIENT)}, 'Avoir')`);
  await until(`__t.modal()?.includes('Émettre un avoir')`, "fenêtre d'avoir");
  const prefilled = await ev(`document.querySelector('.modal-content input').value`);
  await ev(`__t.fill("Montant HT de l'avoir (€)", '400,50')`);
  await ev(`__t.fill('Motif', 'Prestation partiellement annulée')`);
  await ev(`__t.click('.modal-footer .btn-primary', "Émettre l'avoir")`);
  await until(`__t.modal()?.includes('Avoir émis')`, 'confirmation');
  const notice = await ev(`__t.flat(__t.modal())`);
  await ev(`__t.click('.modal-footer .btn-primary', 'OK')`);
  await until(`__t.flat(document.body.innerText).includes('Avoirs émis')`, 'liste des avoirs');
  const line = (await ev('__t.rows()')).find((r) => r.includes('AVO-'));
  if (!line) throw new Error('avoir absent du tableau');
  if (!line.includes('400,50') && !line.includes('480,60')) {
    throw new Error(`montant inattendu sur la ligne : ${line}`);
  }
  return { montantPréRempli: prefilled, message: notice.slice(0, 90), ligne: line };
});

await step('préparer un envoi par e-mail', async () => {
  await ev(`__t.clickInRow('FAC-', 'Envoyer par e-mail')`);
  await until(`__t.modal()?.includes('Envoyer la facture par e-mail')`, "fenêtre d'envoi");
  const to = await ev(`document.querySelector('.modal-content input[type=email]').value`);
  const body = await ev(`__t.flat(document.querySelector('.modal-content textarea').value)`);
  // Rien n'est envoyé : la fenêtre est refermée.
  await ev(`__t.click('.modal-footer .btn-secondary', 'Annuler')`);
  await until(`__t.modal() === null`, 'fermeture');
  if (!to.includes('@')) throw new Error(`destinataire non pré-rempli : « ${to} »`);
  if (!body.includes('Bonjour')) throw new Error(`message non pré-rempli : ${body.slice(0, 80)}`);
  return to;
});

await step('convertir un devis en facture', async () => {
  await goto('Devis');
  await ev(`__t.clickInRow('Facturer', 'Facturer')`);
  await until(`__t.modal()?.includes('Convertir en facture')`, 'confirmation');
  await ev(`__t.click('.modal-footer .btn-danger', 'Confirmer')`);
  await until(`__t.modal()?.includes('Conversion réussie')`, 'message de conversion');
  const notice = await ev(`__t.flat(__t.modal())`);
  await ev(`__t.click('.modal-footer .btn-primary', 'OK')`);
  await until(`__t.modal() === null`, 'fermeture');
  if ((await ev('__t.heading()')) !== 'Factures') throw new Error('la conversion ne bascule pas sur les factures');
  return notice.slice(0, 90);
});

await step('enregistrer puis supprimer une dépense', async () => {
  await goto('Dépenses');
  await ev(`__t.click('.page-header .btn-primary', 'Nouvelle Dépense')`);
  await until(`__t.modal()?.includes('Enregistrer une dépense')`, 'fenêtre de dépense');
  await ev(`__t.fill('Fournisseur', 'Forge de Nidavellir')`);
  await ev(`__t.fill('Montant (€)', '49,90')`);
  await ev(`__t.click('.modal-footer .btn-primary', 'Enregistrer')`);
  await until(`__t.modal()?.includes('Dépense enregistrée')`, 'confirmation');
  await ev(`__t.click('.modal-footer .btn-primary', 'OK')`);
  await until(`__t.rows().some(r => r.includes('Nidavellir'))`, 'dépense ajoutée');
  await ev(`__t.clickInRow('Nidavellir', 'Supprimer')`);
  await until(`__t.modal()?.includes('Supprimer la dépense')`, 'confirmation de suppression');
  await ev(`__t.click('.modal-footer .btn-danger', 'Confirmer')`);
  await until(`!__t.rows().some(r => r.includes('Nidavellir'))`, 'dépense retirée');
  return 'créée puis supprimée';
});

await step('supprimer un client, après confirmation', async () => {
  await goto('Clients');
  await ev(`__t.clickInRow(${JSON.stringify(CLIENT)}, 'Supprimer')`);
  await until(`__t.modal()?.includes('Supprimer le client')`, 'confirmation');
  await ev(`__t.click('.modal-footer .btn-danger', 'Confirmer')`);
  await until(`!__t.rows().some(r => r.includes(${JSON.stringify(CLIENT)}))`, 'client retiré');
  return 'client supprimé';
});

await step('les cotisations suivent le réglage ACRE', async () => {
  const charges = async () => {
    await goto('Dashboard');
    return ev(`__t.flat(document.body.innerText.split('CHARGES URSSAF')[1]).split(' ').slice(0, 3).join(' ')`);
  };
  const full = await charges();
  await goto('Paramètres');
  await ev(`document.querySelector('#acreCheckbox').click()`);
  await wait(1200);
  const reduced = await charges();
  await goto('Paramètres');
  await ev(`document.querySelector('#acreCheckbox').click()`);
  await wait(1200);
  const restored = await charges();
  if (full === reduced) throw new Error(`les cotisations n'ont pas bougé : ${full}`);
  if (full !== restored) throw new Error(`valeur non rétablie : ${full} puis ${restored}`);
  return `${full} → ${reduced} → ${full}`;
});

// L'autre moitié du garde : l'hôte refuse d'éditer une pièce dont l'émetteur est
// incomplet. Le sélecteur de fichier n'apparaît jamais — le refus est rendu
// avant lui — donc ce parcours peut l'exercer sans rester bloqué sur une
// fenêtre du système.
await step("aucune facture n'est éditée sans SIRET", async () => {
  await goto('Paramètres');
  const kept = await ev(`__t.value('SIRET')`);
  await ev(`__t.fill('SIRET', '')`);
  await wait(600);

  await goto('Factures');
  await ev(`__t.clickInRow('FAC-', 'Télécharger le PDF')`);
  await until(`__t.modal()?.includes('le SIRET')`, 'refus faute de SIRET');
  const refusal = await ev(`__t.flat(__t.modal())`);
  await ev(`__t.click('.modal-footer .btn-primary', 'OK')`);
  await until(`__t.modal() === null`, 'fermeture');

  await goto('Paramètres');
  await ev(`__t.fill('SIRET', ${JSON.stringify(kept)})`);
  await wait(600);
  if ((await ev(`__t.value('SIRET')`)) !== kept) throw new Error('SIRET non rétabli');
  if (!refusal.includes('Paramètres')) throw new Error(`refus sans indication : ${refusal}`);
  return refusal.slice(0, 120);
});

// ------------------------------------------------------------------ verdict

if (noticed.length > 0) {
  results.push({ name: 'aucune erreur en console', ok: false, detail: noticed.join(' ⏎ ') });
  console.error(`  ÉCHEC aucune erreur en console — ${noticed.length} message(s) :`);
  for (const message of noticed) console.error(`        ${message}`);
} else {
  results.push({ name: 'aucune erreur en console', ok: true });
  console.log('  ok   aucune erreur en console');
}

writeFileSync(`${SHOTS}/rapport.json`, JSON.stringify(results, null, 2));
ws.close();
stop();

const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} étapes réussies.`);
if (failed.length > 0) {
  console.error(`Échecs : ${failed.map((f) => f.name).join(', ')}`);
  process.exit(1);
}
