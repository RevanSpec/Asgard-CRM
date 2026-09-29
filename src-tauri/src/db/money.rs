//! Conversion entre les montants décimaux de l'interface et les centimes stockés.
//!
//! L'interface manipule encore des `Number` JavaScript — la bascule vers un type
//! décimal est l'objet de la phase 3. La base, elle, stocke des entiers dès
//! maintenant : c'est le seul endroit où l'arrondi peut être décidé une fois
//! pour toutes, et où il cesse de s'accumuler silencieusement.
//!
//! ⚠️ Cette conversion **change des valeurs** lors de la reprise d'une base
//! existante. `1899,99 € × 20 %` valait `379.99800000000005` en flottant ; il
//! vaut `380,00 €` une fois arrondi au centime. C'est plus juste — une facture
//! ne se libelle pas en fractions de centime — mais cela modifie des documents
//! déjà émis. D'où le rapport d'écarts produit à l'import (`backup::import`).

/// Arrondi au centime, demi-supérieur en valeur absolue.
///
/// `f64::round` arrondit déjà le demi à l'écart de zéro, ce qui est la règle
/// attendue en comptabilité française : 0,005 € devient 0,01 € et −0,005 €
/// devient −0,01 €.
pub fn to_cents(amount: f64) -> i64 {
    (amount * 100.0).round() as i64
}

pub fn from_cents(cents: i64) -> f64 {
    cents as f64 / 100.0
}

/// L'arrondi au centime modifie-t-il ce montant ?
///
/// Sert à distinguer, à l'import, les documents réellement touchés de ceux qui
/// traversent la conversion sans bouger — l'immense majorité.
pub fn differs_after_rounding(amount: f64) -> bool {
    (from_cents(to_cents(amount)) - amount).abs() > f64::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_amounts_to_the_nearest_cent() {
        assert_eq!(to_cents(1899.99), 189_999);
        assert_eq!(to_cents(0.0), 0);
        assert_eq!(to_cents(3200.0), 320_000);
        assert_eq!(to_cents(1450.35), 145_035);
    }

    /// Les artefacts flottants figés par les golden tests de la phase 0
    /// disparaissent à l'entrée en base — c'est le début de la correction du
    /// défaut D5.
    #[test]
    fn float_artifacts_collapse_onto_the_cent() {
        assert_eq!(to_cents(379.998_000_000_000_05), 38_000);
        assert_eq!(from_cents(to_cents(379.998_000_000_000_05)), 380.00);

        assert_eq!(to_cents(2279.9880000000003), 227_999);
        assert_eq!(to_cents(79.769_25), 7_977);
    }

    #[test]
    fn round_trip_is_stable_for_amounts_already_on_the_cent() {
        for amount in [0.0, 1.0, 12.34, 1899.99, 188_700.0] {
            assert_eq!(from_cents(to_cents(amount)), amount, "montant {amount}");
            assert!(!differs_after_rounding(amount), "montant {amount}");
        }
    }

    #[test]
    fn detects_amounts_that_the_conversion_changes() {
        assert!(differs_after_rounding(379.998));
        assert!(differs_after_rounding(79.769_25));
        assert!(!differs_after_rounding(380.0));
    }

    #[test]
    fn negative_amounts_round_away_from_zero() {
        assert_eq!(to_cents(-0.005), -1);
        assert_eq!(to_cents(-12.344), -1234);
    }
}
