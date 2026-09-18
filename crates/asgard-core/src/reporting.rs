//! Agrégats du tableau de bord, livre des recettes et export CSV.
//!
//! Porte `src/domain/reporting.js`. Une décision de fond au passage : les
//! séries mensuelles suivaient la date d'**émission** alors que le total annuel
//! affiché juste au-dessus suivait l'**encaissement**. Les deux chiffres ne se
//! recoupaient pas, sans que rien ne l'indique.
//!
//! Ici, chaque série dit sur quelle base elle est construite, via
//! [`urssaf::Basis`], et le tableau de bord expose les deux explicitement :
//! ce qui est facturé d'un côté, ce qui est encaissé de l'autre.

use serde::Serialize;

use crate::model::{Expense, Invoice, ServiceType, Settings, Status};
use crate::money::{format_csv, round_cents, Money};
use crate::urssaf::{self, Basis};
use rust_decimal_macros::dec;

pub const MONTH_LABELS: [&str; 12] = [
    "Jan", "Fév", "Mar", "Avr", "Mai", "Jun", "Jul", "Aoû", "Sep", "Oct", "Nov", "Déc",
];

/// Catégories de dépenses reconnues. Toute autre valeur bascule dans « Autre ».
pub const EXPENSE_CATEGORIES: [&str; 7] = [
    "Achats", "Déplacements", "Logiciels", "Télécoms", "Bureautique", "Cotisations", "Autre",
];

/// Plancher du maximum de l'axe des ordonnées du graphe mensuel.
const CHART_MIN_SCALE: Money = dec!(1000);

/// Marge appliquée aux extrema du graphe.
const CHART_HEADROOM: Money = dec!(1.15);

/// Chiffre d'affaires, facturé et encaissé.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Revenue {
    /// HT encaissé — la base déclarable.
    pub ht: Money,
    /// TTC encaissé.
    pub ttc: Money,
    /// HT facturé, brouillons exclus.
    pub ht_facture: Money,
    /// TTC facturé.
    pub ttc_facture: Money,
}

pub fn revenue(invoices: &[Invoice]) -> Revenue {
    let sum = |basis: Basis, field: fn(&Invoice) -> Money| {
        round_cents(
            invoices
                .iter()
                .filter(|i| basis.includes(i))
                .map(field)
                .sum::<Money>(),
        )
    };

    Revenue {
        ht: sum(Basis::Collected, |i| i.amount_ht),
        ttc: sum(Basis::Collected, |i| i.amount_total),
        ht_facture: sum(Basis::Issued, |i| i.amount_ht),
        ttc_facture: sum(Basis::Issued, |i| i.amount_total),
    }
}

pub fn total_expenses(expenses: &[Expense]) -> Money {
    round_cents(expenses.iter().map(|e| e.amount).sum::<Money>())
}

/// Répartition du CA encaissé par type d'activité, en pourcentage.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakdown {
    pub bnc: Money,
    pub bic: Money,
    pub vente: Money,
    pub total: Money,
}

pub fn breakdown(invoices: &[Invoice]) -> Breakdown {
    let sum_of = |service_type: ServiceType| {
        invoices
            .iter()
            .filter(|i| i.status == Status::Payee && i.service_type == service_type)
            .map(|i| i.amount_ht)
            .sum::<Money>()
    };

    let bnc = sum_of(ServiceType::ServiceBnc);
    let bic = sum_of(ServiceType::ServiceBic);
    let vente = sum_of(ServiceType::Vente);
    let total = bnc + bic + vente;

    // Un total nul renvoie zéro partout plutôt que NaN — le comportement
    // JavaScript, mais garanti par le type plutôt que par une garde oubliable.
    let pct = |part: Money| {
        if total.is_zero() {
            Money::ZERO
        } else {
            part / total * dec!(100)
        }
    };

    Breakdown {
        bnc: pct(bnc),
        bic: pct(bic),
        vente: pct(vente),
        total: round_cents(total),
    }
}

