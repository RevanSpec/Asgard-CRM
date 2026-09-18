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

    /// Identifiant du symbole dans `public/icons.svg`, réutilisé tel quel.
    pub fn icon(self) -> &'static str {
        match self {
            Tab::Dashboard => "dashboard",
            Tab::Clients => "users",
            Tab::Estimates => "file-text",
            Tab::Invoices => "file",
            Tab::Expenses => "credit-card",
            Tab::Compta => "bar-chart",
            Tab::Settings => "settings",
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
    /// Vrai tant que le premier chargement n'a pas abouti.
    pub loading: RwSignal<bool>,
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
            loading: RwSignal::new(true),
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
pub fn use_app() -> App {
    use_context::<App>().expect("l'état applicatif doit être fourni par la racine")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tab_has_a_label_and_an_icon() {
        for tab in Tab::ALL {
            assert!(!tab.label().is_empty(), "{tab:?}");
            assert!(!tab.icon().is_empty(), "{tab:?}");
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
