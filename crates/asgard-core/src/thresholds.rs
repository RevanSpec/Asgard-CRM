//! Seuils de franchise en base de TVA et plafonds du régime micro-entreprise.
//!
//! Les montants étaient des littéraux dispersés dans le JSX — `App.jsx` pour les
//! alertes, `ComptaTab.jsx` pour les jauges — ce qui rendait impossible leur
//! mise à jour annuelle sans toucher au rendu, et garantissait qu'ils
//! divergeraient tôt ou tard.
//!
//! Ils sont ici **datés par exercice**. Les barèmes changent : la loi de
//! finances les révise périodiquement, et une facture de 2025 ne s'apprécie pas
//! au barème de 2027. `for_year` retient le barème connu le plus récent qui ne
//! soit pas postérieur à l'exercice demandé.

use serde::Serialize;

use crate::model::{Invoice, Status};
use crate::money::{round_cents, Money};
use rust_decimal_macros::dec;

/// Barème d'une famille d'activité pour un exercice.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    /// Seuil de la franchise en base de TVA.
    pub tva_limit: Money,
    /// Limite de tolérance : au-delà, la TVA devient due immédiatement.
    pub tva_tolerance: Money,
    /// Plafond du régime micro-entreprise.
    pub micro_limit: Money,
    /// Montant à partir duquel l'application alerte sur la TVA.
    pub tva_watch: Money,
    /// Montant à partir duquel l'application alerte sur le plafond.
    pub micro_watch: Money,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub year: i32,
    pub service: Limits,
    pub vente: Limits,
}

/// Barèmes connus, du plus ancien au plus récent.
///
/// Ajouter un exercice consiste à ajouter une entrée ici, et rien d'autre.
const SCHEDULES: &[Schedule] = &[Schedule {
    year: 2026,
    service: Limits {
        tva_limit: dec!(36800),
        tva_tolerance: dec!(39100),
        micro_limit: dec!(77700),
        tva_watch: dec!(34000),
        micro_watch: dec!(70000),
    },
    vente: Limits {
        tva_limit: dec!(91900),
        tva_tolerance: dec!(101000),
        micro_limit: dec!(188700),
        tva_watch: dec!(85000),
        micro_watch: dec!(170000),
    },
}];

/// Barème applicable à un exercice.
///
/// Retient le plus récent qui ne soit pas postérieur à l'année demandée ; pour
/// une année antérieure au premier barème connu, retourne celui-ci faute de
/// mieux — en le signalant par son champ `year`, qui ne correspondra pas.
pub fn for_year(year: i32) -> &'static Schedule {
    SCHEDULES
        .iter()
        .filter(|schedule| schedule.year <= year)
        .next_back()
        .unwrap_or(&SCHEDULES[0])
}

/// Chiffre d'affaires encaissé d'un exercice, ventilé entre services et ventes.
///
/// Tout ce qui n'est pas de la vente de marchandises compte comme service : les
/// BNC et les BIC de prestation partagent le même plafond.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnualCa {
    pub service: Money,
    pub vente: Money,
}

pub fn annual_ca(invoices: &[Invoice], year: i32) -> AnnualCa {
    let mut service = Money::ZERO;
    let mut vente = Money::ZERO;

    for invoice in invoices {
        if invoice.status != Status::Payee {
            continue;
        }
        let Some(date) = invoice.collected_on() else { continue };
        if date.year != year {
            continue;
        }

        if invoice.service_type.is_sale() {
            vente += invoice.amount_ht;
        } else {
            service += invoice.amount_ht;
        }
    }

    AnnualCa {
        service: round_cents(service),
        vente: round_cents(vente),
    }
}

/// Une jauge de l'onglet « Seuils de Chiffre d'Affaires ».
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauge {
    pub ca: Money,
    pub tva_limit: Money,
    pub tva_tolerance: Money,
    pub micro_limit: Money,
    /// Pourcentage de la tolérance TVA atteint, plafonné à 100.
    pub pct_tva: Money,
    /// Pourcentage du plafond micro atteint, plafonné à 100.
    pub pct_micro: Money,
    pub exceeded_tva: bool,
    pub remaining_before_tva: Money,
}