/// Séries mensuelles du graphe.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlySeries {
    pub labels: [&'static str; 12],
    pub ca_values: Vec<Money>,
    pub profit_values: Vec<Money>,
    pub max_val: Money,
    pub min_val: Money,
}

/// Construit les séries mensuelles d'un exercice.
///
/// `basis` dit explicitement ce qu'on trace : l'encaissé se recoupe avec le CA
/// du tableau de bord, le facturé avec la projection de trésorerie. L'ancienne
/// implémentation mélangeait les deux sans le dire.
pub fn monthly_series(
    invoices: &[Invoice],
    expenses: &[Expense],
    settings: &Settings,
    year: i32,
    basis: Basis,
) -> MonthlySeries {
    let mut ca = vec![Money::ZERO; 12];
    let mut charges = vec![Money::ZERO; 12];
    let mut spent = vec![Money::ZERO; 12];

    for invoice in invoices.iter().filter(|i| basis.includes(i)) {
        // L'encaissé se range au mois du règlement, le facturé au mois d'émission.
        let date = match basis {
            Basis::Collected => invoice.collected_on(),
            Basis::Issued => Some(invoice.date),
        };
        let Some(date) = date else { continue };
        if date.year != year {
            continue;
        }

        let index = date.month_index();
        ca[index] += invoice.amount_ht;
        charges[index] += urssaf::charges_for(invoice, settings);
    }

    for expense in expenses.iter().filter(|e| e.date.year == year) {
        spent[expense.date.month_index()] += expense.amount;
    }

    let ca_values: Vec<Money> = ca.iter().map(|v| round_cents(*v)).collect();
    let profit_values: Vec<Money> = (0..12)
        .map(|i| round_cents(ca[i] - spent[i] - charges[i]))
        .collect();

    let all = ca_values.iter().chain(profit_values.iter());
    let max = all.clone().copied().fold(CHART_MIN_SCALE, Money::max);
    let min = all.copied().fold(Money::ZERO, Money::min);

    MonthlySeries {
        labels: MONTH_LABELS,
        ca_values,
        profit_values,
        max_val: round_cents(max * CHART_HEADROOM),
        min_val: round_cents(min * CHART_HEADROOM),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorySlice {
    pub key: String,
    pub label: String,
    pub amount: Money,
    pub pct: Money,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseBreakdown {
    pub total: Money,
    pub list: Vec<CategorySlice>,
}

pub fn expenses_by_category(expenses: &[Expense]) -> ExpenseBreakdown {
    let mut totals: Vec<(&str, Money)> = EXPENSE_CATEGORIES
        .iter()
        .map(|c| (*c, Money::ZERO))
        .collect();

    let mut total = Money::ZERO;

    for expense in expenses {
        // Une catégorie inconnue bascule dans « Autre » plutôt que d'être perdue.
        let index = EXPENSE_CATEGORIES
            .iter()
            .position(|c| *c == expense.category)
            .unwrap_or(EXPENSE_CATEGORIES.len() - 1);

        totals[index].1 += expense.amount;
        total += expense.amount;
    }

    let list = totals
        .into_iter()
        .filter(|(_, amount)| !amount.is_zero())
        .map(|(key, amount)| CategorySlice {
            key: key.to_string(),
            label: key.to_string(),
            amount: round_cents(amount),
            pct: if total.is_zero() {
                Money::ZERO
            } else {
                amount / total * dec!(100)
            },
        })
        .collect();

    ExpenseBreakdown {
        total: round_cents(total),
        list,
    }
}

/// Livre des recettes : factures encaissées, par ordre chronologique.
///
/// `ascending` pour l'export réglementaire, l'inverse pour l'affichage.
pub fn recettes_ledger(invoices: &[Invoice], ascending: bool) -> Vec<&Invoice> {
    let mut ledger: Vec<&Invoice> = invoices
        .iter()
        .filter(|i| i.status == Status::Payee)
        .collect();

    ledger.sort_by_key(|i| (i.collected_on(), i.id));
    if !ascending {
        ledger.reverse();
    }
    ledger
}

/// Export CSV du livre des recettes.
///
/// Séparateur point-virgule, décimale virgule, BOM UTF-8 en tête pour qu'Excel
/// reconnaisse l'encodage — le format attendu par l'existant.
pub fn recettes_csv(invoices: &[Invoice]) -> String {
    let mut csv = String::from(
        "\u{FEFF}Date Encaissement;Facture;Client;Moyen de Paiement;Montant HT;Montant TTC\n",
    );

    for invoice in recettes_ledger(invoices, true) {
        let date = invoice
            .collected_on()
            .map(|d| d.format_fr())
            .unwrap_or_default();
        let method = invoice.payment_method.as_deref().unwrap_or("Virement");

        csv.push_str(&format!(
            "\"{}\";\"{}\";\"{}\";\"{}\";{};{}\n",
            date,
            invoice.invoice_number,
            invoice.company_name,
            method,
            format_csv(invoice.amount_ht),
            format_csv(invoice.amount_total),
        ));
    }

    csv
}

pub fn recettes_csv_filename(year: i32) -> String {
    format!("Livre_des_recettes_{year}.csv")
}

/// Tout ce que le tableau de bord affiche, en un seul calcul.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub revenue: Revenue,
    pub total_expenses: Money,
    /// Cotisations sur l'encaissé — la base déclarable.
    pub urssaf_charges: Money,
    /// Cotisations sur le facturé — projection, si tout rentre.
    pub urssaf_charges_projected: Money,
    pub net_profit: Money,
    pub breakdown: Breakdown,
    pub monthly: MonthlySeries,
    pub expenses_by_category: ExpenseBreakdown,
    pub alerts: Vec<crate::thresholds::Alert>,
    pub gauges: crate::thresholds::Gauges,
}

pub fn dashboard(
    invoices: &[Invoice],
    expenses: &[Expense],
    settings: &Settings,
    year: i32,
) -> Dashboard {
    let revenue = revenue(invoices);
    let total_expenses = total_expenses(expenses);
    let urssaf_charges = urssaf::total_charges(invoices, settings, Basis::Collected);

    Dashboard {
        net_profit: round_cents(revenue.ht - urssaf_charges - total_expenses),
        revenue,
        total_expenses,
        urssaf_charges,
        urssaf_charges_projected: urssaf::total_charges(invoices, settings, Basis::Issued),
        breakdown: breakdown(invoices),
        monthly: monthly_series(invoices, expenses, settings, year, Basis::Issued),
        expenses_by_category: expenses_by_category(expenses),
        alerts: crate::thresholds::alerts(invoices, year),
        gauges: crate::thresholds::gauges(invoices, year),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    #[test]
    fn separates_invoiced_from_collected() {
        let r = revenue(&fixtures::reference_invoices());

        assert_eq!(r.ht, dec!(19650.34));
        assert_eq!(r.ht_facture, dec!(24400.34));
        // L'écart est la facture émise mais non réglée.
        assert_eq!(r.ht_facture - r.ht, dec!(4750));
    }

    #[test]
    fn collected_ttc_is_exact_where_floats_drifted() {
        let r = revenue(&fixtures::reference_invoices());

        // En f64 ce total valait 22906.10725 : une valeur qu'aucune facture ne
        // peut porter, puisqu'on ne facture pas des fractions de centime.
        assert_eq!(r.ttc, dec!(22906.11));
    }

    #[test]
    fn sums_every_expense_across_exercises() {
        assert_eq!(total_expenses(&fixtures::reference_expenses()), dec!(1846.09));
    }

    #[test]
    fn breakdown_shares_add_up_to_one_hundred() {
        let b = breakdown(&fixtures::reference_invoices());

        assert_eq!(b.total, dec!(19650.34));
        assert_eq!(round_cents(b.bnc + b.bic + b.vente), dec!(100));
    }

    #[test]
    fn an_empty_portfolio_yields_zero_not_nan() {
        let b = breakdown(&[]);
        assert_eq!((b.bnc, b.bic, b.vente, b.total), (Money::ZERO, Money::ZERO, Money::ZERO, Money::ZERO));

        let r = revenue(&[]);
        assert_eq!(r.ht, Money::ZERO);
    }

    #[test]
    fn monthly_series_on_the_issued_basis_matches_the_old_chart() {
        let s = monthly_series(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
            Basis::Issued,
        );

        assert_eq!(
            s.ca_values,
            vec![
                dec!(0), dec!(8100), dec!(1899.99), dec!(1450.35), dec!(3200),
                dec!(4750), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0)
            ]
        );
    }

    /// Sur la base encaissée, la facture émise en décembre 2025 et réglée en
    /// janvier 2026 apparaît en janvier — ce que l'ancien graphe ne montrait
    /// nulle part, alors que le total annuel la comptait.
    #[test]
    fn the_collected_basis_places_a_straddling_invoice_in_january() {
        let s = monthly_series(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
            Basis::Collected,
        );

        assert_eq!(s.ca_values[0], dec!(5000));
        // Et la somme des mois se recoupe enfin avec le CA encaissé annuel.
        assert_eq!(
            s.ca_values.iter().sum::<Money>(),
            revenue(&fixtures::reference_invoices()).ht
        );
    }

    #[test]
    fn chart_bounds_apply_a_floor_and_headroom() {
        let empty = monthly_series(&[], &[], &fixtures::standard_settings(), 2026, Basis::Issued);

        assert_eq!(empty.max_val, dec!(1150));
        assert_eq!(empty.min_val, dec!(0));
    }

    #[test]
    fn unknown_expense_categories_fall_into_other() {
        let b = expenses_by_category(&fixtures::reference_expenses());

        let other = b.list.iter().find(|s| s.key == "Autre").unwrap();
        assert_eq!(other.amount, dec!(145));
        assert!(b.list.iter().all(|s| !s.amount.is_zero()));
    }

    #[test]
    fn ledger_keeps_only_collected_invoices_in_order() {
        let invoices = fixtures::reference_invoices();

        let ascending = recettes_ledger(&invoices, true);
        assert_eq!(ascending.len(), 5);
        assert_eq!(ascending.iter().map(|i| i.id).collect::<Vec<_>>(), [1, 2, 3, 4, 5]);

        let descending = recettes_ledger(&invoices, false);
        assert_eq!(descending.iter().map(|i| i.id).collect::<Vec<_>>(), [5, 4, 3, 2, 1]);
    }

    #[test]
    fn csv_carries_the_bom_and_the_regulatory_header() {
        let csv = recettes_csv(&fixtures::reference_invoices());

        assert!(csv.starts_with('\u{FEFF}'));
        assert_eq!(
            csv.lines().next().unwrap(),
            "\u{FEFF}Date Encaissement;Facture;Client;Moyen de Paiement;Montant HT;Montant TTC"
        );
    }

    #[test]
    fn csv_rows_follow_the_collection_order() {
        let csv = recettes_csv(&fixtures::reference_invoices());
        let rows: Vec<&str> = csv.lines().skip(1).collect();

        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows[0],
            "\"08/01/2026\";\"FAC-STARKINDUS-2025-0001\";\"Stark Industries\";\"virement\";5000,00;6000,00"
        );
        assert!(rows[2].contains("1899,99;2279,99"));
    }

    #[test]
    fn csv_defaults_a_missing_payment_method() {
        let mut invoice = fixtures::invoice(ServiceType::ServiceBnc, dec!(100), Status::Payee);
        invoice.payment_method = None;

        assert!(recettes_csv(&[invoice]).contains("\"Virement\""));
    }

    #[test]
    fn an_empty_ledger_yields_only_the_header() {
        assert_eq!(recettes_csv(&[]).lines().count(), 1);
    }

    #[test]
    fn the_dashboard_exposes_both_urssaf_bases() {
        let d = dashboard(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
        );

        assert_eq!(d.revenue.ht, dec!(19650.34));
        assert!(d.urssaf_charges_projected > d.urssaf_charges);
        assert_eq!(
            d.net_profit,
            round_cents(d.revenue.ht - d.urssaf_charges - d.total_expenses)
        );
    }
}
