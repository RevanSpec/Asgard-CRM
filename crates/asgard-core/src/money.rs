//! Montants.
//!
//! Correction définitive du défaut D5. La phase 2 avait fait la moitié du
//! chemin en stockant des centimes ; ici les **calculs** cessent d'être flottants.
//!
//! `Decimal` est un type à virgule fixe : `1899.99` y est exactement
//! `1899,99`, pas la plus proche approximation binaire. La TVA de cette facture
//! valait `379.99800000000005` en `f64` ; elle vaut ici `379,998` puis
//! `380,00 €` après arrondi — sans artefact intermédiaire à traîner.
//!
//! Les golden tests de la phase 0 figeaient les artefacts flottants pour
//! prouver qu'ils existaient. `parity.rs` rejoue les mêmes fixtures et documente
//! chaque écart : c'est le rapport que le plan exigeait avant toute bascule.

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Montant monétaire. Toujours manipulé au centime près à l'affichage, mais la
/// précision interne permet d'enchaîner des calculs sans perte.
pub type Money = Decimal;

/// Pourcentage (taux de TVA, taux de cotisation). Exprimé tel qu'on l'écrit :
/// `20` pour 20 %, `21.1` pour 21,1 %.
pub type Rate = Decimal;

const CENT: Decimal = dec!(100);

/// Arrondi au centime, demi-supérieur en valeur absolue.
///
/// C'est la règle de l'arrondi commercial français : 0,005 € devient 0,01 €,
/// et −0,005 € devient −0,01 €. `MidpointAwayFromZero` la nomme exactement.
///
/// Un arrondi bancaire (`MidpointNearestEven`) donnerait 0,00 € sur le premier
/// cas : ce n'est pas ce que fait l'application aujourd'hui, et le changer
/// silencieusement modifierait des factures.
pub fn round_cents(amount: Money) -> Money {
    amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Convertit en centiemes entiers, pour la base.
pub fn to_cents(amount: Money) -> i64 {
    round_cents(amount * CENT)
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .unwrap_or(0)
}

pub fn from_cents(cents: i64) -> Money {
    Decimal::from(cents) / CENT
}

/// Reconstruit un montant depuis un `f64`.
///
/// Sert uniquement à la frontière avec l'interface, qui manipule encore des
/// `Number` JavaScript, et à la lecture des sauvegardes historiques. Le passage
/// par la chaîne décimale est délibéré : `Decimal::from_f64_retain(379.998…05)`
/// conserverait l'artefact binaire, alors que le formater d'abord le supprime.
pub fn from_f64(value: f64) -> Money {
    Decimal::from_str(&format!("{value:.10}"))
        .unwrap_or_default()
        .normalize()
}

/// Repasse en `f64` pour l'interface. Perte de précision assumée : c'est le
/// dernier maillon, juste avant l'affichage.
pub fn to_f64(amount: Money) -> f64 {
    amount.to_f64().unwrap_or(0.0)
}

/// Applique un pourcentage à un montant, sans arrondir.
///
/// L'arrondi est laissé à l'appelant : arrondir chaque ligne puis sommer ne
/// donne pas le même total que sommer puis arrondir, et c'est une décision
/// comptable, pas un détail d'implémentation.
pub fn percent_of(amount: Money, rate: Rate) -> Money {
    amount * rate / CENT
}

/// TVA et total d'une ligne, arrondis au centime.
///
/// Remplace `computeAmounts` (`src/domain/money.js`).
pub fn compute_amounts(amount_ht: Money, tva_rate: Rate) -> ComputedAmounts {
    let amount_tva = round_cents(percent_of(amount_ht, tva_rate));
    let amount_ht = round_cents(amount_ht);

    ComputedAmounts {
        amount_ht,
        tva_rate,
        amount_tva,
        amount_total: amount_ht + amount_tva,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComputedAmounts {
    pub amount_ht: Money,
    pub tva_rate: Rate,
    pub amount_tva: Money,
    pub amount_total: Money,
}

/// Formatage français : espace insécable pour les milliers, virgule décimale.
///
/// Reproduit `toLocaleString('fr-FR')`, mais sans dépendre de la version d'ICU
/// du moteur de rendu — ce qui rendait les tests de la phase 0 sensibles à la
/// version de Node.
pub fn format_fr(amount: Money) -> String {
    let rounded = round_cents(amount);
    let negative = rounded.is_sign_negative();
    let text = rounded.abs().to_string();

    let (integer, fraction) = match text.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (text, None),
    };

    let mut grouped = String::new();
    for (index, ch) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index) % 3 == 0 {
            grouped.push('\u{202F}'); // espace fine insécable
        }
        grouped.push(ch);
    }

    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(&grouped);
    if let Some(fraction) = fraction {
        out.push(',');
        out.push_str(&fraction);
    }
    out
}