fn gauge(ca: Money, limits: &Limits) -> Gauge {
    let pct = |limit: Money| {
        if limit.is_zero() {
            Money::ZERO
        } else {
            (ca / limit * dec!(100)).min(dec!(100))
        }
    };

    Gauge {
        ca,
        tva_limit: limits.tva_limit,
        tva_tolerance: limits.tva_tolerance,
        micro_limit: limits.micro_limit,
        pct_tva: pct(limits.tva_tolerance),
        pct_micro: pct(limits.micro_limit),
        exceeded_tva: ca > limits.tva_limit,
        remaining_before_tva: limits.tva_tolerance - ca,
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauges {
    pub year: i32,
    pub service: Gauge,
    pub vente: Gauge,
}

pub fn gauges(invoices: &[Invoice], year: i32) -> Gauges {
    let ca = annual_ca(invoices, year);
    let schedule = for_year(year);

    Gauges {
        year,
        service: gauge(ca.service, &schedule.service),
        vente: gauge(ca.vente, &schedule.vente),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AlertLevel {
    Warning,
    Danger,
}

/// Alerte affichée en tête du tableau de bord.
///
/// Le texte est produit ici plutôt que dans le JSX : c'est du contenu
/// réglementaire, il change avec les barèmes, et le laisser dans le rendu
/// garantissait qu'il divergerait des montants réellement appliqués.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    #[serde(rename = "type")]
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
}

pub fn alerts(invoices: &[Invoice], year: i32) -> Vec<Alert> {
    use crate::money::format_fr as fr;

    let ca = annual_ca(invoices, year);
    let schedule = for_year(year);
    let mut out = Vec::new();

    // Services : alerte TVA, puis alerte de plafond. L'ordre est celui de
    // l'affichage historique et reste vérifié par un test.
    let s = &schedule.service;
    if ca.service > s.tva_watch {
        if ca.service > s.tva_tolerance {
            out.push(Alert {
                level: AlertLevel::Danger,
                title: "Seuil de TVA Services Dépassé".into(),
                message: format!(
                    "Votre CA annuel de services ({} €) a dépassé la limite de tolérance de la franchise en base de TVA ({} €). Vous devez facturer de la TVA.",
                    fr(ca.service), fr(s.tva_tolerance)
                ),
            });
        } else {
            out.push(Alert {
                level: AlertLevel::Warning,
                title: "Seuil de TVA Services Proche".into(),
                message: format!(
                    "Votre CA annuel de services ({} €) approche le seuil de la franchise en base de TVA ({} € / limite de tolérance : {} €).",
                    fr(ca.service), fr(s.tva_limit), fr(s.tva_tolerance)
                ),
            });
        }
    }

    if ca.service > s.micro_watch {
        out.push(Alert {
            level: AlertLevel::Warning,
            title: "Plafond Micro-Entreprise Services Proche".into(),
            message: format!(
                "Votre CA annuel de services ({} €) approche le plafond de la micro-entreprise ({} €).",
                fr(ca.service), fr(s.micro_limit)
            ),
        });
    }

    let v = &schedule.vente;
    if ca.vente > v.tva_watch {
        if ca.vente > v.tva_tolerance {
            out.push(Alert {
                level: AlertLevel::Danger,
                title: "Seuil de TVA Ventes Dépassé".into(),
                message: format!(
                    "Votre CA annuel de ventes ({} €) a dépassé la limite de tolérance de la franchise en base de TVA ({} €). Vous devez facturer de la TVA.",
                    fr(ca.vente), fr(v.tva_tolerance)
                ),
            });
        } else {
            out.push(Alert {
                level: AlertLevel::Warning,
                title: "Seuil de TVA Ventes Proche".into(),
                message: format!(
                    "Votre CA annuel de ventes ({} €) approche le seuil de la franchise en base de TVA ({} € / limite de tolérance : {} €).",
                    fr(ca.vente), fr(v.tva_limit), fr(v.tva_tolerance)
                ),
            });
        }
    }

    if ca.vente > v.micro_watch {
        out.push(Alert {
            level: AlertLevel::Warning,
            title: "Plafond Micro-Entreprise Ventes Proche".into(),
            message: format!(
                "Votre CA annuel de ventes ({} €) approche le plafond de la micro-entreprise ({} €).",
                fr(ca.vente), fr(v.micro_limit)
            ),
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn schedule_2026_matches_the_published_limits() {
        let s = for_year(2026);

        assert_eq!(s.service.tva_limit, dec!(36800));
        assert_eq!(s.service.tva_tolerance, dec!(39100));
        assert_eq!(s.service.micro_limit, dec!(77700));
        assert_eq!(s.vente.tva_limit, dec!(91900));
        assert_eq!(s.vente.tva_tolerance, dec!(101000));
        assert_eq!(s.vente.micro_limit, dec!(188700));
    }

    #[test]
    fn a_later_year_keeps_the_most_recent_known_schedule() {
        assert_eq!(for_year(2030).year, 2026);
        assert_eq!(for_year(2026).year, 2026);
    }

    #[test]
    fn an_earlier_year_falls_back_to_the_oldest_schedule() {
        assert_eq!(for_year(2019).year, 2026);
    }

    #[test]
    fn splits_annual_ca_between_services_and_sales() {
        let ca = annual_ca(&fixtures::reference_invoices(), 2026);

        assert_eq!(ca.service, dec!(14999.99));
        assert_eq!(ca.vente, dec!(4650.35));
    }

    #[test]
    fn annual_ca_follows_the_collection_year() {
        // La facture émise en décembre 2025 est encaissée en janvier 2026.
        let ca_2025 = annual_ca(&fixtures::reference_invoices(), 2025);

        assert_eq!(ca_2025.service, dec!(0));
        assert_eq!(ca_2025.vente, dec!(0));
    }

    #[test]
    fn gauges_cap_percentages_at_one_hundred() {
        let g = gauges(&fixtures::threshold_scenario("venteApproachingMicro"), 2026);

        assert_eq!(g.vente.ca, dec!(175000));
        assert_eq!(g.vente.pct_tva, dec!(100));
        assert!(g.vente.pct_micro < dec!(100));
    }

    #[test]
    fn gauges_report_the_remaining_margin() {
        let g = gauges(&fixtures::reference_invoices(), 2026);

        assert_eq!(g.service.ca, dec!(14999.99));
        assert!(!g.service.exceeded_tva);
        assert_eq!(g.service.remaining_before_tva, dec!(24100.01));
    }

    #[test]
    fn no_alert_while_every_trigger_is_below_its_watch_level() {
        assert!(alerts(&fixtures::reference_invoices(), 2026).is_empty());
    }

    #[test]
    fn warns_when_approaching_the_service_tva_threshold() {
        let a = alerts(&fixtures::threshold_scenario("serviceApproachingTva"), 2026);

        assert_eq!(a.len(), 1);
        assert_eq!(a[0].level, AlertLevel::Warning);
        assert_eq!(a[0].title, "Seuil de TVA Services Proche");
        assert!(a[0].message.contains("36\u{202F}800 €"));
    }

    #[test]
    fn escalates_past_the_service_tolerance() {
        let a = alerts(&fixtures::threshold_scenario("serviceExceededTva"), 2026);

        assert_eq!(a.len(), 1);
        assert_eq!(a[0].level, AlertLevel::Danger);
        assert!(a[0].message.contains("Vous devez facturer de la TVA."));
    }

    #[test]
    fn stacks_the_tva_and_ceiling_alerts() {
        let a = alerts(&fixtures::threshold_scenario("serviceApproachingMicro"), 2026);

        assert_eq!(
            a.iter().map(|x| x.title.as_str()).collect::<Vec<_>>(),
            ["Seuil de TVA Services Dépassé", "Plafond Micro-Entreprise Services Proche"]
        );
    }

    #[test]
    fn sale_alerts_use_their_own_limits() {
        let a = alerts(&fixtures::threshold_scenario("venteApproachingTva"), 2026);

        assert_eq!(a.len(), 1);
        assert_eq!(a[0].title, "Seuil de TVA Ventes Proche");
        assert!(a[0].message.contains("91\u{202F}900 €"));
    }

    /// L'ordre d'affichage est celui de l'implémentation historique : services
    /// d'abord, ventes ensuite.
    #[test]
    fn services_are_reported_before_sales() {
        let mut invoices = fixtures::threshold_scenario("serviceApproachingMicro");
        invoices.extend(fixtures::threshold_scenario("venteApproachingMicro"));

        assert_eq!(
            alerts(&invoices, 2026).iter().map(|a| a.title.as_str()).collect::<Vec<_>>(),
            [
                "Seuil de TVA Services Dépassé",
                "Plafond Micro-Entreprise Services Proche",
                "Seuil de TVA Ventes Dépassé",
                "Plafond Micro-Entreprise Ventes Proche",
            ]
        );
    }
}
