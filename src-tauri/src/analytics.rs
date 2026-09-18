//! Pont entre la base et le noyau métier.
//!
//! `asgard-core` ne connaît ni SQLite ni Tauri : il prend des données et rend
//! des données. Ce module fait la traduction, une fois, à un seul endroit.
//!
//! C'est aussi ici que les montants quittent le `f64` : la base stocke des
//! centimes entiers depuis la phase 2, et ils deviennent des `Decimal` sans
//! jamais passer par un flottant. Le seul `f64` restant est à la sortie, juste
//! avant l'interface, qui manipule encore des `Number` JavaScript.

use asgard_core::model::{CivilDate, Expense, Invoice, ServiceType, Settings, Status};
use asgard_core::money::{from_cents, from_f64};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};

use crate::db::DbError;

/// Réglages tels que l'interface les envoie, en `camelCase` et en flottants.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    #[serde(default = "default_bnc")]
    pub urssaf_service_bnc: f64,
    #[serde(default = "default_bic")]
    pub urssaf_service_bic: f64,
    #[serde(default = "default_vente")]
    pub urssaf_vente: f64,
    #[serde(default)]
    pub acre_enabled: bool,
}

fn default_bnc() -> f64 {
    21.1
}
fn default_bic() -> f64 {
    21.1
}
fn default_vente() -> f64 {
    12.3
}

impl From<SettingsInput> for Settings {
    fn from(input: SettingsInput) -> Self {
        Settings {
            urssaf_service_bnc: from_f64(input.urssaf_service_bnc),
            urssaf_service_bic: from_f64(input.urssaf_service_bic),
            urssaf_vente: from_f64(input.urssaf_vente),
            acre_enabled: input.acre_enabled,
        }
    }
}

/// Un type d'activité inconnu retombe sur les BNC, comme le faisait le
/// JavaScript. C'est la valeur par défaut des formulaires.
fn service_type(raw: &str) -> ServiceType {
    match raw {
        "service_bic" => ServiceType::ServiceBic,
        "vente" => ServiceType::Vente,
        _ => ServiceType::ServiceBnc,
    }
}

fn status(raw: &str) -> Status {
    match raw {
        "payee" => Status::Payee,
        "envoyee" => Status::Envoyee,
        _ => Status::Brouillon,
    }
}

/// Charge les factures depuis la base, directement en types du domaine.
///
/// Les montants passent des centimes entiers au `Decimal` sans étape flottante
/// intermédiaire — c'est ce qui rend le calcul exact de bout en bout.
pub async fn load_invoices(pool: &SqlitePool) -> Result<Vec<Invoice>, DbError> {
    let rows = sqlx::query(
        "SELECT id, invoice_number, company_name, service_type, amount_ht_cents, tva_rate,
                amount_tva_cents, amount_total_cents, date, status, payment_date, payment_method
         FROM invoices WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let date: String = r.get("date");
            let payment_date: Option<String> = r.get("payment_date");

            Invoice {
                id: r.get("id"),
                invoice_number: r.get("invoice_number"),
                company_name: r.get("company_name"),
                service_type: service_type(&r.get::<String, _>("service_type")),
                amount_ht: from_cents(r.get("amount_ht_cents")),
                tva_rate: from_f64(r.get("tva_rate")),
                amount_tva: from_cents(r.get("amount_tva_cents")),
                amount_total: from_cents(r.get("amount_total_cents")),
                date: CivilDate::parse(&date).unwrap_or(CivilDate::new(1970, 1, 1)),
                status: status(&r.get::<String, _>("status")),
                payment_date: payment_date.as_deref().and_then(CivilDate::parse),
                payment_method: r.get("payment_method"),
            }
        })
        .collect())
}

pub async fn load_expenses(pool: &SqlitePool) -> Result<Vec<Expense>, DbError> {
    let rows = sqlx::query(
        "SELECT id, date, merchant, category, amount_cents
         FROM expenses WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let date: String = r.get("date");

            Expense {
                id: r.get("id"),
                date: CivilDate::parse(&date).unwrap_or(CivilDate::new(1970, 1, 1)),
                merchant: r.get("merchant"),
                category: r.get("category"),
                amount: from_cents(r.get("amount_cents")),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_activity_types_and_statuses() {
        assert_eq!(service_type("service_bic"), ServiceType::ServiceBic);
        assert_eq!(service_type("vente"), ServiceType::Vente);
        assert_eq!(service_type("service_bnc"), ServiceType::ServiceBnc);

        // Une valeur inconnue ne fait pas échouer la lecture : elle retombe sur
        // le défaut des formulaires, comme le JavaScript le faisait.
        assert_eq!(service_type("inconnu"), ServiceType::ServiceBnc);

        assert_eq!(status("payee"), Status::Payee);
        assert_eq!(status("envoyee"), Status::Envoyee);
        assert_eq!(status("n'importe quoi"), Status::Brouillon);
    }

    #[test]
    fn settings_default_to_the_published_rates() {
        let input: SettingsInput = serde_json::from_str("{}").unwrap();
        let settings: Settings = input.into();

        assert_eq!(settings.urssaf_service_bnc, from_f64(21.1));
        assert_eq!(settings.urssaf_vente, from_f64(12.3));
        assert!(!settings.acre_enabled);
    }

    #[test]
    fn settings_survive_the_float_boundary_without_artifacts() {
        let input: SettingsInput =
            serde_json::from_str(r#"{"urssafServiceBnc": 21.1, "acreEnabled": true}"#).unwrap();
        let settings: Settings = input.into();

        assert_eq!(settings.urssaf_service_bnc.to_string(), "21.1");
        assert!(settings.acre_enabled);
    }
}
