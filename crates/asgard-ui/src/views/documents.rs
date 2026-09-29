//! Éléments communs aux factures et aux devis : listes d'options, formulaire.
//!
//! Les deux formulaires de `App.jsx` étaient deux copies presque identiques ;
//! ils partagent désormais leurs listes et leurs champs.

use asgard_core::model::{Operation, ServiceType};
use leptos::prelude::*;

/// Types d'activité proposés, libellés repris mot pour mot.
pub fn service_types() -> Vec<(String, String)> {
    vec![
        ("service_bnc".into(), "Prestation de service - Profession Libérale (BNC)".into()),
        ("service_bic".into(), "Prestation de service - Artisanale / Commerciale (BIC)".into()),
        ("vente".into(), "Achat / Vente de marchandises (BIC)".into()),
    ]
}

/// Natures d'opération proposées, dans les termes du décret n° 2022-1299.
///
/// La mention devient obligatoire avec la facturation électronique : une pièce
/// doit dire si elle porte sur des biens, des services, ou les deux.
pub fn operation_kinds() -> Vec<(String, String)> {
    vec![
        ("services".into(), "Prestations de services".into()),
        ("biens".into(), "Livraisons de biens".into()),
        ("mixte".into(), "Les deux".into()),
    ]
}

/// Nature qu'un type d'activité laisse attendre.
pub fn default_operation(service_type: &str) -> String {
    Operation::from_service_type(ServiceType::from_stored(service_type))
        .as_str()
        .to_string()
}

/// La nature affichée doit-elle suivre un changement de type d'activité ?
///
/// Oui tant qu'elle vaut ce que l'ancien type laissait attendre — personne ne
/// l'a donc choisie. Non sinon : un « les deux » saisi à dessein ne se retrouve
/// pas tout seul.
pub fn follows(previous_type: &str, shown: &str) -> bool {
    shown == default_operation(previous_type)
}

/// Fait suivre la nature de l'opération au type d'activité, selon [`follows`].
pub fn follow_service_type(service_type: RwSignal<String>, operation: RwSignal<String>) {
    let previous = StoredValue::new(service_type.get_untracked());

    Effect::new(move |_| {
        let current = service_type.get();
        let was = previous.get_value();
        if current == was {
            return;
        }
        if follows(&was, &operation.get_untracked()) {
            operation.set(default_operation(&current));
        }
        previous.set_value(current);
    });
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

    /// La nature proposée suit le type d'activité : une vente livre des biens,
    /// tout le reste est une prestation.
    #[test]
    fn the_proposed_operation_follows_the_activity() {
        assert_eq!(default_operation("vente"), "biens");
        assert_eq!(default_operation("service_bnc"), "services");
        assert_eq!(default_operation("service_bic"), "services");

        let kinds: Vec<_> = operation_kinds().into_iter().map(|(k, _)| k).collect();
        assert_eq!(kinds, ["services", "biens", "mixte"]);
    }

    /// Changer d'activité corrige une nature laissée par défaut, mais n'efface
    /// pas un choix : « les deux » ne se retrouverait par aucune déduction.
    #[test]
    fn a_chosen_operation_survives_a_change_of_activity() {
        assert!(follows("service_bnc", "services"), "valeur par défaut : elle peut suivre");
        assert!(follows("vente", "biens"));

        assert!(!follows("service_bnc", "mixte"), "choix délibéré : il doit rester");
        assert!(!follows("service_bnc", "biens"), "choix délibéré aussi");
        assert!(!follows("vente", "services"));
    }

    #[test]
    fn option_lists_carry_the_original_values() {
        let types: Vec<_> = service_types().into_iter().map(|(k, _)| k).collect();
        assert_eq!(types, ["service_bnc", "service_bic", "vente"]);

        let rates: Vec<_> = tva_rates().into_iter().map(|(k, _)| k).collect();
        assert_eq!(rates, ["0", "5.5", "10", "20"]);
    }
}
