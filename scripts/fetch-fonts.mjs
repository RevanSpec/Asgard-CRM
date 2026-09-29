// Récupère Inter et Outfit en local, sous-ensembles latins seulement.
//
//     node scripts/fetch-fonts.mjs
//
// `style/index.css` importait ces deux polices depuis Google Fonts. La requête
// n'aboutissait jamais : la politique de sécurité de contenu n'autorise que
// `font-src 'self' data:`, si bien que l'application s'affichait avec les
// polices du système — pas celles pour lesquelles le thème a été dessiné.
//
// Les embarquer est de toute façon la bonne réponse pour une application
// locale : plus aucune requête réseau au démarrage, et le rendu ne dépend plus
// d'un service tiers.
//
// Ce script n'a pas à tourner à chaque compilation. Il sert à produire — ou à
// rafraîchir — le contenu de `assets/fonts/` et `style/fonts.css`, qui sont
// versionnés. Les fichiers récupérés sont ceux que Google Fonts sert à un
// navigateur moderne : des polices variables woff2, déjà découpées par plage
// Unicode. Seuls `latin` et `latin-ext` sont conservés — le cyrillique, le grec
// et le vietnamien ne servent à personne ici.
//
// Inter et Outfit sont publiées sous SIL Open Font License 1.1, dont le texte
// accompagne les fichiers (`assets/fonts/OFL-*.txt`), comme la licence l'exige.

import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const CSS =
  'https://fonts.googleapis.com/css2?family=Inter:wght@300..700&family=Outfit:wght@400..800&display=swap';

// Sans en-tête de navigateur, l'API renvoie du TTF au lieu du woff2.
const UA =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36';

const WANTED = new Set(['latin', 'latin-ext']);

const FONTS = resolve('crates/asgard-ui/assets/fonts');
const STYLE = resolve('crates/asgard-ui/style/fonts.css');

/** Textes de licence, exigés par l'OFL pour toute redistribution. */
const LICENCES = {
  'OFL-Inter.txt': 'https://raw.githubusercontent.com/rsms/inter/master/LICENSE.txt',
  'OFL-Outfit.txt': 'https://raw.githubusercontent.com/Outfitio/Outfit-Fonts/main/OFL.txt',
};

async function get(url) {
  const response = await fetch(url, { headers: { 'User-Agent': UA } });
  if (!response.ok) throw new Error(`${response.status} sur ${url}`);
  return response;
}

await mkdir(FONTS, { recursive: true });

const css = await (await get(CSS)).text();

// Chaque bloc est précédé d'un commentaire nommant le sous-ensemble.
const blocks = [...css.matchAll(/\/\* ([\w-]+) \*\/\s*(@font-face \{[^}]*\})/g)];
const rules = [];

for (const [, subset, block] of blocks) {
  if (!WANTED.has(subset)) continue;

  const field = (name) => block.match(new RegExp(`${name}: ([^;]+);`))[1];
  const family = field('font-family').replace(/'/g, '');
  const url = block.match(/url\(([^)]+)\)/)[1];
  const name = `${family.toLowerCase()}-${subset}.woff2`;

  const bytes = Buffer.from(await (await get(url)).arrayBuffer());
  await writeFile(resolve(FONTS, name), bytes);
  console.log(`${name.padEnd(24)} ${(bytes.length / 1024).toFixed(1).padStart(6)} ko`);

  rules.push(
    [
      '@font-face {',
      `  font-family: '${family}';`,
      '  font-style: normal;',
      `  font-weight: ${field('font-weight')};`,
      '  font-display: swap;',
      `  src: url('fonts/${name}') format('woff2');`,
      `  unicode-range: ${field('unicode-range')};`,
      '}',
    ].join('\n'),
  );
}

if (rules.length === 0) throw new Error('aucun sous-ensemble latin dans la réponse');

for (const [name, url] of Object.entries(LICENCES)) {
  await writeFile(resolve(FONTS, name), await (await get(url)).text());
  console.log(`${name.padEnd(24)} licence`);
}

const header = `/* Polices de l'application, servies depuis le disque.

   Produit par \`scripts/fetch-fonts.mjs\`, qui reprend les règles telles que
   Google Fonts les sert à un navigateur moderne — mêmes plages Unicode, mêmes
   graisses variables. Ne pas modifier à la main : relancer le script.

   Les fichiers vivent dans \`assets/fonts/\`, que trunk copie dans \`dist/fonts/\`
   (voir \`index.html\`). Le chemin des \`url()\` est donc relatif à la racine du
   \`dist\`, pas à ce fichier. */

`;

await writeFile(STYLE, header + rules.join('\n\n') + '\n');
console.log(`\n${rules.length} règles écrites dans ${STYLE}`);
