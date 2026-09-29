//! État applicatif.
//!
//! Remplace les 39 `useState` de `App.jsx`. Le regroupement en une structure
//! unique n'est pas cosmétique : dans la version React, chaque écriture devait
//! penser à rappeler `loadAllData()`, et un oubli laissait l'écran désynchronisé
//! de la base sans que rien ne le signale. Ici `reload()` est le seul chemin,
//! et les vues n'ont pas à s'en souvenir.

use asgard_core::reporting::Dashboard;
use asgard_core::urssaf::Declaration;
use asgard_ipc::Snapshot;
use leptos::prelude::*;
use serde::Serialize;

use crate::ipc;

/// Onglet affiché.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Clients,
    Estimates,
    Invoices,
    Expenses,
    Compta,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 7] = [
        Tab::Dashboard,
        Tab::Clients,
        Tab::Estimates,
        Tab::Invoices,
        Tab::Expenses,
        Tab::Compta,
        Tab::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Clients => "Clients",
            Tab::Estimates => "Devis",
            Tab::Invoices => "Factures",
            Tab::Expenses => "Dépenses",
            Tab::Compta => "Comptabilité",
            Tab::Settings => "Paramètres",
        }
    }
}

/// Période de déclaration choisie dans l'onglet Comptabilité.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub monthly: bool,
    pub year: i32,
    pub month: u32,
    pub quarter: u32,
}

impl Default for Period {
    fn default() -> Self {
        Self { monthly: true, year: 2026, month: 1, quarter: 1 }
    }
}

/// Forme attendue par les commandes `dashboard` et `urssaf_declaration`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PeriodPayload {
    period_type: &'static str,
    year: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    month: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quarter: Option<u32>,
}

impl From<Period> for PeriodPayload {
    fn from(period: Period) -> Self {
        if period.monthly {
            Self {
                period_type: "monthly",
                year: period.year,
                month: Some(period.month),
                quarter: None,
            }
        } else {
            Self {
                period_type: "quarterly",
                year: period.year,
                month: None,
                quarter: Some(period.quarter),
            }
        }
    }
}

/// Action destructrice en attente de confirmation.
///
/// La version React stockait une closure `onConfirm` dans l'état. Un enum est
/// préférable : chaque cas est nommé, testable, et le texte de confirmation
/// vit à côté de l'action qu'il annonce au lieu d'être dispersé dans six
/// gestionnaires.
#[derive(Debug, Clone, PartialEq)]
pub enum Pending {
    DeleteClient(i64),
    DeleteInvoices(Vec<i64>),
    DeleteEstimate(i64),
    DeleteExpense(i64),
    ConvertEstimate { id: i64, number: String },
    /// Restauration d'une sauvegarde déjà lue et reconnue.
    ImportBackup(serde_json::Value),
}

impl Pending {
    pub fn title(&self) -> &'static str {
        match self {
            Pending::DeleteClient(_) => "Supprimer le client",
            Pending::DeleteInvoices(ids) if ids.len() > 1 => "Supprimer les factures sélectionnées",
            Pending::DeleteInvoices(_) => "Supprimer la facture",
            Pending::DeleteEstimate(_) => "Supprimer le devis",
            Pending::DeleteExpense(_) => "Supprimer la dépense",
            Pending::ConvertEstimate { .. } => "Convertir en facture",
            Pending::ImportBackup(_) => "Restaurer une sauvegarde",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Pending::DeleteClient(_) => concat!(
                "Êtes-vous sûr de vouloir supprimer ce client ? Toutes ses factures ",
                "associées resteront dans l'historique mais déconnectées.",
            )
            .into(),
            Pending::DeleteInvoices(ids) if ids.len() > 1 => format!(
                "Êtes-vous sûr de vouloir supprimer les {} factures sélectionnées ? Les \
                 factures émises seront conservées et retirées de la liste ; seuls les \
                 brouillons sont réellement effacés.",
                ids.len()
            ),
            Pending::DeleteInvoices(_) => concat!(
                "Êtes-vous sûr de vouloir supprimer cette facture ? Si elle a été émise, ",
                "elle sera conservée et retirée de la liste : une facture émise ne se ",
                "supprime pas.",
            )
            .into(),
            Pending::DeleteEstimate(_) => "Êtes-vous sûr de vouloir supprimer ce devis ?".into(),
            Pending::DeleteExpense(_) => {
                "Êtes-vous sûr de vouloir supprimer cette dépense ?".into()
            }
            Pending::ConvertEstimate { number, .. } => format!(
                "Voulez-vous convertir le devis {number} en facture ? Un nouveau numéro \
                 de facture sera généré automatiquement."
            ),
            // Repris du `window.confirm` de l'original.
            Pending::ImportBackup(_) => concat!(
                "Êtes-vous sûr de vouloir importer cette sauvegarde ? Cette action écrasera ",
                "TOUTES les données actuelles de l'application (clients, factures, devis, ",
                "dépenses et paramètres).",
            )
            .into(),
        }
    }
}

