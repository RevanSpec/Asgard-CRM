//! Asgard CRM — processus hôte Tauri.
//!
//! Remplace `main.js` (Electron). Deux différences structurelles :
//!
//! 1. Le webview n'a aucun accès à Node. L'ancienne fenêtre était créée avec
//!    `nodeIntegration: true` et `contextIsolation: false`, ce qui exposait
//!    `require('fs')` et `require('child_process')` à du contenu rendu — dont
//!    des logos et des sauvegardes importées (défaut D1). Ici, l'interface ne
//!    peut appeler que les commandes déclarées ci-dessous.
//!
//! 2. Le mot de passe SMTP ne traverse plus la frontière. Il est lu dans le
//!    trousseau de l'OS au moment de l'envoi (défaut D2).

mod mail;
mod secrets;

use mail::{EmailData, SendOutcome, SmtpConfig};

/// Mot de passe à utiliser pour une requête donnée.
///
/// Priorité au trousseau. Le champ `pass` de la configuration reste accepté
/// pour un cas précis : le bouton « Tester la connexion », qui doit pouvoir
/// valider un mot de passe saisi avant qu'il ne soit enregistré.
fn resolve_password(config: &SmtpConfig) -> Result<String, String> {
    if let Some(explicit) = config.pass.as_ref().filter(|value| !value.is_empty()) {
        return Ok(explicit.clone());
    }

    secrets::read()
        .map_err(|error| error.to_string())
        .map(|stored| stored.unwrap_or_default())
}

#[tauri::command]
async fn send_email(smtp_config: SmtpConfig, email_data: EmailData) -> SendOutcome {
    let password = match resolve_password(&smtp_config) {
        Ok(password) => password,
        Err(error) => return SendOutcome::failed(error),
    };

    // L'envoi SMTP est bloquant : il est déporté hors du fil principal pour ne
    // pas figer l'interface pendant la négociation TLS.
    let result = tauri::async_runtime::spawn_blocking(move || {
        mail::send(&smtp_config, &email_data, &password)
    })
    .await;

    match result {
        Ok(Ok(message_id)) => SendOutcome::ok(Some(message_id)),
        Ok(Err(error)) => SendOutcome::failed(error),
        Err(error) => SendOutcome::failed(error),
    }
}

#[tauri::command]
async fn test_smtp(smtp_config: SmtpConfig) -> SendOutcome {
    let password = match resolve_password(&smtp_config) {
        Ok(password) => password,
        Err(error) => return SendOutcome::failed(error),
    };

    let result = tauri::async_runtime::spawn_blocking(move || {
        mail::verify(&smtp_config, &password)
    })
    .await;

    match result {
        Ok(Ok(())) => SendOutcome::ok(None),
        Ok(Err(error)) => SendOutcome::failed(error),
        Err(error) => SendOutcome::failed(error),
    }
}

#[tauri::command]
fn set_smtp_password(password: String) -> Result<(), String> {
    secrets::store(&password).map_err(|error| error.to_string())
}

#[tauri::command]
fn has_smtp_password() -> Result<bool, String> {
    secrets::exists().map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_smtp_password() -> Result<(), String> {
    secrets::clear().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            send_email,
            test_smtp,
            set_smtp_password,
            has_smtp_password,
            clear_smtp_password,
        ])
        .run(tauri::generate_context!())
        .expect("échec du démarrage d'Asgard CRM");
}
