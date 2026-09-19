//! Opérations envoyées à l'hôte.
//!
//! Dans `App.jsx`, une trentaine de gestionnaires mêlaient appel à l'hôte,
//! rechargement et message à l'utilisateur, chacun à sa façon. Ils sont réunis
//! ici, et suivent tous le même chemin : appeler, puis recharger, puis informer.
//! Les vues ne font plus que déclencher.

use asgard_ipc::{ClientInput, DocumentInput, ExpenseInput, PaymentInput};
use leptos::prelude::Set;
use serde::{Deserialize, Serialize};

use crate::ipc::{self, IpcError};
use crate::settings::{self, Settings};
use crate::state::{App, Pending};
use crate::templates::Kind;

/// Compte rendu d'une suppression, tel que le renvoie l'hôte depuis la phase 2.
#[derive(Debug, Deserialize)]
struct DeleteOutcome {
    discarded: usize,
    archived: usize,
}

/// Message affiché quand des factures émises ont été archivées plutôt
/// qu'effacées. Textes repris de `App.jsx` (phase 2).
fn archive_notice(single: bool, outcome: &DeleteOutcome) -> (&'static str, String) {
    if single {
        (
            "Facture archivée",
            concat!(
                "Cette facture a été émise : elle est retirée de la liste mais conservée. ",
                "Le code de commerce impose dix ans de conservation — une facture émise ",
                "s'annule par un avoir, elle ne se supprime pas.",
            )
            .into(),
        )
    } else {
        (
            "Factures archivées",
            format!(
                "{} facture(s) émise(s) ont été retirées de la liste mais conservées, \
                 comme l'impose le code de commerce. {} brouillon(s) supprimé(s).",
                outcome.archived, outcome.discarded
            ),
        )
    }
}

/// Réponse des commandes d'envoi, forme héritée des handlers Electron.
#[derive(Debug, Deserialize)]
pub struct SendOutcome {
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// Configuration SMTP envoyée à l'hôte. **Sans mot de passe** : l'hôte le lit
/// dans le trousseau du système depuis la phase 1.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtpConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub secure: String,
    pub from: String,
    /// Mot de passe saisi mais pas encore enregistré, pour le seul test de
    /// connexion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,
}

impl SmtpConfig {
    pub fn from_settings(settings: &Settings, pass: Option<String>) -> Self {
        Self {
            host: settings.smtp_host.clone(),
            port: settings.smtp_port.clone(),
            user: settings.smtp_user.clone(),
            secure: settings.smtp_secure.clone(),
            from: settings.email.clone(),
            pass: pass.filter(|p| !p.is_empty()),
        }
    }
}

async fn finish(app: App, result: Result<(), IpcError>) {
    match result {
        Ok(()) => app.reload().await,
        Err(error) => app.report(error),
    }
}

/// Exécute une action confirmée.
pub async fn execute(app: App, action: Pending) {
    match action {
        Pending::DeleteClient(id) => {
            #[derive(Serialize)]
            struct Args {
                id: i64,
            }
            finish(app, ipc::invoke("delete_client", &Args { id }).await).await;
        }

        Pending::DeleteInvoices(ids) => {
            #[derive(Serialize)]
            struct Args {
                ids: Vec<i64>,
            }
            let single = ids.len() == 1;
            match ipc::invoke::<_, DeleteOutcome>("delete_invoices", &Args { ids }).await {
                Ok(outcome) => {
                    app.reload().await;
                    // Depuis la phase 2, une facture émise est archivée et non
                    // effacée. Le dire évite qu'on la croie perdue.
                    if outcome.archived > 0 {
                        let (title, body) = archive_notice(single, &outcome);
                        app.inform(title, body);
                    }
                }
                Err(error) => app.report(error),
            }
        }

        Pending::DeleteEstimate(id) => {
            #[derive(Serialize)]
            struct Args {
                id: i64,
            }
            match ipc::invoke::<_, DeleteOutcome>("delete_estimate", &Args { id }).await {
                Ok(outcome) => {
                    app.reload().await;
                    if outcome.archived > 0 {
                        app.inform(
                            "Devis archivé",
                            "Ce devis a déjà été envoyé ou accepté : il est retiré de la \
                             liste mais conservé.",
                        );
                    }
                }
                Err(error) => app.report(error),
            }
        }

        Pending::DeleteExpense(id) => {
            #[derive(Serialize)]
            struct Args {
                id: i64,
            }
            finish(app, ipc::invoke("delete_expense", &Args { id }).await).await;
        }

        Pending::ConvertEstimate { id, .. } => {
            #[derive(Serialize)]
            struct Args {
                id: i64,
            }
            match ipc::invoke::<_, asgard_ipc::Invoice>("convert_estimate", &Args { id }).await {
                Ok(invoice) => {
                    app.reload().await;
                    app.tab.set(crate::state::Tab::Invoices);
                    app.inform(
                        "Conversion réussie !",
                        format!(
                            "Le devis a été converti en facture {} et enregistré en brouillon.",
                            invoice.invoice_number
                        ),
                    );
                }
                Err(error) => app.report_as("Erreur lors de la conversion", error),
            }
        }

        Pending::ImportBackup(backup) => crate::backup::import(app, backup).await,
    }
}