/// E-mail en cours de composition, relu par l'utilisateur avant envoi.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailCompose {
    pub kind: crate::templates::Kind,
    pub id: i64,
    pub number: String,
    pub to: String,
    pub subject: String,
    pub text: String,
}

/// Règlement en cours de saisie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentForm {
    pub invoice_id: i64,
    pub number: String,
}

/// Message affiché à l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

/// Tout l'état partagé, fourni par le composant racine.
#[derive(Clone, Copy)]
pub struct App {
    pub tab: RwSignal<Tab>,
    pub snapshot: RwSignal<Option<Snapshot>>,
    pub dashboard: RwSignal<Option<Dashboard>>,
    pub declaration: RwSignal<Option<Declaration>>,
    pub period: RwSignal<Period>,
    pub notice: RwSignal<Option<Notice>>,
    /// Action destructrice en attente de confirmation.
    pub pending: RwSignal<Option<Pending>>,
    /// Fenêtre d'envoi par e-mail, ouverte depuis plusieurs écrans.
    pub email: RwSignal<Option<EmailCompose>>,
    /// Fenêtre de règlement, ouverte depuis les factures et le tableau de bord.
    pub payment: RwSignal<Option<PaymentForm>>,
    /// Vrai tant que le premier chargement n'a pas abouti.
    pub loading: RwSignal<bool>,
    /// Incrémenté quand les réglages sont remplacés de l'extérieur (reprise
    /// d'une sauvegarde) : l'écran Paramètres se reconstruit alors, au lieu de
    /// réécrire les réglages restaurés avec les valeurs qu'il avait en mémoire.
    pub settings_epoch: RwSignal<u32>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            tab: RwSignal::new(Tab::Dashboard),
            snapshot: RwSignal::new(None),
            dashboard: RwSignal::new(None),
            declaration: RwSignal::new(None),
            period: RwSignal::new(Period::default()),
            notice: RwSignal::new(None),
            pending: RwSignal::new(None),
            email: RwSignal::new(None),
            payment: RwSignal::new(None),
            loading: RwSignal::new(true),
            settings_epoch: RwSignal::new(0),
        }
    }

    /// Affiche un message à l'utilisateur.
    pub fn inform(&self, title: impl Into<String>, body: impl Into<String>) {
        self.notice.set(Some(Notice {
            title: title.into(),
            body: body.into(),
        }));
    }

    /// Signale une opération qui a échoué, sans la faire disparaître.
    pub fn report(&self, error: ipc::IpcError) {
        self.inform("Erreur", error.to_string());
    }

    /// Même chose, précédé de ce qui était tenté, comme les messages de
    /// l'original (« Erreur lors de la conversion : … »).
    pub fn report_as(&self, context: &str, error: ipc::IpcError) {
        self.inform("Erreur", format!("{context} : {error}"));
    }

    /// Recharge données et agrégats.
    ///
    /// Seul chemin de rafraîchissement : toute écriture s'y termine, ce qui
    /// évite l'oubli de `loadAllData()` que la version React rendait possible.
    pub async fn reload(self) {
        match ipc::call::<Snapshot>("load_snapshot").await {
            Ok(snapshot) => self.snapshot.set(Some(snapshot)),
            Err(error) => {
                self.report(error);
                self.loading.set(false);
                return;
            }
        }

        self.refresh_metrics().await;
        self.loading.set(false);
    }

    /// Recalcule les agrégats et la déclaration de la période courante.
    pub async fn refresh_metrics(self) {
        let settings = crate::settings::load();
        let year = self.period.get_untracked().year;

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct DashboardArgs<'a> {
            settings: &'a crate::settings::Settings,
            year: i32,
        }

        match ipc::invoke::<_, Dashboard>("dashboard", &DashboardArgs { settings: &settings, year })
            .await
        {
            Ok(dashboard) => self.dashboard.set(Some(dashboard)),
            Err(error) => self.report(error),
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct DeclarationArgs<'a> {
            settings: &'a crate::settings::Settings,
            period: PeriodPayload,
        }

        let args = DeclarationArgs {
            settings: &settings,
            period: self.period.get_untracked().into(),
        };

        match ipc::invoke::<_, Declaration>("urssaf_declaration", &args).await {
            Ok(declaration) => self.declaration.set(Some(declaration)),
            Err(error) => self.report(error),
        }
    }
}

