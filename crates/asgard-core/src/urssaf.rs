//! Cotisations sociales du micro-entrepreneur — correction définitive du défaut D8.
//!
//! Le calcul existait en **trois exemplaires** dans le code JavaScript :
//! `App.jsx · calculateUrssafCharges`, `App.jsx · getMonthlyFinancialsData`, et
//! une IIFE au milieu du JSX de `ComptaTab.jsx`. La phase 0 les avait
//! rassemblés dans `src/domain/urssaf.js` en préservant leur divergence pour
//! pouvoir la prouver ; ici elle est tranchée.
//!
//! **La divergence, et la décision.** Le graphe mensuel calculait les
//! cotisations sur les factures « non brouillon » — donc émises, même impayées —
//! tandis que le total annuel affiché juste au-dessus ne comptait que
//! l'encaissé. Sur le jeu de référence l'écart atteignait 1 002,25 €, et
//! l'utilisateur voyait les deux chiffres côte à côte.
//!
//! Le micro-entrepreneur déclare son chiffre d'affaires **réellement encaissé**
//! (art. L613-7 du code de la sécurité sociale) : c'est l'encaissé qui fait foi.
//! `Basis::Collected` est donc la règle, et `Basis::Issued` reste disponible
//! pour ce qu'elle est réellement — une projection de trésorerie, nommée comme
//! telle plutôt que confondue avec une base déclarative.

use serde::{Deserialize, Serialize};

use crate::model::{Invoice, ServiceType, Settings, Status};
use crate::money::{percent_of, round_cents, Money, Rate};

/// Abattement ACRE : les taux sont divisés par deux la première année.
pub const ACRE_DIVISOR: i64 = 2;

/// Sur quelles factures asseoir un calcul.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Basis {
    /// Factures encaissées. **La seule base déclarable à l'URSSAF.**
    Collected,
    /// Factures émises, réglées ou non. Projection de trésorerie : ce que
    /// coûteront les cotisations si tout ce qui est facturé rentre.
    Issued,
}

impl Basis {
    pub fn includes(self, invoice: &Invoice) -> bool {
        match self {
            Basis::Collected => invoice.status == Status::Payee,
            Basis::Issued => invoice.status != Status::Brouillon,
        }
    }
}

/// Taux brut d'un type d'activité, avant ACRE.
pub fn base_rate(service_type: ServiceType, settings: &Settings) -> Rate {
    match service_type {
        ServiceType::ServiceBnc => settings.urssaf_service_bnc,
        ServiceType::ServiceBic => settings.urssaf_service_bic,
        ServiceType::Vente => settings.urssaf_vente,
    }
}

/// Taux effectivement appliqué, ACRE comprise.
pub fn effective_rate(service_type: ServiceType, settings: &Settings) -> Rate {
    let rate = base_rate(service_type, settings);
    if settings.acre_enabled {
        rate / Rate::from(ACRE_DIVISOR)
    } else {
        rate
    }
}

/// Cotisations dues sur une facture, assises sur le montant HT.
pub fn charges_for(invoice: &Invoice, settings: &Settings) -> Money {
    percent_of(invoice.amount_ht, effective_rate(invoice.service_type, settings))
}

/// Total des cotisations sur une base donnée.
///
/// Les cotisations sont sommées **avant** d'être arrondies : arrondir chaque
/// ligne puis additionner fait dériver le total de quelques centimes sur un
/// exercice entier.
pub fn total_charges(invoices: &[Invoice], settings: &Settings, basis: Basis) -> Money {
    let raw = invoices
        .iter()
        .filter(|invoice| basis.includes(invoice))
        .map(|invoice| charges_for(invoice, settings))
        .sum::<Money>();

    round_cents(raw)
}

/// Période de déclaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "periodType")]
pub enum Period {
    Monthly { year: i32, month: u32 },
    Quarterly { year: i32, quarter: u32 },
}

