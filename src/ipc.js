/**
 * Frontière entre l'interface et le processus hôte.
 *
 * Point de changement unique prévu par le plan de migration : les phases
 * suivantes déplacent la base de données, les calculs puis la génération de PDF
 * vers Rust, et chaque déplacement ajoute des fonctions ici sans que le JSX ait
 * à savoir d'où viennent les données.
 *
 * Remplace `window.require('electron').ipcRenderer`, qui n'existait que parce
 * que la fenêtre Electron tournait avec `nodeIntegration: true` — c'est-à-dire
 * avec tout Node accessible au contenu rendu (défaut D1). Ici, seules les
 * commandes déclarées dans `src-tauri/src/lib.rs` sont atteignables.
 */
import { invoke } from '@tauri-apps/api/core';

/**
 * L'application tourne-t-elle dans sa coquille de bureau ?
 *
 * Faux quand le front est servi seul par Vite (`npm run dev:vite`), ce qui reste
 * pratique pour travailler l'interface sans recompiler Rust. Les fonctionnalités
 * qui dépendent de l'hôte sont alors désactivées plutôt que de lever.
 */
export function isDesktop() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/** Message affiché lorsqu'une fonctionnalité de bureau est appelée hors coquille. */
const WEB_ONLY = "Cette fonctionnalité n'est disponible que dans la version de bureau de l'application.";

/**
 * Les commandes d'envoi renvoient toujours `{ success, error? }` plutôt que de
 * lever : c'est la forme que le JSX attendait déjà des handlers Electron, et
 * elle évite d'avoir un try/catch autour de chaque appel dans les vues.
 */
function unavailable() {
  return { success: false, error: WEB_ONLY };
}

/**
 * Envoie un document par e-mail.
 *
 * `smtpConfig` ne porte plus de mot de passe : l'hôte le lit dans le trousseau
 * de l'OS au moment de l'envoi (défaut D2). Le secret ne traverse plus la
 * frontière.
 */
export async function sendEmail(smtpConfig, emailData) {
  if (!isDesktop()) return unavailable();
  return invoke('send_email', { smtpConfig, emailData });
}

/** Vérifie une configuration SMTP sans envoyer de message. */
export async function testSmtp(smtpConfig) {
  if (!isDesktop()) return unavailable();
  return invoke('test_smtp', { smtpConfig });
}

/**
 * Enregistre le mot de passe SMTP dans le trousseau de l'OS.
 * Une chaîne vide efface l'entrée.
 */
export async function setSmtpPassword(password) {
  if (!isDesktop()) return;
  await invoke('set_smtp_password', { password });
}

/**
 * Un mot de passe est-il enregistré ?
 *
 * L'interface ne peut pas relire le secret — elle affiche seulement si un
 * mot de passe existe, pour distinguer « aucun » de « déjà configuré ».
 */
export async function hasSmtpPassword() {
  if (!isDesktop()) return false;
  return invoke('has_smtp_password');
}

export async function clearSmtpPassword() {
  if (!isDesktop()) return;
  await invoke('clear_smtp_password');
}
