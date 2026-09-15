/**
 * Archive les PDF produits par jsPDF sur le jeu de référence.
 *
 * Ces fichiers sont la cible visuelle de la phase 4 : la réimplémentation Rust
 * (`printpdf` ou Typst) devra produire des documents équivalents. Les métriques
 * différeront — jsPDF s'appuie sur les polices PDF standard, Rust devra
 * embarquer une fonte — d'où la nécessité d'une référence figée pour comparer
 * page à page plutôt que de discuter de souvenirs.
 *
 * Usage : npm run pdf:reference
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { generateInvoicePDF, generateEstimatePDF } from '../src/pdfGenerator.js';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, '..');
const outDir = path.join(root, 'fixtures', 'pdf-reference');

const dataset = JSON.parse(
  fs.readFileSync(path.join(root, 'fixtures', 'reference-dataset.json'), 'utf8'),
);

const { clients, invoices, estimates, settings } = dataset;
const clientById = new Map(clients.map((client) => [client.id, client]));

/**
 * Date d'édition figée : le pied de page imprime la date du jour. Sans point
 * fixe, la référence changerait à chaque exécution et ne servirait plus de base
 * de comparaison pour la phase 4.
 */
const GENERATED_AT = new Date('2026-09-15T12:00:00Z');

fs.mkdirSync(outDir, { recursive: true });

/** Les deux jeux de réglages changent la couleur d'accent et les mentions. */
const variants = [
  { suffix: '', businessSettings: settings.standard },
  { suffix: '-acre', businessSettings: settings.acre },
];

let written = 0;

/**
 * jsPDF tire un identifiant de document aléatoire, inscrit dans le trailer sous
 * `/ID`. C'est la dernière source de non-déterminisme une fois la date figée :
 * sans normalisation, régénérer la référence salit le dépôt à chaque exécution
 * sans qu'aucun contenu n'ait changé.
 */
const STABLE_ID = '00000000000000000000000000000000';

function normalizeDocumentId(buffer) {
  const text = buffer.toString('latin1');
  const normalized = text.replace(
    /\/ID\s*\[\s*<[0-9A-F]{32}>\s*<[0-9A-F]{32}>\s*\]/i,
    `/ID [ <${STABLE_ID}> <${STABLE_ID}> ]`,
  );
  return Buffer.from(normalized, 'latin1');
}

function write(doc, name) {
  // jsPDF horodate le PDF à la seconde près dans ses métadonnées. Sans la figer,
  // deux exécutions successives produisent des octets différents.
  doc.setCreationDate(GENERATED_AT);

  const buffer = normalizeDocumentId(Buffer.from(doc.output('arraybuffer')));
  fs.writeFileSync(path.join(outDir, name), buffer);
  written += 1;
  console.log(`  ${name}  ${(buffer.length / 1024).toFixed(1)} Ko`);
}

console.log('Factures :');
invoices.forEach((invoice) => {
  const client = clientById.get(invoice.clientId);
  variants.forEach(({ suffix, businessSettings }) => {
    write(
      generateInvoicePDF(invoice, client, businessSettings, GENERATED_AT),
      `${invoice.invoiceNumber}${suffix}.pdf`,
    );
  });
});

console.log('Devis :');
estimates.forEach((estimate) => {
  const client = clientById.get(estimate.clientId);
  variants.forEach(({ suffix, businessSettings }) => {
    write(
      generateEstimatePDF(estimate, client, businessSettings, GENERATED_AT),
      `${estimate.estimateNumber}${suffix}.pdf`,
    );
  });
});

console.log(`\n${written} PDF de référence écrits dans fixtures/pdf-reference/`);