impl Period {
    pub fn year(self) -> i32 {
        match self {
            Period::Monthly { year, .. } | Period::Quarterly { year, .. } => year,
        }
    }

    /// Une facture encaissée tombe-t-elle dans cette période ?
    ///
    /// La date retenue est celle de l'**encaissement**, avec repli sur la date
    /// d'émission quand elle manque.
    pub fn contains(self, invoice: &Invoice) -> bool {
        if invoice.status != Status::Payee {
            return false;
        }

        let Some(date) = invoice.collected_on() else {
            return false;
        };

        match self {
            Period::Monthly { year, month } => date.year == year && date.month == month,
            Period::Quarterly { year, quarter } => {
                date.year == year && quarter_of(date.month) == quarter
            }
        }
    }
}

pub fn quarter_of(month: u32) -> u32 {
    (month.saturating_sub(1)) / 3 + 1
}

/// Une ligne de la déclaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclarationLine {
    pub ht: Money,
    pub rate: Rate,
    pub charges: Money,
}

/// Déclaration d'une période : chiffre d'affaires encaissé et cotisations,
/// ventilés par type d'activité.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Declaration {
    pub bnc: DeclarationLine,
    pub bic: DeclarationLine,
    pub vente: DeclarationLine,
    pub total_ca: Money,
    pub total_charges: Money,
}

