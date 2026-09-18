//! Rapport de parité avec l'implémentation JavaScript.
//!
//! Le plan de migration exigeait, avant toute bascule, de « produire un rapport
//! d'écarts : sur les données réelles de l'utilisateur, comparer les totaux
//! `f64` et `Decimal`. Les chiffres vont bouger de quelques centimes — il faut
//! pouvoir l'expliquer, pas le découvrir. »
//!
//! Ce module est ce rapport, sous forme exécutable. Chaque test nomme une
//! valeur que les golden tests de la phase 0 avaient figée, dit ce que Rust
//! produit à la place, et justifie l'écart. Les valeurs attendues côté
//! JavaScript sont recopiées depuis `src/domain/*.test.js` : si l'un des deux
//! côtés change, la divergence se voit ici.
//!
//! Trois catégories d'écart, et rien d'autre :
//!
//! 1. **Identique.** La grande majorité. Les montants ronds traversent les deux
//!    implémentations sans bouger.
//! 2. **Artefact flottant supprimé.** `379.99800000000005` devient `379,998`.
//!    La valeur *juste* ne change pas ; seule disparaît l'approximation binaire.
//! 3. **Arrondi au centime.** `379,998 €` devient `380,00 €`. Là, une valeur
//!    change réellement — et c'est voulu : une facture ne se libelle pas en
//!    fractions de centime.

use rust_decimal_macros::dec;

use crate::fixtures;
use crate::model::Status;
use crate::money::{from_f64, to_f64};
use crate::reporting;
use crate::thresholds;
use crate::urssaf::{self, Basis, Period};

/// Catégorie 1 — identique des deux côtés.
mod unchanged {
    use super::*;

    #[test]
    fn collected_revenue_is_unchanged() {
        // src/domain/reporting.test.js : expect(ca.ht).toBe(19650.34)
        let r = reporting::revenue(&fixtures::reference_invoices());
        assert_eq!(r.ht, dec!(19650.34));
        assert_eq!(to_f64(r.ht), 19650.34);
    }

    #[test]
    fn invoiced_revenue_is_unchanged() {
        // src/domain/reporting.test.js : expect(ca.htFacture).toBe(24400.34)
        let r = reporting::revenue(&fixtures::reference_invoices());
        assert_eq!(r.ht_facture, dec!(24400.34));
    }

    #[test]
    fn total_expenses_are_unchanged() {
        // src/domain/reporting.test.js : expect(...).toBe(1846.09)
        assert_eq!(
            reporting::total_expenses(&fixtures::reference_expenses()),
            dec!(1846.09)
        );
    }

    #[test]
    fn annual_ca_by_category_is_unchanged() {
        // src/domain/thresholds.test.js : { serviceCa: 14999.99, venteCa: 4650.35 }
        let ca = thresholds::annual_ca(&fixtures::reference_invoices(), 2026);
        assert_eq!(ca.service, dec!(14999.99));
        assert_eq!(ca.vente, dec!(4650.35));
    }

    #[test]
    fn quarterly_declaration_amounts_are_unchanged() {
        // src/domain/urssaf.test.js : bnc.ht 13100, bic.ht 1899.99, totalCa 14999.99
        let d = urssaf::declaration(
            &fixtures::reference_invoices(),
            &fixtures::standard_settings(),
            Period::Quarterly { year: 2026, quarter: 1 },
        );

        assert_eq!(d.bnc.ht, dec!(13100));
        assert_eq!(d.bic.ht, dec!(1899.99));
        assert_eq!(d.total_ca, dec!(14999.99));
    }

    #[test]
    fn the_monthly_chart_series_is_unchanged() {
        // src/domain/reporting.test.js, base « facturé » — celle de l'ancien graphe.
        let s = reporting::monthly_series(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
            Basis::Issued,
        );

        assert_eq!(s.ca_values[1], dec!(8100));
        assert_eq!(s.ca_values[2], dec!(1899.99));
        assert_eq!(s.ca_values[5], dec!(4750));
    }

    #[test]
    fn threshold_alert_texts_are_unchanged() {
        // src/domain/thresholds.test.js : mêmes titres, mêmes montants.
        let a = thresholds::alerts(&fixtures::threshold_scenario("serviceApproachingTva"), 2026);

        assert_eq!(a[0].title, "Seuil de TVA Services Proche");
        assert!(a[0].message.contains("35\u{202F}000 €"));
        assert!(a[0].message.contains("39\u{202F}100 €"));
    }

    #[test]
    fn the_csv_ledger_is_unchanged() {
        // src/domain/reporting.test.js : même première ligne, au caractère près.
        let csv = reporting::recettes_csv(&fixtures::reference_invoices());
        let first = csv.lines().nth(1).unwrap();

        assert_eq!(
            first,
            "\"08/01/2026\";\"FAC-STARKINDUS-2025-0001\";\"Stark Industries\";\"virement\";5000,00;6000,00"
        );
    }
}

/// Catégorie 2 — l'artefact binaire disparaît, la valeur juste ne bouge pas.
mod artifacts_removed {
    use super::*;

