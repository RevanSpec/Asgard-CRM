//! Types du domaine.
//!
//! Les statuts et les types d'activité étaient des chaînes libres comparées à
//! la main (`inv.serviceType === 'service_bnc'`), répétées dans une douzaine
//! d'endroits. Une faute de frappe y passait inaperçue et faisait silencieusement
//! sortir une facture d'un agrégat. Ce sont désormais des énumérations : une
//! valeur inconnue se signale au moment de la désérialisation, pas six mois plus
//! tard dans une déclaration.

use serde::{Deserialize, Serialize};

use crate::money::{Money, Rate};

/// Type d'activité, qui détermine le taux de cotisation applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    /// Prestations de services libérales, relevant des BNC.
    ServiceBnc,
    /// Prestations de services artisanales ou commerciales, relevant des BIC.
    ServiceBic,
    /// Achat et revente de marchandises.
    Vente,
}

impl ServiceType {
    pub const ALL: [ServiceType; 3] = [
        ServiceType::ServiceBnc,
        ServiceType::ServiceBic,
        ServiceType::Vente,
    ];

    /// Les seuils et plafonds ne distinguent que deux familles : tout ce qui
    /// n'est pas de la vente de marchandises relève des services.
    pub fn is_sale(self) -> bool {
        matches!(self, ServiceType::Vente)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Brouillon,
    Envoyee,
    Payee,
}

/// Date civile, sans heure.
///
/// Les dates comptables n'ont pas d'heure, et en traîner une était une source
/// d'erreur : l'ancienne implémentation lisait l'année en heure locale tout en
/// construisant ses bornes en UTC, si bien qu'une facture du 1er janvier à
/// 00 h 30 sortait de la fenêtre de son propre exercice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CivilDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl CivilDate {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Lit une date ISO. L'heure, si elle est présente, est ignorée.
    pub fn parse(text: &str) -> Option<Self> {
        let date = text.get(..10)?;
        let mut parts = date.split('-');

        Some(Self {
            year: parts.next()?.parse().ok()?,
            month: parts.next()?.parse().ok()?,
            day: parts.next()?.parse().ok()?,
        })
    }

    /// Format français, tel qu'affiché dans les tableaux et le CSV.
    pub fn format_fr(self) -> String {
        format!("{:02}/{:02}/{}", self.day, self.month, self.year)
    }

    /// Index de mois de 0 à 11, pour les séries mensuelles.
    pub fn month_index(self) -> usize {
        (self.month.clamp(1, 12) - 1) as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub invoice_number: String,
    pub company_name: String,
    pub service_type: ServiceType,
    pub amount_ht: Money,
    pub tva_rate: Rate,
    pub amount_tva: Money,
    pub amount_total: Money,
    /// Date d'émission.
    pub date: CivilDate,
    pub status: Status,
    /// Date d'encaissement, absente tant que la facture n'est pas réglée.
    pub payment_date: Option<CivilDate>,
    pub payment_method: Option<String>,
}

impl Invoice {
    /// Date à retenir pour tout ce qui relève de l'encaissement : la date de
    /// règlement, avec repli sur la date d'émission quand elle manque.
    ///
    /// Le repli reproduit le comportement historique (`inv.paymentDate || inv.date`).
    pub fn collected_on(&self) -> Option<CivilDate> {
        if self.status != Status::Payee {
            return None;
        }
        Some(self.payment_date.unwrap_or(self.date))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: i64,
    pub date: CivilDate,
    pub merchant: String,
    pub category: String,
    pub amount: Money,
}

/// Réglages métier de l'utilisateur.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub urssaf_service_bnc: Rate,
    pub urssaf_service_bic: Rate,
    pub urssaf_vente: Rate,
    pub acre_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        use rust_decimal_macros::dec;

        Self {
            urssaf_service_bnc: dec!(21.1),
            urssaf_service_bic: dec!(21.1),
            urssaf_vente: dec!(12.3),
            acre_enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iso_dates_with_or_without_time() {
        assert_eq!(CivilDate::parse("2026-04-18"), Some(CivilDate::new(2026, 4, 18)));
        assert_eq!(
            CivilDate::parse("2026-04-18T10:00:00.000Z"),
            Some(CivilDate::new(2026, 4, 18))
        );
        assert_eq!(CivilDate::parse("pas une date"), None);
        assert_eq!(CivilDate::parse("2026-04"), None);
    }

    /// Une date civile n'a pas d'heure, donc pas de fuseau : le 1er janvier à
    /// 00 h 30 reste au 1er janvier, quel que soit le décalage. C'est ce que
    /// l'ancienne implémentation ne garantissait pas.
    #[test]
    fn a_civil_date_has_no_timezone_to_slip_on() {
        assert_eq!(
            CivilDate::parse("2026-01-01T00:30:00+01:00"),
            Some(CivilDate::new(2026, 1, 1))
        );
        assert_eq!(
            CivilDate::parse("2025-12-31T23:30:00Z"),
            Some(CivilDate::new(2025, 12, 31))
        );
    }

    #[test]
    fn formats_dates_the_french_way() {
        assert_eq!(CivilDate::new(2026, 1, 8).format_fr(), "08/01/2026");
        assert_eq!(CivilDate::new(2026, 12, 31).format_fr(), "31/12/2026");
    }

    #[test]
    fn collection_date_falls_back_to_the_issue_date() {
        use rust_decimal_macros::dec;
        use crate::fixtures;

        let mut invoice = fixtures::invoice(ServiceType::ServiceBnc, dec!(100), Status::Payee);
        invoice.date = CivilDate::new(2026, 3, 12);
        invoice.payment_date = None;

        assert_eq!(invoice.collected_on(), Some(CivilDate::new(2026, 3, 12)));

        invoice.payment_date = Some(CivilDate::new(2026, 3, 30));
        assert_eq!(invoice.collected_on(), Some(CivilDate::new(2026, 3, 30)));
    }

    #[test]
    fn an_unpaid_invoice_has_no_collection_date() {
        use rust_decimal_macros::dec;
        use crate::fixtures;

        let invoice = fixtures::invoice(ServiceType::ServiceBnc, dec!(100), Status::Envoyee);
        assert_eq!(invoice.collected_on(), None);
    }

    #[test]
    fn only_sales_are_sales() {
        assert!(ServiceType::Vente.is_sale());
        assert!(!ServiceType::ServiceBnc.is_sale());
        assert!(!ServiceType::ServiceBic.is_sale());
    }
}
