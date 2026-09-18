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

mod analytics;
mod db;
mod documents;
mod mail;
mod secrets;

use tauri::Manager;

use db::backup::{ImportReport, LegacyBackup};
use asgard_ipc::*;
use db::repo::{self, DeleteOutcome};
use db::{Db, DbError};
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


// ------------------------------------------------------------ base de données
//
// Les commandes renvoient exactement les formes que Dexie produisait, pour que
// les composants React restent inchangés. Seul `src/db.js` a changé de nature :
// d'un schéma Dexie il devient un adaptateur vers ces commandes.

#[tauri::command]
async fn load_snapshot(db: tauri::State<'_, Db>) -> Result<Snapshot, DbError> {
    repo::snapshot(&db.pool).await
}

#[tauri::command]
async fn save_client(db: tauri::State<'_, Db>, client: ClientInput) -> Result<i64, DbError> {
    repo::save_client(&db.pool, client).await
}

#[tauri::command]
async fn delete_client(db: tauri::State<'_, Db>, id: i64) -> Result<(), DbError> {
    repo::delete_client(&db.pool, id).await
}

#[tauri::command]
async fn create_invoice(db: tauri::State<'_, Db>, invoice: DocumentInput) -> Result<Invoice, DbError> {
    repo::create_invoice(&db.pool, invoice).await
}

#[tauri::command]
async fn set_invoice_status(db: tauri::State<'_, Db>, id: i64, status: String) -> Result<(), DbError> {
    repo::set_invoice_status(&db.pool, id, &status).await
}

#[tauri::command]
async fn record_payment(db: tauri::State<'_, Db>, payment: PaymentInput) -> Result<(), DbError> {
    repo::record_payment(&db.pool, payment).await
}

#[tauri::command]
async fn delete_invoices(db: tauri::State<'_, Db>, ids: Vec<i64>) -> Result<DeleteOutcome, DbError> {
    repo::delete_invoices(&db.pool, &ids).await
}

#[tauri::command]
async fn save_estimate(db: tauri::State<'_, Db>, estimate: DocumentInput) -> Result<Estimate, DbError> {
    repo::save_estimate(&db.pool, estimate).await
}

#[tauri::command]
async fn set_estimate_status(db: tauri::State<'_, Db>, id: i64, status: String) -> Result<(), DbError> {
    repo::set_estimate_status(&db.pool, id, &status).await
}

#[tauri::command]
async fn delete_estimate(db: tauri::State<'_, Db>, id: i64) -> Result<DeleteOutcome, DbError> {
    repo::delete_estimate(&db.pool, id).await
}

#[tauri::command]
async fn convert_estimate(db: tauri::State<'_, Db>, id: i64) -> Result<Invoice, DbError> {
    repo::convert_estimate(&db.pool, id).await
}

#[tauri::command]
async fn save_expense(db: tauri::State<'_, Db>, expense: ExpenseInput) -> Result<i64, DbError> {
    repo::save_expense(&db.pool, expense).await
}

#[tauri::command]
async fn delete_expense(db: tauri::State<'_, Db>, id: i64) -> Result<(), DbError> {
    repo::delete_expense(&db.pool, id).await
}

#[tauri::command]
async fn export_backup(db: tauri::State<'_, Db>) -> Result<serde_json::Value, DbError> {
    db::backup::export(&db.pool).await
}

/// Reprend une sauvegarde et rend compte de ce qui a changé.
///
/// Le rapport n'est pas décoratif : le passage des flottants aux centimes
/// modifie certains montants, et l'utilisateur doit pouvoir expliquer un
/// chiffre qui a bougé plutôt que de le découvrir dans une déclaration.
#[tauri::command]
async fn import_backup(
    db: tauri::State<'_, Db>,
    backup: serde_json::Value,
) -> Result<ImportReport, DbError> {
    let parsed: LegacyBackup = serde_json::from_value(backup)?;
    db::backup::import(&db.pool, parsed).await
}

/// Copie atomique du fichier de base, ce qu'IndexedDB ne permettait pas.
#[tauri::command]
async fn backup_to_file(db: tauri::State<'_, Db>, destination: String) -> Result<(), DbError> {
    db::backup::vacuum_into(&db.pool, &destination).await
}

/// Chemin du fichier, pour que l'utilisateur sache où vivent ses données.
#[tauri::command]
fn database_path(db: tauri::State<'_, Db>) -> String {
    db.path.to_string_lossy().to_string()
}


// ------------------------------------------------------------ calculs metier
//
// Phase 3 : les agregats ne sont plus calcules dans le JSX mais dans
// `asgard-core`, en arithmetique decimale. L'interface ne fait plus qu'afficher.

/// Tout ce que le tableau de bord montre, en un seul calcul.
#[tauri::command]
async fn dashboard(
    db: tauri::State<'_, Db>,
    settings: analytics::SettingsInput,
    year: i32,
) -> Result<asgard_core::reporting::Dashboard, DbError> {
    let invoices = analytics::load_invoices(&db.pool).await?;
    let expenses = analytics::load_expenses(&db.pool).await?;

    Ok(asgard_core::reporting::dashboard(
        &invoices,
        &expenses,
        &settings.into(),
        year,
    ))
}

/// Declaration URSSAF d'une periode, assise sur le chiffre d'affaires encaisse.
#[tauri::command]
async fn urssaf_declaration(
    db: tauri::State<'_, Db>,
    settings: analytics::SettingsInput,
    period: asgard_core::urssaf::Period,
) -> Result<asgard_core::urssaf::Declaration, DbError> {
    let invoices = analytics::load_invoices(&db.pool).await?;

    Ok(asgard_core::urssaf::declaration(
        &invoices,
        &settings.into(),
        period,
    ))
}