/// Formatage CSV : virgule décimale, deux chiffres, pas de séparateur de milliers.
pub fn format_csv(amount: Money) -> String {
    format!("{:.2}", round_cents(amount)).replace('.', ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_are_exact_where_floats_were_not() {
        // L'artefact que les golden tests de la phase 0 figeaient :
        // en f64, 1899.99 * 20 / 100 vaut 379.99800000000005.
        let tva = percent_of(dec!(1899.99), dec!(20));

        assert_eq!(tva, dec!(379.998));
        assert_ne!(to_f64(tva), 379.998_000_000_000_05);
    }

    #[test]
    fn rounds_half_away_from_zero() {
        assert_eq!(round_cents(dec!(0.005)), dec!(0.01));
        assert_eq!(round_cents(dec!(-0.005)), dec!(-0.01));
        assert_eq!(round_cents(dec!(0.004)), dec!(0.00));
        assert_eq!(round_cents(dec!(379.998)), dec!(380.00));
        assert_eq!(round_cents(dec!(79.76925)), dec!(79.77));
    }

    /// L'arrondi bancaire donnerait 0,00 € — ce n'est pas la règle appliquée
    /// aujourd'hui, et en changer modifierait des factures déjà émises.
    #[test]
    fn does_not_use_bankers_rounding() {
        assert_eq!(round_cents(dec!(0.005)), dec!(0.01));
        assert_eq!(round_cents(dec!(0.015)), dec!(0.02));
        assert_eq!(round_cents(dec!(0.025)), dec!(0.03));
    }

    #[test]
    fn converts_to_and_from_cents() {
        assert_eq!(to_cents(dec!(1899.99)), 189_999);
        assert_eq!(to_cents(dec!(0)), 0);
        assert_eq!(to_cents(dec!(-12.344)), -1234);
        assert_eq!(from_cents(189_999), dec!(1899.99));
    }

    /// Le passage par la chaîne décimale efface l'artefact binaire au lieu de
    /// le faire entrer dans le calcul.
    #[test]
    fn f64_conversion_drops_binary_artifacts() {
        assert_eq!(from_f64(379.998_000_000_000_05), dec!(379.998));
        assert_eq!(from_f64(1899.99), dec!(1899.99));
        assert_eq!(from_f64(0.1) + from_f64(0.2), dec!(0.3));
    }

    #[test]
    fn compute_amounts_matches_the_invoice_form() {
        let a = compute_amounts(dec!(5000), dec!(20));
        assert_eq!(a.amount_tva, dec!(1000));
        assert_eq!(a.amount_total, dec!(6000));

        let b = compute_amounts(dec!(1899.99), dec!(20));
        assert_eq!(b.amount_tva, dec!(380.00));
        assert_eq!(b.amount_total, dec!(2279.99));

        let c = compute_amounts(dec!(1450.35), dec!(5.5));
        assert_eq!(c.amount_tva, dec!(79.77));
        assert_eq!(c.amount_total, dec!(1530.12));

        let exempt = compute_amounts(dec!(1200), dec!(0));
        assert_eq!(exempt.amount_tva, dec!(0));
        assert_eq!(exempt.amount_total, dec!(1200));
    }

    #[test]
    fn formats_amounts_the_french_way() {
        assert_eq!(format_fr(dec!(35000)), "35\u{202F}000");
        assert_eq!(format_fr(dec!(1450.35)), "1\u{202F}450,35");
        assert_eq!(format_fr(dec!(188700)), "188\u{202F}700");
        assert_eq!(format_fr(dec!(0)), "0");
        assert_eq!(format_fr(dec!(-1234.5)), "-1\u{202F}234,5");
    }

    #[test]
    fn formats_csv_amounts() {
        assert_eq!(format_csv(dec!(1899.99)), "1899,99");
        assert_eq!(format_csv(dec!(1530.11925)), "1530,12");
        assert_eq!(format_csv(dec!(3200)), "3200,00");
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    fn money() -> impl Strategy<Value = Money> {
        (-100_000_000_i64..100_000_000_i64).prop_map(from_cents)
    }

    proptest! {
        /// L'invariant comptable fondamental : HT + TVA = TTC, exactement.
        /// En `f64` il ne tenait qu'à l'epsilon près.
        #[test]
        fn ht_plus_tva_equals_total(ht in money(), rate in 0_u32..=25) {
            let a = compute_amounts(ht, Decimal::from(rate));
            prop_assert_eq!(a.amount_ht + a.amount_tva, a.amount_total);
        }

        /// Un aller-retour par les centimes ne perd rien sur un montant déjà
        /// exprimé au centime.
        #[test]
        fn cents_round_trip(cents in -100_000_000_i64..100_000_000_i64) {
            prop_assert_eq!(to_cents(from_cents(cents)), cents);
        }

        /// Arrondir un montant déjà arrondi ne le change plus.
        #[test]
        fn rounding_is_idempotent(amount in money()) {
            prop_assert_eq!(round_cents(round_cents(amount)), round_cents(amount));
        }

        /// Un taux nul ne produit jamais de TVA, quel que soit le montant.
        #[test]
        fn a_zero_rate_yields_no_tva(ht in money()) {
            let a = compute_amounts(ht, Decimal::ZERO);
            prop_assert_eq!(a.amount_tva, Decimal::ZERO);
            prop_assert_eq!(a.amount_total, a.amount_ht);
        }
    }
}