pub async fn save_client(app: App, client: ClientInput) -> bool {
    #[derive(Serialize)]
    struct Args {
        client: ClientInput,
    }
    match ipc::invoke::<_, i64>("save_client", &Args { client }).await {
        Ok(_) => {
            app.reload().await;
            true
        }
        Err(error) => {
            app.report(error);
            false
        }
    }
}

/// Crée une facture. L'hôte attribue le numéro et calcule TVA et total.
pub async fn create_invoice(app: App, invoice: DocumentInput) -> bool {
    #[derive(Serialize)]
    struct Args {
        invoice: DocumentInput,
    }
    match ipc::invoke::<_, asgard_ipc::Invoice>("create_invoice", &Args { invoice }).await {
        Ok(_) => {
            app.reload().await;
            true
        }
        Err(error) => {
            app.report(error);
            false
        }
    }
}

pub async fn save_estimate(app: App, estimate: DocumentInput) -> bool {
    #[derive(Serialize)]
    struct Args {
        estimate: DocumentInput,
    }
    match ipc::invoke::<_, asgard_ipc::Estimate>("save_estimate", &Args { estimate }).await {
        Ok(_) => {
            app.reload().await;
            app.inform("Succès", "Devis enregistré avec succès !");
            true
        }
        Err(error) => {
            app.report_as("Erreur d'enregistrement", error);
            false
        }
    }
}

pub async fn save_expense(app: App, expense: ExpenseInput) -> bool {
    #[derive(Serialize)]
    struct Args {
        expense: ExpenseInput,
    }
    match ipc::invoke::<_, i64>("save_expense", &Args { expense }).await {
        Ok(_) => {
            app.reload().await;
            app.inform("Succès", "Dépense enregistrée !");
            true
        }
        Err(error) => {
            app.report_as("Erreur d'enregistrement", error);
            false
        }
    }
}

pub async fn record_payment(app: App, payment: PaymentInput, number: String) -> bool {
    #[derive(Serialize)]
    struct Args {
        payment: PaymentInput,
    }
    match ipc::invoke::<_, ()>("record_payment", &Args { payment }).await {
        Ok(()) => {
            app.reload().await;
            app.inform("Succès", format!("Facture {number} marquée comme payée."));
            true
        }
        Err(error) => {
            app.report_as("Impossible d'enregistrer le règlement", error);
            false
        }
    }
}

/// Coordonnées de l'émetteur imprimées sur le PDF, tirées des réglages.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IssuerArgs {
    company_name: String,
    contact_name: String,
    address: String,
    phone: String,
    email: String,
    siret: String,
    iban: String,
    custom_color: String,
}

impl From<&Settings> for IssuerArgs {
    fn from(s: &Settings) -> Self {
        Self {
            company_name: s.company_name.clone(),
            contact_name: s.contact_name.clone(),
            address: s.address.clone(),
            phone: s.phone.clone(),
            email: s.email.clone(),
            siret: s.siret.clone(),
            iban: s.iban.clone(),
            custom_color: s.custom_color.clone(),
        }
    }
}

/// Enregistre une pièce en PDF, après le sélecteur de fichiers du système.
pub async fn export_pdf(app: App, kind: Kind, id: i64) {
    #[derive(Serialize)]
    struct Args {
        kind: Kind,
        id: i64,
        issuer: IssuerArgs,
    }

    let issuer = IssuerArgs::from(&settings::load());
    match ipc::invoke::<_, Option<String>>("export_document", &Args { kind, id, issuer }).await {
        Ok(Some(path)) => app.inform("Enregistré", saved_notice(kind, &path)),
        // L'utilisateur a fermé le sélecteur : ce n'est pas une erreur.
        Ok(None) => {}
        Err(error) => app.report(error),
    }
}

/// « Facture enregistrée : … » ou « Devis enregistré : … », comme l'original.
fn saved_notice(kind: Kind, path: &str) -> String {
    match kind {
        Kind::Estimate => format!("Devis enregistré : {path}"),
        Kind::Invoice | Kind::Reminder => format!("Facture enregistrée : {path}"),
    }
}

/// Confirmation d'envoi. L'original annonçait « La facture » même pour un
/// devis ; le port nomme la bonne pièce.
fn sent_notice(kind: Kind, recipient: &str) -> String {
    match kind {
        Kind::Estimate => format!("Le devis a été envoyé avec succès à {recipient} !"),
        Kind::Invoice | Kind::Reminder => {
            format!("La facture a été envoyée avec succès à {recipient} !")
        }
    }
}