/// Livre des recettes au format CSV reglementaire.
#[tauri::command]
async fn recettes_csv(db: tauri::State<'_, Db>) -> Result<String, DbError> {
    let invoices = analytics::load_invoices(&db.pool).await?;
    Ok(asgard_core::reporting::recettes_csv(&invoices))
}


// ---------------------------------------------------------- pieces et envois
//
// Phase 4 : le PDF est produit par l'hote. L'interface ne fait plus que demander
// une piece par son identifiant — plus de document encode en base64 traversant
// l'IPC, et plus de jsPDF.

use tauri_plugin_dialog::DialogExt;

/// Enregistre une piece sur le disque, apres le selecteur du systeme.
///
/// Renvoie le chemin choisi, ou `None` si l'utilisateur a renonce. Remplace le
/// `<a download>` sur une URL blob:, dont le comportement sous la CSP de Tauri
/// n'avait jamais pu etre verifie.
#[tauri::command]
async fn export_document(
    app: tauri::AppHandle,
    db: tauri::State<'_, Db>,
    kind: documents::Kind,
    id: i64,
    issuer: documents::IssuerInput,
) -> Result<Option<String>, DbError> {
    let rendered = documents::render(&db.pool, kind, id, issuer.into()).await?;

    let chosen = app
        .dialog()
        .file()
        .set_file_name(format!("{}.pdf", rendered.number))
        .add_filter("Document PDF", &["pdf"])
        .blocking_save_file();

    let Some(path) = chosen else { return Ok(None) };
    let path = path
        .into_path()
        .map_err(|error| DbError::Pdf(error.to_string()))?;

    std::fs::write(&path, &rendered.bytes)?;
    Ok(Some(path.to_string_lossy().to_string()))
}

/// Enregistre un contenu texte — livre des recettes, sauvegarde JSON.
///
/// Meme raison que ci-dessus : ces deux exports passaient par un telechargement
/// declenche depuis la page.
#[tauri::command]
async fn export_text(
    app: tauri::AppHandle,
    contents: String,
    suggested_name: String,
    extension: String,
) -> Result<Option<String>, DbError> {
    let chosen = app
        .dialog()
        .file()
        .set_file_name(&suggested_name)
        .add_filter("Fichier", &[extension.as_str()])
        .blocking_save_file();

    let Some(path) = chosen else { return Ok(None) };
    let path = path
        .into_path()
        .map_err(|error| DbError::Pdf(error.to_string()))?;

    std::fs::write(&path, contents.as_bytes())?;
    Ok(Some(path.to_string_lossy().to_string()))
}

/// Envoie une piece par e-mail, piece jointe comprise.
///
/// Le PDF est produit ici : ni le document ni le mot de passe SMTP ne traversent
/// la frontiere.
#[tauri::command]
async fn send_document(
    db: tauri::State<'_, Db>,
    kind: documents::Kind,
    id: i64,
    issuer: documents::IssuerInput,
    smtp_config: SmtpConfig,
    message: DocumentMessage,
) -> Result<SendOutcome, DbError> {
    // Le resultat est toujours `Ok` : les echecs d'envoi voyagent dans
    // `SendOutcome`, forme que le JSX attend depuis les handlers Electron. Tauri
    // exige neanmoins un `Result` des lors qu'une commande asynchrone emprunte
    // un `State`.
    let rendered = match documents::render(&db.pool, kind, id, issuer.into()).await {
        Ok(rendered) => rendered,
        Err(error) => return Ok(SendOutcome::failed(error)),
    };

    let password = match resolve_password(&smtp_config) {
        Ok(password) => password,
        Err(error) => return Ok(SendOutcome::failed(error)),
    };

    let email = EmailData {
        to: message.to,
        subject: message.subject,
        text: message.text,
        filename: Some(format!("{}.pdf", rendered.number)),
        attachment: Some(rendered.bytes),
    };

    let result = tauri::async_runtime::spawn_blocking(move || {
        mail::send(&smtp_config, &email, &password)
    })
    .await;

    Ok(match result {
        Ok(Ok(message_id)) => SendOutcome::ok(Some(message_id)),
        Ok(Err(error)) => SendOutcome::failed(error),
        Err(error) => SendOutcome::failed(error),
    })
}

/// Adresse enregistree du client d'une piece, pour pre-remplir le formulaire.
#[tauri::command]
async fn document_recipient(
    db: tauri::State<'_, Db>,
    kind: documents::Kind,
    id: i64,
) -> Result<Option<String>, DbError> {
    documents::recipient(&db.pool, kind, id).await
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocumentMessage {
    to: String,
    subject: String,
    text: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;

            // L'ouverture est bloquante et doit aboutir avant le premier rendu :
            // sans base, aucune commande de données ne peut répondre.
            let db = tauri::async_runtime::block_on(async move {
                let db = db::open(data_dir).await?;
                db::seed_demo_data_if_empty(&db.pool).await?;
                Ok::<_, DbError>(db)
            })?;

            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            test_smtp,
            set_smtp_password,
            has_smtp_password,
            clear_smtp_password,
            load_snapshot,
            save_client,
            delete_client,
            create_invoice,
            set_invoice_status,
            record_payment,
            delete_invoices,
            save_estimate,
            set_estimate_status,
            delete_estimate,
            convert_estimate,
            save_expense,
            delete_expense,
            export_backup,
            import_backup,
            backup_to_file,
            database_path,
            dashboard,
            urssaf_declaration,
            recettes_csv,
            export_document,
            export_text,
            send_document,
            document_recipient,
        ])
        .run(tauri::generate_context!())
        .expect("échec du démarrage d'Asgard CRM");
}
