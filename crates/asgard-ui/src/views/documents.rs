//! Éléments communs aux factures et aux devis : listes d'options, formulaire.
//!
//! Les deux formulaires de `App.jsx` étaient deux copies presque identiques ;
//! ils partagent désormais leurs listes et leurs champs.

use leptos::prelude::*;

/// Types d'activité proposés, libellés repris mot pour mot.
pub fn service_types() -> Vec<(String, String)> {
    vec![
        ("service_bnc".into(), "Prestation de service - Profession Libérale (BNC)".into()),
        ("service_bic".into(), "Prestation de service - Artisanale / Commerciale (BIC)".into()),
        ("vente".into(), "Achat / Vente de marchandises (BIC)".into()),
    ]
}

/// Taux de TVA proposés.
pub fn tva_rates() -> Vec<(String, String)> {
    vec![
        ("0".into(), "0% (Franchise en base de TVA)".into()),
        ("5.5".into(), "5.5% (Taux réduit)".into()),
        ("10".into(), "10% (Taux intermédiaire)".into()),
        ("20".into(), "20% (Taux standard)".into()),
    ]
}

/// Clients disponibles, au format des listes déroulantes.
pub fn client_options() -> Signal<Vec<(String, String)>> {
    let app = crate::state::use_app();
    Signal::derive(move || {
        app.snapshot
            .get()
            .map(|s| {
                s.clients
                    .into_iter()
                    .map(|c| (c.id.to_string(), format!("{} ({})", c.company_name, c.contact_name)))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// Badge d'un type d'activité, tel qu'affiché dans les tableaux.
pub fn type_badge(service_type: &str) -> impl IntoView {
    let (class, label) = match service_type {
        "service_bic" => ("badge badge-blue", "BIC"),
        "vente" => ("badge badge-success", "Vente"),
        _ => ("badge badge-blue", "BNC"),
    };
    view! { <span class=class>{label}</span> }
}

/// Badge d'un statut de facture.
///
/// **Défaut de l'original corrigé.** `InvoicesTab.jsx` testait
/// `status === 'envoye'`, alors que la base stocke `'envoyee'` pour les
/// factures — `'envoye'` est l'orthographe des devis. Résultat : une facture
/// envoyée n'affichait **aucun badge**, et le bouton « Relancer », conditionné
/// au même test, n'apparaissait jamais.
pub fn invoice_status_badge(status: &str) -> impl IntoView {
    let (class, label) = match status {
        "payee" => ("badge badge-success", "Payée"),
        "envoyee" => ("badge badge-blue", "Envoyée"),
        _ => ("badge badge-secondary", "Brouillon"),
    };
    view! { <span class=class>{label}</span> }
}

/// Badge d'un statut de devis. Ici l'original était cohérent avec la base.
pub fn estimate_status_badge(status: &str) -> impl IntoView {
    let (class, label) = match status {
        "envoye" => ("badge badge-blue", "Envoyé"),
        "accepte" => ("badge badge-success", "Accepté"),
        "refuse" => ("badge badge-danger", "Refusé"),
        _ => ("badge badge-secondary", "Brouillon"),
    };
    view! { <span class=class>{label}</span> }
}

/// Une facture peut-elle être relancée ? Seulement si elle a été envoyée et
/// n'est pas encore réglée.
pub fn can_remind(status: &str) -> bool {
    status == "envoyee"
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le défaut corrigé : l'orthographe stockée pour une facture envoyée est
    /// `envoyee`. Avec `envoye`, la relance ne s'affichait jamais.
    #[test]
    fn a_sent_invoice_can_be_reminded() {
        assert!(can_remind("envoyee"));
        assert!(!can_remind("envoye"), "c'est l'orthographe des devis, pas des factures");
        assert!(!can_remind("payee"));
        assert!(!can_remind("brouillon"));
    }

    #[test]
    fn option_lists_carry_the_original_values() {
        let types: Vec<_> = service_types().into_iter().map(|(k, _)| k).collect();
        assert_eq!(types, ["service_bnc", "service_bic", "vente"]);

        let rates: Vec<_> = tva_rates().into_iter().map(|(k, _)| k).collect();
        assert_eq!(rates, ["0", "5.5", "10", "20"]);
    }
}
