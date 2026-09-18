//! Jeu de référence de la phase 0, lu depuis Rust.
//!
//! `fixtures/reference-dataset.json` a été figé en phase 0 pour servir d'oracle.
//! Les golden tests JavaScript l'utilisent ; ces modules l'utilisent aussi.
//! **C'est le même fichier**, et c'est tout l'intérêt : la parité entre les deux
//! implémentations se démontre sur des données identiques, pas sur deux jeux
//! qu'on croit équivalents.
//!
//! Le fichier est embarqué à la compilation. S'il change, les tests des deux
//! côtés bougent ensemble.

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde_json::Value;

use crate::model::{CivilDate, Expense, Invoice, ServiceType, Settings, Status};
use crate::money::from_f64;

const DATASET: &str = include_str!("../../../fixtures/reference-dataset.json");

fn dataset() -> Value {
    serde_json::from_str(DATASET).expect("le jeu de référence doit rester du JSON valide")
}

fn money(value: &Value, key: &str) -> Decimal {
    from_f64(value.get(key).and_then(Value::as_f64).unwrap_or(0.0))
}

fn text(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or_default().to_string()
}

fn date(value: &Value, key: &str) -> Option<CivilDate> {
    value.get(key).and_then(Value::as_str).and_then(CivilDate::parse)
}

fn service_type(value: &Value) -> ServiceType {
    match value.get("serviceType").and_then(Value::as_str) {
        Some("service_bic") => ServiceType::ServiceBic,
        Some("vente") => ServiceType::Vente,
        _ => ServiceType::ServiceBnc,
    }
}

fn status(value: &Value) -> Status {
    match value.get("status").and_then(Value::as_str) {
        Some("payee") => Status::Payee,
        Some("envoyee") => Status::Envoyee,
        _ => Status::Brouillon,
    }
}

fn parse_invoice(value: &Value) -> Invoice {
    Invoice {
        id: value.get("id").and_then(Value::as_i64).unwrap_or(0),
        invoice_number: text(value, "invoiceNumber"),
        company_name: text(value, "companyName"),
        service_type: service_type(value),
        amount_ht: money(value, "amountHt"),
        tva_rate: money(value, "tvaRate"),
        amount_tva: money(value, "amountTva"),
        amount_total: money(value, "amountTotal"),
        date: date(value, "date").unwrap_or(CivilDate::new(2026, 1, 1)),
        status: status(value),
        payment_date: date(value, "paymentDate"),
        payment_method: value
            .get("paymentMethod")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

/// Les sept factures du jeu de référence.
pub fn reference_invoices() -> Vec<Invoice> {
    dataset()["invoices"]
        .as_array()
        .expect("le jeu de référence doit contenir des factures")
        .iter()
        .map(parse_invoice)
        .collect()
}

/// Les huit dépenses du jeu de référence, exercices 2025 et 2026 confondus.
pub fn reference_expenses() -> Vec<Expense> {
    dataset()["expenses"]
        .as_array()
        .expect("le jeu de référence doit contenir des dépenses")
        .iter()
        .map(|value| Expense {
            id: value.get("id").and_then(Value::as_i64).unwrap_or(0),
            date: date(value, "date").unwrap_or(CivilDate::new(2026, 1, 1)),
            merchant: text(value, "merchant"),
            category: text(value, "category"),
            amount: money(value, "amount"),
        })
        .collect()
}

fn parse_settings(key: &str) -> Settings {
    let data = dataset();
    let value = &data["settings"][key];

    Settings {
        urssaf_service_bnc: money(value, "urssafServiceBnc"),
        urssaf_service_bic: money(value, "urssafServiceBic"),
        urssaf_vente: money(value, "urssafVente"),
        acre_enabled: value
            .get("acreEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

pub fn standard_settings() -> Settings {
    parse_settings("standard")
}

pub fn acre_settings() -> Settings {
    parse_settings("acre")
}

/// Scénarios réduits de dépassement de seuil, un par branche d'alerte.
pub fn threshold_scenario(name: &str) -> Vec<Invoice> {
    dataset()["thresholdScenarios"][name]
        .as_array()
        .unwrap_or_else(|| panic!("scénario de seuil inconnu : {name}"))
        .iter()
        .map(parse_invoice)
        .collect()
}

/// Facture minimale, pour les tests qui n'ont besoin que d'un montant et d'un
/// statut.
pub fn invoice(service_type: ServiceType, amount_ht: Decimal, status: Status) -> Invoice {
    Invoice {
        id: 1,
        invoice_number: "FAC-TEST-2026-0001".into(),
        company_name: "Test".into(),
        service_type,
        amount_ht,
        tva_rate: dec!(20),
        amount_tva: amount_ht * dec!(0.2),
        amount_total: amount_ht * dec!(1.2),
        date: CivilDate::new(2026, 3, 12),
        status,
        payment_date: if status == Status::Payee {
            Some(CivilDate::new(2026, 3, 12))
        } else {
            None
        },
        payment_method: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_the_phase_zero_dataset() {
        let invoices = reference_invoices();

        assert_eq!(invoices.len(), 7);
        assert_eq!(invoices[0].invoice_number, "FAC-STARKINDUS-2025-0001");
        assert_eq!(invoices[0].amount_ht, dec!(5000));
        assert_eq!(invoices[0].status, Status::Payee);
        assert_eq!(invoices[0].date, CivilDate::new(2025, 12, 15));
        assert_eq!(invoices[0].payment_date, Some(CivilDate::new(2026, 1, 8)));
    }

    #[test]
    fn covers_every_activity_type_and_status() {
        let invoices = reference_invoices();

        for service_type in ServiceType::ALL {
            assert!(
                invoices.iter().any(|i| i.service_type == service_type),
                "le jeu de référence doit couvrir {service_type:?}"
            );
        }

        for status in [Status::Brouillon, Status::Envoyee, Status::Payee] {
            assert!(
                invoices.iter().any(|i| i.status == status),
                "le jeu de référence doit couvrir {status:?}"
            );
        }
    }

    #[test]
    fn loads_both_settings_variants() {
        assert!(!standard_settings().acre_enabled);
        assert!(acre_settings().acre_enabled);
        assert_eq!(standard_settings().urssaf_vente, dec!(12.3));
    }

    #[test]
    fn loads_expenses_across_two_exercises() {
        let expenses = reference_expenses();

        assert_eq!(expenses.len(), 8);
        assert!(expenses.iter().any(|e| e.date.year == 2025));
        assert!(expenses.iter().any(|e| e.category == "Mobilier"));
    }

    #[test]
    fn loads_every_threshold_scenario() {
        for name in [
            "serviceApproachingTva",
            "serviceExceededTva",
            "serviceApproachingMicro",
            "venteApproachingTva",
            "venteExceededTva",
            "venteApproachingMicro",
        ] {
            assert_eq!(threshold_scenario(name).len(), 1, "scénario {name}");
        }
    }

    /// Les montants non ronds du jeu traversent la conversion sans artefact :
    /// c'est ce que la phase 0 ne pouvait pas garantir.
    #[test]
    fn non_round_amounts_survive_the_conversion_exactly() {
        let invoices = reference_invoices();

        let wayne = invoices.iter().find(|i| i.amount_ht == dec!(1899.99)).unwrap();
        assert_eq!(wayne.amount_ht, dec!(1899.99));

        let coffee = invoices.iter().find(|i| i.amount_ht == dec!(1450.35)).unwrap();
        assert_eq!(coffee.tva_rate, dec!(5.5));
    }
}
