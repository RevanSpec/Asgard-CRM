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

/// Charge les factures **et les avoirs** depuis la base, en types du domaine.
///
/// Les montants passent des centimes entiers au `Decimal` sans étape flottante
/// intermédiaire — c'est ce qui rend le calcul exact de bout en bout.
///
/// Les avoirs entrent ici sous forme de recettes négatives (voir
/// `credits_as_negative_invoices`). C'est le seul endroit à le savoir : le
/// tableau de bord, les cotisations, les seuils et le livre des recettes les
/// prennent en compte sans une ligne de plus.
pub async fn load_invoices(pool: &SqlitePool) -> Result<Vec<Invoice>, DbError> {
    let rows = sqlx::query(
        "SELECT id, invoice_number, company_name, service_type, amount_ht_cents, tva_rate,
                amount_tva_cents, amount_total_cents, date, status, payment_date, payment_method
         FROM invoices WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    let mut invoices: Vec<Invoice> = rows
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
        .collect();

    invoices.extend(credits_as_negative_invoices(pool).await?);
    Ok(invoices)
}

/// Avoirs, vus comme des recettes négatives.
///
/// Un avoir se comporte exactement comme une facture de signe opposé : même
/// type d'activité — donc même taux de cotisation —, et un remboursement qui
/// joue le rôle d'un encaissement. Tant qu'il n'est pas remboursé, il ne
/// diminue que le facturé, jamais l'encaissé : c'est la règle de la
/// comptabilité de trésorerie, appliquée dans le sens inverse.
///
/// L'identifiant est rendu négatif : les deux tables numérotent chacune depuis
/// 1, et un doublon fausserait toute lecture par identifiant.
async fn credits_as_negative_invoices(pool: &SqlitePool) -> Result<Vec<Invoice>, DbError> {
    let rows = sqlx::query(
        "SELECT id, credit_number, company_name, service_type, amount_ht_cents, tva_rate,
                amount_tva_cents, amount_total_cents, date, refunded_on
         FROM credit_notes WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let date: String = r.get("date");
            let refunded_on: Option<String> = r.get("refunded_on");

            Invoice {
                id: -r.get::<i64, _>("id"),
                invoice_number: r.get("credit_number"),
                company_name: r.get("company_name"),
                service_type: service_type(&r.get::<String, _>("service_type")),
                amount_ht: -from_cents(r.get("amount_ht_cents")),
                tva_rate: from_f64(r.get("tva_rate")),
                amount_tva: -from_cents(r.get("amount_tva_cents")),
                amount_total: -from_cents(r.get("amount_total_cents")),
                date: CivilDate::parse(&date).unwrap_or(CivilDate::new(1970, 1, 1)),
                status: if refunded_on.is_some() { Status::Payee } else { Status::Envoyee },
                payment_date: refunded_on.as_deref().and_then(CivilDate::parse),
                payment_method: Some("remboursement".into()),
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

    use crate::db::repo;
    use asgard_ipc::{ClientInput, CreditNoteInput, DocumentInput};
    use sqlx::SqlitePool;

    async fn pool_with_a_paid_invoice() -> (SqlitePool, i64) {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        repo::save_client(
            &pool,
            ClientInput {
                id: None,
                company_name: "Asgard Coffee & Co".into(),
                contact_name: String::new(),
                email: String::new(),
                phone: String::new(),
                address: String::new(),
                siren: String::new(),
                vat_number: String::new(),
                delivery_address: String::new(),
            },
        )
        .await
        .unwrap();

        let invoice = repo::create_invoice(
            &pool,
            DocumentInput {
                id: None,
                client_id: 1,
                company_name: "Asgard Coffee & Co".into(),
                service_type: "service_bnc".into(),
                description: "Audit".into(),
                amount_ht: 1000.0,
                tva_rate: 20.0,
                date: "2026-04-18T10:00:00Z".into(),
                payment_terms_days: None,
                operation_kind: None,
                status: Some("payee".into()),
            },
        )
        .await
        .unwrap();

        sqlx::query("UPDATE invoices SET payment_date = ?1 WHERE id = ?2")
            .bind("2026-04-20T00:00:00Z")
            .bind(invoice.id)
            .execute(&pool)
            .await
            .unwrap();

        (pool, invoice.id)
    }

    /// Un avoir remboursé diminue l'encaissé — donc l'assiette déclarable.
    #[tokio::test]
    async fn a_refunded_credit_note_reduces_collected_revenue() {
        let (pool, invoice_id) = pool_with_a_paid_invoice().await;

        let before = asgard_core::reporting::revenue(&load_invoices(&pool).await.unwrap());
        assert_eq!(before.ht, asgard_core::from_f64(1000.0));

        repo::create_credit_note(
            &pool,
            CreditNoteInput {
                invoice_id,
                description: "Prestation annulée".into(),
                amount_ht: 400.0,
                date: "2026-05-02T00:00:00Z".into(),
                refunded_on: Some("2026-05-04T00:00:00Z".into()),
            },
        )
        .await
        .unwrap();

        let after = asgard_core::reporting::revenue(&load_invoices(&pool).await.unwrap());
        assert_eq!(after.ht, asgard_core::from_f64(600.0));
        assert_eq!(after.ht_facture, asgard_core::from_f64(600.0));
    }

    /// Tant que l'argent n'est pas reparti, l'avoir ne touche pas l'encaissé :
    /// c'est la règle de la comptabilité de trésorerie, en sens inverse.
    #[tokio::test]
    async fn an_unrefunded_credit_note_only_reduces_the_issued_basis() {
        let (pool, invoice_id) = pool_with_a_paid_invoice().await;

        repo::create_credit_note(
            &pool,
            CreditNoteInput {
                invoice_id,
                description: "Remise à valoir".into(),
                amount_ht: 400.0,
                date: "2026-05-02T00:00:00Z".into(),
                refunded_on: None,
            },
        )
        .await
        .unwrap();

        let revenue = asgard_core::reporting::revenue(&load_invoices(&pool).await.unwrap());
        assert_eq!(revenue.ht, asgard_core::from_f64(1000.0), "l'encaissé ne bouge pas");
        assert_eq!(revenue.ht_facture, asgard_core::from_f64(600.0), "le facturé, si");
    }

    /// Les cotisations suivent : on ne doit pas de charges sur une recette
    /// rendue.
    #[tokio::test]
    async fn charges_follow_the_credit_note() {
        let (pool, invoice_id) = pool_with_a_paid_invoice().await;
        let settings = asgard_core::model::Settings::default();

        let before = asgard_core::urssaf::total_charges(
            &load_invoices(&pool).await.unwrap(),
            &settings,
            asgard_core::urssaf::Basis::Collected,
        );

        repo::create_credit_note(
            &pool,
            CreditNoteInput {
                invoice_id,
                description: "Prestation annulée".into(),
                amount_ht: 1000.0,
                date: "2026-05-02T00:00:00Z".into(),
                refunded_on: Some("2026-05-04T00:00:00Z".into()),
            },
        )
        .await
        .unwrap();

        let after = asgard_core::urssaf::total_charges(
            &load_invoices(&pool).await.unwrap(),
            &settings,
            asgard_core::urssaf::Basis::Collected,
        );

        assert!(before > asgard_core::Money::ZERO);
        assert_eq!(after, asgard_core::Money::ZERO, "une recette entièrement rendue n'est pas cotisable");
    }

    /// Le livre des recettes doit porter l'avoir, avec son numéro et son signe.
    #[tokio::test]
    async fn the_revenue_book_lists_the_credit_note() {
        let (pool, invoice_id) = pool_with_a_paid_invoice().await;

        repo::create_credit_note(
            &pool,
            CreditNoteInput {
                invoice_id,
                description: "Prestation annulée".into(),
                amount_ht: 400.0,
                date: "2026-05-02T00:00:00Z".into(),
                refunded_on: Some("2026-05-04T00:00:00Z".into()),
            },
        )
        .await
        .unwrap();

        let csv = asgard_core::reporting::recettes_csv(&load_invoices(&pool).await.unwrap());

        assert!(csv.contains("AVO-ASGARDCOFF-2026-0001"), "CSV produit :\n{csv}");
        assert!(csv.contains("-400"), "le montant doit être négatif :\n{csv}");
    }

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