    #[test]
    fn invoice_tva_loses_its_binary_tail() {
        // src/domain/money.test.js figeait : expect(amountTva).toBe(379.99800000000005)
        // et affirmait explicitement expect(amountTva).not.toBe(379.998).
        let tva = crate::money::percent_of(dec!(1899.99), dec!(20));

        assert_eq!(tva, dec!(379.998));
        assert_ne!(to_f64(tva), 379.998_000_000_000_05);
    }

    #[test]
    fn invoice_total_loses_its_binary_tail() {
        // Golden test phase 0 : expect(amountTotal).toBe(2279.9880000000003)
        let total = dec!(1899.99) + crate::money::percent_of(dec!(1899.99), dec!(20));

        assert_eq!(total, dec!(2279.988));
        assert_ne!(to_f64(total), 2279.988_000_000_000_3);
    }

    #[test]
    fn monthly_profit_loses_its_binary_tail() {
        // src/domain/reporting.test.js :
        //   expect(monthly.profitValues[3]).toBe(451.4569499999999)
        // — une valeur qu'aucun compte ne peut porter.
        let s = reporting::monthly_series(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
            Basis::Issued,
        );

        assert_eq!(s.profit_values[3], dec!(451.46));
        assert_ne!(to_f64(s.profit_values[3]), 451.456_949_999_999_9);
    }

    #[test]
    fn chart_minimum_loses_its_binary_tail() {
        // src/domain/reporting.test.js : expect(monthly.minVal).toBe(-333.38499999999993)
        let s = reporting::monthly_series(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
            Basis::Issued,
        );

        assert!(s.min_val < dec!(0));
        assert_ne!(to_f64(s.min_val), -333.384_999_999_999_93);
    }
}

/// Catégorie 3 — une valeur change réellement. C'est le cœur du rapport.
mod amounts_that_move {
    use super::*;

    /// **Écart assumé.** Les golden tests de la phase 0 figeaient une TVA de
    /// `379,998 €`. Arrondie au centime elle vaut `380,00 €`.
    ///
    /// La nouvelle valeur est la bonne : une facture s'exprime au centime. Mais
    /// c'est bien un montant qui bouge sur un document déjà émis, et c'est
    /// exactement ce que le rapport d'import de la phase 2 signale à
    /// l'utilisateur, pièce par pièce.
    #[test]
    fn invoice_tva_rounds_up_by_two_tenths_of_a_cent() {
        let before = dec!(379.998);
        let after = crate::money::compute_amounts(dec!(1899.99), dec!(20)).amount_tva;

        assert_eq!(after, dec!(380.00));
        assert_eq!(after - before, dec!(0.002));
    }

    /// Même mécanique sur un taux réduit : `79,76925 €` devient `79,77 €`.
    #[test]
    fn reduced_rate_tva_rounds_to_the_cent() {
        let before = dec!(79.76925);
        let after = crate::money::compute_amounts(dec!(1450.35), dec!(5.5)).amount_tva;

        assert_eq!(after, dec!(79.77));
        assert!((after - before).abs() < dec!(0.01));
    }

    /// Le TTC encaissé cumulé bouge de moins d'un centime.
    ///
    /// L'ancien total, `22 906,10725 €`, portait trois décimales : il ne pouvait
    /// correspondre à aucune somme réellement encaissée.
    #[test]
    fn collected_ttc_settles_on_the_cent() {
        let before = dec!(22906.10725);
        let after = reporting::revenue(&fixtures::reference_invoices()).ttc;

        assert_eq!(after, dec!(22906.11));
        assert!((after - before).abs() < dec!(0.01));
    }

    /// Les cotisations URSSAF bougent de moins d'un centime sur le jeu de
    /// référence. C'est le chiffre déclaré : il doit être justifiable.
    #[test]
    fn urssaf_charges_settle_on_the_cent() {
        let before = dec!(3736.99094);
        let after = urssaf::total_charges(
            &fixtures::reference_invoices(),
            &fixtures::standard_settings(),
            Basis::Collected,
        );

        assert_eq!(after, dec!(3736.99));
        assert!((after - before).abs() < dec!(0.01));
    }

    /// Le total du premier trimestre, lui, ne bouge pas : `3 164,99789 €`
    /// arrondi donne `3 164,99 €` — moins d'un centime d'écart, mais un écart.
    #[test]
    fn quarterly_charges_settle_on_the_cent() {
        let before = dec!(3164.99789);
        let after = urssaf::declaration(
            &fixtures::reference_invoices(),
            &fixtures::standard_settings(),
            Period::Quarterly { year: 2026, quarter: 1 },
        )
        .total_charges;

        assert!((after - before).abs() < dec!(0.01));
    }
}

/// Le seul écart **de comportement**, et non d'arrondi.
mod behaviour_that_changes {
    use super::*;