pub fn declaration(invoices: &[Invoice], settings: &Settings, period: Period) -> Declaration {
    let line = |service_type: ServiceType| {
        let ht: Money = invoices
            .iter()
            .filter(|invoice| invoice.service_type == service_type && period.contains(invoice))
            .map(|invoice| invoice.amount_ht)
            .sum();

        let rate = effective_rate(service_type, settings);

        DeclarationLine {
            ht: round_cents(ht),
            rate,
            charges: round_cents(percent_of(ht, rate)),
        }
    };

    let bnc = line(ServiceType::ServiceBnc);
    let bic = line(ServiceType::ServiceBic);
    let vente = line(ServiceType::Vente);

    Declaration {
        total_ca: bnc.ht + bic.ht + vente.ht,
        total_charges: bnc.charges + bic.charges + vente.charges,
        bnc,
        bic,
        vente,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use rust_decimal_macros::dec;

    #[test]
    fn rates_follow_the_activity_type() {
        let settings = fixtures::standard_settings();

        assert_eq!(base_rate(ServiceType::ServiceBnc, &settings), dec!(21.1));
        assert_eq!(base_rate(ServiceType::ServiceBic, &settings), dec!(21.1));
        assert_eq!(base_rate(ServiceType::Vente, &settings), dec!(12.3));
    }

    #[test]
    fn acre_halves_every_rate() {
        let settings = fixtures::acre_settings();

        assert_eq!(effective_rate(ServiceType::ServiceBnc, &settings), dec!(10.55));
        assert_eq!(effective_rate(ServiceType::Vente, &settings), dec!(6.15));
    }

    #[test]
    fn charges_are_assessed_on_the_net_amount() {
        let settings = fixtures::standard_settings();
        let invoice = fixtures::invoice(ServiceType::ServiceBnc, dec!(8100), Status::Payee);

        // 8 100 × 21,1 % = 1 709,10 — et non sur le TTC.
        assert_eq!(round_cents(charges_for(&invoice, &settings)), dec!(1709.10));
    }

    #[test]
    fn quarters_split_the_year_in_four() {
        assert_eq!([1, 2, 3].map(quarter_of), [1, 1, 1]);
        assert_eq!([4, 5, 6].map(quarter_of), [2, 2, 2]);
        assert_eq!([7, 8, 9].map(quarter_of), [3, 3, 3]);
        assert_eq!([10, 11, 12].map(quarter_of), [4, 4, 4]);
    }

    #[test]
    fn only_collected_invoices_are_declarable() {
        let invoices = fixtures::reference_invoices();
        let settings = fixtures::standard_settings();

        let collected = total_charges(&invoices, &settings, Basis::Collected);
        let issued = total_charges(&invoices, &settings, Basis::Issued);

        assert!(issued > collected);
        assert_eq!(issued - collected, dec!(1002.25));
    }

    /// La divergence que la phase 0 avait figée est désormais nommée : ce ne
    /// sont plus deux calculs qui se contredisent, mais deux questions
    /// différentes.
    #[test]
    fn the_two_bases_answer_different_questions() {
        let unpaid = fixtures::invoice(ServiceType::ServiceBnc, dec!(4750), Status::Envoyee);

        assert!(!Basis::Collected.includes(&unpaid));
        assert!(Basis::Issued.includes(&unpaid));

        let draft = fixtures::invoice(ServiceType::ServiceBnc, dec!(1000), Status::Brouillon);
        assert!(!Basis::Collected.includes(&draft));
        assert!(!Basis::Issued.includes(&draft));
    }

    #[test]
    fn a_period_follows_the_collection_date_not_the_issue_date() {
        let invoices = fixtures::reference_invoices();
        let settings = fixtures::standard_settings();

        // Facture émise le 15/12/2025, encaissée le 08/01/2026.
        let january = declaration(&invoices, &settings, Period::Monthly { year: 2026, month: 1 });
        let december = declaration(&invoices, &settings, Period::Monthly { year: 2025, month: 12 });

        assert_eq!(january.bnc.ht, dec!(5000));
        assert_eq!(december.total_ca, dec!(0));
    }

    #[test]
    fn declares_the_first_quarter_of_2026() {
        let invoices = fixtures::reference_invoices();
        let settings = fixtures::standard_settings();

        let q1 = declaration(&invoices, &settings, Period::Quarterly { year: 2026, quarter: 1 });

        assert_eq!(q1.bnc.ht, dec!(13100));
        assert_eq!(q1.bic.ht, dec!(1899.99));
        assert_eq!(q1.vente.ht, dec!(0));
        assert_eq!(q1.total_ca, dec!(14999.99));
    }

    #[test]
    fn an_invoice_lands_in_the_quarter_of_its_collection() {
        let invoices = fixtures::reference_invoices();
        let settings = fixtures::standard_settings();

        // Facture 5 : émise en mai (T2), encaissée en juillet (T3).
        let q2 = declaration(&invoices, &settings, Period::Quarterly { year: 2026, quarter: 2 });
        let q3 = declaration(&invoices, &settings, Period::Quarterly { year: 2026, quarter: 3 });

        assert_eq!(q2.vente.ht, dec!(1450.35));
        assert_eq!(q3.vente.ht, dec!(3200));
    }

    #[test]
    fn an_empty_period_declares_zero() {
        let invoices = fixtures::reference_invoices();
        let settings = fixtures::standard_settings();

        let november = declaration(&invoices, &settings, Period::Monthly { year: 2026, month: 11 });

        assert_eq!(november.total_ca, dec!(0));
        assert_eq!(november.total_charges, dec!(0));
        assert_eq!(november.bnc.rate, dec!(21.1));
    }

    #[test]
    fn acre_halves_the_declared_charges() {
        let invoices = fixtures::reference_invoices();
        let full = declaration(
            &invoices,
            &fixtures::standard_settings(),
            Period::Quarterly { year: 2026, quarter: 1 },
        );
        let acre = declaration(
            &invoices,
            &fixtures::acre_settings(),
            Period::Quarterly { year: 2026, quarter: 1 },
        );

        assert_eq!(acre.total_ca, full.total_ca);
        assert_eq!(acre.bnc.rate, dec!(10.55));
        // Le total ACRE vaut la moitié, à l'arrondi par ligne près.
        assert!((full.total_charges / dec!(2) - acre.total_charges).abs() <= dec!(0.02));
    }
}
