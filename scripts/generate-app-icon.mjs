/**
 * Génère l'icône source de l'application : un écu doré sur fond bleu nuit,
 * reprenant les couleurs de l'interface (`--color-gold` #E5A93C sur #0B0F19).
 *
 * Écrit un PNG 1024×1024 que `npx tauri icon` décline ensuite dans tous les
 * formats requis. Le script est versionné pour que l'icône soit reproductible
 * plutôt que d'être un binaire opaque déposé dans le dépôt.
 *
 * Rendu en pur Node — pas de dépendance graphique : le tracé est simple et un
 * suréchantillonnage 4×4 suffit à lisser les bords.
 *
 * Usage : npm run icon
 */
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import { fileURLToPath } from 'node:url';

const SIZE = 1024;
const SUPERSAMPLE = 4;

const NAVY = [11, 15, 25];
const GOLD = [229, 169, 60];
const GOLD_DEEP = [176, 122, 30];

/** Coin arrondi du fond, en proportion du côté — convention des icônes macOS. */
const BACKGROUND_RADIUS = 0.22;

/** L'écu occupe cette fraction de la largeur, centré. */
const SHIELD_SCALE = 0.58;

/** Arrondi du haut de l'écu, en proportion de sa demi-largeur. */
const SHIELD_TOP_RADIUS = 0.42;

/**
 * Exposant du fuselage. En dessous de 1 les flancs bombent vers l'extérieur
 * avant de converger en pointe, ce qui donne sa silhouette à l'écu.
 */
const SHIELD_TAPER = 0.62;

/** Point dans le fond arrondi ? Coordonnées normalisées dans [-1, 1]. */
function insideBackground(x, y) {
  const r = BACKGROUND_RADIUS * 2;
  const ax = Math.abs(x);
  const ay = Math.abs(y);
  const limit = 1 - r;

  if (ax <= limit || ay <= limit) return ax <= 1 && ay <= 1;
  return (ax - limit) ** 2 + (ay - limit) ** 2 <= r ** 2;
}

/** Point dans l'écu ? Coordonnées normalisées, y croissant vers le bas. */
function insideShield(x, y) {
  if (y > 1 || y < -1) return false;

  const ax = Math.abs(x);

  if (y <= 0) {
    const r = SHIELD_TOP_RADIUS;
    const corner = -1 + r;
    // Flancs droits, sauf dans les deux coins hauts qui sont arrondis.
    if (y >= corner || ax <= 1 - r) return ax <= 1;
    return (ax - (1 - r)) ** 2 + (y - corner) ** 2 <= r ** 2;
  }

  return ax <= (1 - y) ** SHIELD_TAPER;
}

function mix(a, b, t) {
  return [
    Math.round(a[0] + (b[0] - a[0]) * t),
    Math.round(a[1] + (b[1] - a[1]) * t),
    Math.round(a[2] + (b[2] - a[2]) * t),
  ];
}

function render() {
  // RGBA, une ligne précédée de son octet de filtre (0 = aucun).
  const raw = Buffer.alloc(SIZE * (SIZE * 4 + 1));
  const step = 1 / SUPERSAMPLE;
  const samples = SUPERSAMPLE * SUPERSAMPLE;

  for (let py = 0; py < SIZE; py += 1) {
    const rowStart = py * (SIZE * 4 + 1);
    raw[rowStart] = 0;

    for (let px = 0; px < SIZE; px += 1) {
      let background = 0;
      let shield = 0;

      for (let sy = 0; sy < SUPERSAMPLE; sy += 1) {
        for (let sx = 0; sx < SUPERSAMPLE; sx += 1) {
          const x = ((px + (sx + 0.5) * step) / SIZE) * 2 - 1;
          const y = ((py + (sy + 0.5) * step) / SIZE) * 2 - 1;

          if (insideBackground(x, y)) background += 1;

          const shieldY = (y + 0.06) / SHIELD_SCALE;
          if (Math.abs(x / SHIELD_SCALE) <= 1.2 && insideShield(x / SHIELD_SCALE, shieldY)) {
            shield += 1;
          }
        }
      }

      const backgroundAlpha = background / samples;
      const shieldAlpha = (shield / samples) * backgroundAlpha;

      // Dégradé vertical sur l'écu : l'or s'assombrit vers la pointe.
      const gold = mix(GOLD, GOLD_DEEP, py / SIZE);
      const color = mix(NAVY, gold, shieldAlpha);

      const offset = rowStart + 1 + px * 4;
      raw[offset] = color[0];
      raw[offset + 1] = color[1];
      raw[offset + 2] = color[2];
      raw[offset + 3] = Math.round(backgroundAlpha * 255);
    }
  }

  return raw;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);

  const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(zlib.crc32(body) >>> 0);

  return Buffer.concat([length, body, crc]);
}

function encodePng(raw) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(SIZE, 0);
  header.writeUInt32BE(SIZE, 4);
  header[8] = 8; // 8 bits par canal
  header[9] = 6; // RGBA
  header[10] = 0; // deflate
  header[11] = 0; // filtrage adaptatif
  header[12] = 0; // pas d'entrelacement

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const target = path.join(root, 'src-tauri', 'app-icon.png');

fs.mkdirSync(path.dirname(target), { recursive: true });
fs.writeFileSync(target, encodePng(render()));

console.log(`icône ${SIZE}×${SIZE} écrite : ${path.relative(root, target)}`);
console.log('décliner avec : npx tauri icon src-tauri/app-icon.png');
console.log("puis retirer src-tauri/icons/android et /ios — l'application ne cible que le bureau");
