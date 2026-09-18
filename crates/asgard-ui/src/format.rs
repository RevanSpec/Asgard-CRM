//! Formatage pour l'affichage.
//!
//! Le JavaScript s'appuyait sur `toLocaleString('fr-FR')` et `toFixed(2)`. La
//! première dépend de la version d'ICU du moteur, ce qui rendait les tests de
//! la phase 0 sensibles à la version de Node ; `asgard-core` fait le travail
//! lui-même, et la même fonction sert désormais à l'écran et dans les PDF.

use asgard_core::money::{format_fr, round_cents};
use asgard_core::Money;

/// Montant en euros, séparateurs de milliers et deux décimales.
pub fn euros(amount: f64) -> String {
    format!("{} €", with_cents(amount))
}

/// Montant avec ses centimes, toujours affichés.
///
/// `format_fr` élide une partie décimale nulle — ce qui convient à un seuil
/// légal (« 77 700 € ») mais pas à un montant facturé, qui s'écrit avec ses
/// centimes même quand ils valent zéro.
pub fn with_cents(amount: f64) -> String {
    let value = round_cents(asgard_core::from_f64(amount));
    let text = format_fr(value);

    match text.split_once(',') {
        Some((_, decimals)) if decimals.len() == 2 => text,
        Some((units, decimals)) => format!("{units},{decimals}0"),
        None => format!("{text},00"),
    }
}

/// Montant sans centimes, pour les seuils et plafonds.
pub fn round_euros(amount: Money) -> String {
    format!("{} €", format_fr(amount))
}

/// Pourcentage à une décimale, comme les jauges de l'interface.
pub fn percent(value: f64) -> String {
    format!("{value:.1}%")
}

/// Taux de TVA, sans zéro superflu : `20%`, `5.5%`.
pub fn rate(value: f64) -> String {
    let value = asgard_core::from_f64(value).normalize();
    format!("{value}%")
}

/// Date ISO affichée à la française. Une date illisible ressort telle quelle,
/// plutôt que d'être remplacée par un tiret qui masquerait le problème.
pub fn date(iso: &str) -> String {
    asgard_core::CivilDate::parse(iso)
        .map(|d| d.format_fr())
        .unwrap_or_else(|| iso.to_string())
}

/// Libellé d'un moyen de paiement, avec son pictogramme.
pub fn payment_method(method: Option<&str>) -> &'static str {
    match method {
        Some("carte") => "💳 Carte",
        Some("especes") => "💵 Espèces",
        Some("cheque") => "✉️ Chèque",
        // Le virement est le défaut historique, y compris quand le champ manque.
        _ => "🏦 Virement",
    }
}

/// Libellé d'un type d'activité.
pub fn service_type(raw: &str) -> &'static str {
    match raw {
        "service_bic" => "Artisanale/Comm. (BIC)",
        "vente" => "Vente Marchandises",
        _ => "Libérale (BNC)",
    }
}

/// Libellé d'un statut de pièce.
pub fn status(raw: &str) -> &'static str {
    match raw {
        "payee" => "Payée",
        "envoyee" => "Envoyée",
        "envoye" => "Envoyé",
        "accepte" => "Accepté",
        "refuse" => "Refusé",
        _ => "Brouillon",
    }
}

/// Classe CSS d'un statut, reprise du thème existant.
pub fn status_class(raw: &str) -> &'static str {
    match raw {
        "payee" | "accepte" => "badge badge-success",
        "envoyee" | "envoye" => "badge badge-info",
        "refuse" => "badge badge-danger",
        _ => "badge badge-muted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn amounts_always_show_their_cents() {
        assert_eq!(with_cents(33400.0), "33\u{202F}400,00");
        assert_eq!(with_cents(205.98), "205,98");
        assert_eq!(with_cents(1450.5), "1\u{202F}450,50");
        assert_eq!(with_cents(0.0), "0,00");
    }

    #[test]
    fn euros_appends_the_symbol() {
        assert_eq!(euros(6924.2), "6\u{202F}924,20 €");
    }

    /// Les seuils s'affichent sans centimes : « 77 700 € », pas « 77 700,00 € ».
    #[test]
    fn thresholds_omit_the_cents() {
        assert_eq!(round_euros(dec!(77700)), "77\u{202F}700 €");
        assert_eq!(round_euros(dec!(188700)), "188\u{202F}700 €");
    }

    #[test]
    fn rates_drop_trailing_zeros() {
        assert_eq!(rate(20.0), "20%");
        assert_eq!(rate(5.5), "5.5%");
        assert_eq!(rate(0.0), "0%");
    }

    #[test]
    fn percentages_keep_one_decimal() {
        assert_eq!(percent(95.83333), "95.8%");
        assert_eq!(percent(0.0), "0.0%");
    }

    #[test]
    fn dates_display_the_french_way() {
        assert_eq!(date("2026-04-18T10:00:00.000Z"), "18/04/2026");
        assert_eq!(date("2026-01-08"), "08/01/2026");
    }

    /// Une date illisible ressort telle quelle : masquer le problème derrière
    /// un tiret rendrait le diagnostic impossible.
    #[test]
    fn an_unreadable_date_is_shown_as_is() {
        assert_eq!(date("pas une date"), "pas une date");
    }

    #[test]
    fn a_missing_payment_method_defaults_to_transfer() {
        assert_eq!(payment_method(None), "🏦 Virement");
        assert_eq!(payment_method(Some("")), "🏦 Virement");
        assert_eq!(payment_method(Some("carte")), "💳 Carte");
    }

    #[test]
    fn labels_match_the_existing_interface() {
        assert_eq!(service_type("service_bnc"), "Libérale (BNC)");
        assert_eq!(service_type("vente"), "Vente Marchandises");
        assert_eq!(status("payee"), "Payée");
        assert_eq!(status("inconnu"), "Brouillon");
    }
}