/// Envoie une pièce par e-mail. L'hôte produit et attache le PDF lui-même.
pub async fn send_document(
    app: App,
    kind: Kind,
    id: i64,
    to: String,
    subject: String,
    text: String,
) -> bool {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Message {
        to: String,
        subject: String,
        text: String,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        kind: Kind,
        id: i64,
        issuer: IssuerArgs,
        smtp_config: SmtpConfig,
        message: Message,
    }

    let current = settings::load();
    let recipient = to.clone();
    let args = Args {
        kind,
        id,
        issuer: IssuerArgs::from(&current),
        smtp_config: SmtpConfig::from_settings(&current, None),
        message: Message { to, subject, text },
    };

    match ipc::invoke::<_, SendOutcome>("send_document", &args).await {
        Ok(outcome) if outcome.success => {
            app.inform("Succès", sent_notice(kind, &recipient));
            true
        }
        Ok(outcome) => {
            app.inform(
                "Échec de l'envoi",
                format!("Erreur SMTP : {}", outcome.error.unwrap_or_default()),
            );
            false
        }
        Err(error) => {
            app.report_as("Une erreur s'est produite lors de la génération ou de l'envoi du mail", error);
            false
        }
    }
}

/// Enregistre un contenu texte après le sélecteur du système.
///
/// Renvoie le chemin choisi, ou `None` si l'utilisateur a fermé le sélecteur.
/// Le message de confirmation est laissé à l'appelant : l'original nommait ce
/// qui venait d'être enregistré (« Livre des recettes enregistré : … »).
pub async fn export_text(
    contents: String,
    name: String,
    extension: &'static str,
) -> Result<Option<String>, IpcError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        contents: String,
        suggested_name: String,
        extension: &'static str,
    }

    let args = Args { contents, suggested_name: name, extension };
    ipc::invoke("export_text", &args).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le mot de passe ne doit jamais partir vers l'hôte lors d'un envoi : il
    /// est lu dans le trousseau. Seul le test de connexion en transmet un.
    #[test]
    fn smtp_config_omits_an_absent_password() {
        let config = SmtpConfig::from_settings(&Settings::default(), None);
        let json = serde_json::to_string(&config).unwrap();

        assert!(!json.contains("pass"), "{json}");
    }

    #[test]
    fn an_empty_draft_password_is_not_sent_either() {
        let config = SmtpConfig::from_settings(&Settings::default(), Some(String::new()));
        assert!(config.pass.is_none());
    }

    #[test]
    fn smtp_config_uses_the_names_the_host_expects() {
        let config = SmtpConfig::from_settings(&Settings::default(), Some("secret".into()));
        let json = serde_json::to_string(&config).unwrap();

        for key in ["\"host\"", "\"port\"", "\"user\"", "\"secure\"", "\"from\"", "\"pass\""] {
            assert!(json.contains(key), "clé manquante {key} dans {json}");
        }
    }

    #[test]
    fn a_single_archived_invoice_gets_its_own_explanation() {
        let outcome = DeleteOutcome { discarded: 0, archived: 1 };
        let (title, body) = archive_notice(true, &outcome);
        assert_eq!(title, "Facture archivée");
        assert!(body.contains("s'annule par un avoir"));
        assert!(!body.contains("  "), "{body}");
    }

    #[test]
    fn a_bulk_deletion_reports_both_counts() {
        let outcome = DeleteOutcome { discarded: 2, archived: 3 };
        let (title, body) = archive_notice(false, &outcome);
        assert_eq!(title, "Factures archivées");
        assert_eq!(
            body,
            "3 facture(s) émise(s) ont été retirées de la liste mais conservées, comme \
             l'impose le code de commerce. 2 brouillon(s) supprimé(s)."
        );
    }

    #[test]
    fn notices_name_the_right_document() {
        assert_eq!(sent_notice(Kind::Estimate, "a@b.fr"), "Le devis a été envoyé avec succès à a@b.fr !");
        assert_eq!(sent_notice(Kind::Reminder, "a@b.fr"), "La facture a été envoyée avec succès à a@b.fr !");
        assert_eq!(saved_notice(Kind::Invoice, "C:/f.pdf"), "Facture enregistrée : C:/f.pdf");
        assert_eq!(saved_notice(Kind::Estimate, "C:/d.pdf"), "Devis enregistré : C:/d.pdf");
    }

    #[test]
    fn issuer_arguments_are_camel_case() {
        let json = serde_json::to_string(&IssuerArgs::from(&Settings::default())).unwrap();
        assert!(json.contains("\"companyName\""));
        assert!(json.contains("\"customColor\""));
    }
}