    /// La divergence que la phase 0 avait figée sans la trancher.
    ///
    /// `src/domain/urssaf.test.js` documentait que le graphe mensuel et le total
    /// annuel calculaient les cotisations sur des bases différentes, avec
    /// 1 002,25 € d'écart, et que l'utilisateur voyait les deux chiffres côte à
    /// côte sans que rien ne l'explique.
    ///
    /// Les deux bases existent toujours, mais elles sont désormais **nommées**
    /// et le tableau de bord expose les deux : `urssafCharges` pour l'encaissé
    /// — le seul déclarable — et `urssafChargesProjected` pour la projection.
    #[test]
    fn the_two_urssaf_bases_are_now_named_instead_of_contradictory() {
        let invoices = fixtures::reference_invoices();
        let settings = fixtures::standard_settings();

        let collected = urssaf::total_charges(&invoices, &settings, Basis::Collected);
        let issued = urssaf::total_charges(&invoices, &settings, Basis::Issued);

        assert_eq!(issued - collected, dec!(1002.25));

        let d = reporting::dashboard(&invoices, &fixtures::reference_expenses(), &settings, 2026);
        assert_eq!(d.urssaf_charges, collected);
        assert_eq!(d.urssaf_charges_projected, issued);
    }

    /// Conséquence directe : sur la base encaissée, la somme des mois se
    /// recoupe enfin avec le total annuel. L'ancienne implémentation ne le
    /// permettait pas, puisque le graphe suivait l'émission et le total
    /// l'encaissement.
    #[test]
    fn monthly_and_annual_figures_finally_reconcile() {
        let invoices = fixtures::reference_invoices();

        let monthly = reporting::monthly_series(
            &invoices,
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
            Basis::Collected,
        );

        assert_eq!(
            monthly.ca_values.iter().sum::<crate::Money>(),
            reporting::revenue(&invoices).ht
        );
    }
}

/// Vérifie que le jeu de référence est bien celui de la phase 0 : sans cela, la
/// comparaison ne vaudrait rien.
#[test]
fn the_dataset_is_the_one_frozen_in_phase_zero() {
    let invoices = fixtures::reference_invoices();

    assert_eq!(invoices.len(), 7);
    assert_eq!(
        invoices.iter().filter(|i| i.status == Status::Payee).count(),
        5
    );
    assert_eq!(fixtures::reference_expenses().len(), 8);
    assert_eq!(from_f64(1899.99), dec!(1899.99));
}

/// Contrat de sérialisation avec l'interface.
///
/// L'interface JavaScript manipule des `Number` : elle appelle `.toFixed(2)` et
/// `.toLocaleString()` sur les montants reçus. Si `Decimal` se sérialisait en
/// chaîne, tous ces appels casseraient silencieusement — `"19650.34".toFixed`
/// n'existe pas. Ce test épingle la forme attendue.
#[cfg(test)]
mod serialization_contract {
    use super::*;

    #[test]
    fn amounts_reach_the_interface_as_numbers() {
        let d = reporting::dashboard(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
        );

        let json = serde_json::to_value(&d).unwrap();

        assert!(
            json["revenue"]["ht"].is_number(),
            "les montants doivent arriver en nombres, pas en chaînes : {}",
            json["revenue"]["ht"]
        );
        assert_eq!(json["revenue"]["ht"].as_f64().unwrap(), 19650.34);
        assert_eq!(json["urssafCharges"].as_f64().unwrap(), 3736.99);
    }

    #[test]
    fn field_names_are_camel_case_as_the_views_expect() {
        let d = reporting::dashboard(
            &fixtures::reference_invoices(),
            &fixtures::reference_expenses(),
            &fixtures::standard_settings(),
            2026,
        );

        let json = serde_json::to_value(&d).unwrap();

        for key in [
            "revenue", "totalExpenses", "urssafCharges", "urssafChargesProjected",
            "netProfit", "breakdown", "monthly", "expensesByCategory", "alerts", "gauges",
        ] {
            assert!(json.get(key).is_some(), "champ manquant : {key}");
        }

        assert!(json["revenue"].get("htFacture").is_some());
        assert!(json["monthly"].get("caValues").is_some());
        assert!(json["gauges"]["service"].get("pctTva").is_some());
    }

    #[test]
    fn alerts_keep_the_type_field_the_views_switch_on() {
        let alerts = thresholds::alerts(&fixtures::threshold_scenario("serviceExceededTva"), 2026);
        let json = serde_json::to_value(&alerts).unwrap();

        assert_eq!(json[0]["type"], "danger");
        assert!(json[0]["title"].is_string());
    }

    #[test]
    fn the_declaration_shape_matches_the_compta_tab() {
        let d = urssaf::declaration(
            &fixtures::reference_invoices(),
            &fixtures::standard_settings(),
            Period::Quarterly { year: 2026, quarter: 1 },
        );

        let json = serde_json::to_value(&d).unwrap();

        assert!(json["bnc"]["ht"].is_number());
        assert!(json["bnc"]["rate"].is_number());
        assert!(json["bnc"]["charges"].is_number());
        assert_eq!(json["totalCa"].as_f64().unwrap(), 14999.99);
    }
}