/// Récupère l'état depuis le contexte. Panique si la racine ne l'a pas fourni,
/// ce qui serait une erreur de programmation, pas un cas d'exécution.
/// Lit l'état partagé depuis le contexte Leptos.
///
/// **À appeler pendant le rendu seulement.** Dans un gestionnaire
/// d'événement, le contexte n'est plus accessible : l'appel panique, et une
/// panique dans le WebAssembly fige toute l'interface jusqu'au redémarrage de
/// l'application. Les fonctions déclenchées par un clic reçoivent donc l'état
/// en argument — `App` est `Copy`, une fermeture peut le capturer.
pub fn use_app() -> App {
    use_context::<App>().expect("l'état applicatif doit être fourni par la racine")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_titles_distinguish_one_invoice_from_several() {
        assert_eq!(Pending::DeleteInvoices(vec![1]).title(), "Supprimer la facture");
        assert_eq!(
            Pending::DeleteInvoices(vec![1, 2]).title(),
            "Supprimer les factures sélectionnées"
        );
    }

    /// Le message doit annoncer ce qui se passe réellement : depuis la phase 2,
    /// une facture émise est archivée, pas effacée.
    #[test]
    fn deleting_an_invoice_announces_that_issued_ones_are_kept() {
        assert!(Pending::DeleteInvoices(vec![1]).message().contains("conservée"));
        assert!(Pending::DeleteInvoices(vec![1, 2, 3]).message().contains("3 factures"));
    }

    #[test]
    fn converting_names_the_estimate() {
        let action = Pending::ConvertEstimate { id: 4, number: "DEV-X-2026-0004".into() };
        assert!(action.message().contains("DEV-X-2026-0004"));
    }

    #[test]
    fn every_tab_has_a_label() {
        for tab in Tab::ALL {
            assert!(!tab.label().is_empty(), "{tab:?}");
        }
    }

    /// Aucun message ne doit contenir de suite d'espaces : c'est le signe d'une
    /// continuation de ligne mal échappée, qui s'afficherait telle quelle.
    #[test]
    fn confirmation_messages_have_no_runs_of_spaces() {
        for action in [
            Pending::DeleteClient(1),
            Pending::DeleteInvoices(vec![1]),
            Pending::DeleteInvoices(vec![1, 2]),
            Pending::DeleteEstimate(1),
            Pending::DeleteExpense(1),
            Pending::ConvertEstimate { id: 1, number: "DEV-1".into() },
            Pending::ImportBackup(serde_json::Value::Null),
        ] {
            let message = action.message();
            assert!(!message.contains("  "), "espaces multiples dans : {message:?}");
        }
    }

    #[test]
    fn a_monthly_period_serialises_without_a_quarter() {
        let payload: PeriodPayload = Period { monthly: true, year: 2026, month: 6, quarter: 2 }.into();

        assert_eq!(payload.period_type, "monthly");
        assert_eq!(payload.month, Some(6));
        assert_eq!(payload.quarter, None);
    }

    #[test]
    fn a_quarterly_period_serialises_without_a_month() {
        let payload: PeriodPayload = Period { monthly: false, year: 2026, month: 6, quarter: 2 }.into();

        assert_eq!(payload.period_type, "quarterly");
        assert_eq!(payload.quarter, Some(2));
        assert_eq!(payload.month, None);
    }

    /// La forme doit correspondre à `asgard_core::urssaf::Period`, que l'hôte
    /// désérialise. Un champ mal nommé ici casserait la déclaration.
    #[test]
    fn the_period_payload_matches_what_the_host_expects() {
        let payload: PeriodPayload = Period::default().into();
        let json = serde_json::to_string(&payload).unwrap();

        assert!(json.contains("\"periodType\":\"monthly\""), "{json}");
        assert!(json.contains("\"year\""), "{json}");
    }
}
