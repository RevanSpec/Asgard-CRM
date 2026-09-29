//! Écran d'accueil du premier lancement.
//!
//! Il n'existait pas. L'application s'ouvrait sur un tableau de bord rempli de
//! données fictives, avec des réglages qui décrivaient une société inventée —
//! raison sociale, SIRET et IBAN d'apparence crédible. Une facture émise avant
//! d'avoir ouvert les réglages partait donc au nom de quelqu'un d'autre, vers un
//! compte inexistant (défauts D11 et D12).
//!
//! Cet écran demande les quatre informations que l'hôte exige pour éditer une
//! facture, et rien de plus : le reste se remplit dans les réglages, quand le
//! besoin s'en fait sentir. Il ne s'affiche qu'une fois, et jamais sur une
//! installation qui contient déjà du travail.

use leptos::prelude::*;
use serde::Serialize;

use super::widgets::TextField;
use crate::settings::{self, Settings};
use crate::state::use_app;
use crate::ipc;

/// Champs exigés, nommés comme l'hôte les nomme dans son refus.
///
/// L'accueil et le garde de l'hôte doivent dire la même chose : un utilisateur
/// qui lit « il manque le SIRET » doit retrouver le mot sur l'écran.
const REQUIRED: [&str; 4] = ["la raison sociale", "l'adresse", "le SIRET", "l'IBAN"];

pub fn onboarding() -> impl IntoView {
    let app = use_app();

    // Le cache peut déjà contenir quelque chose : une installation dont la base
    // est vide mais dont les réglages ont été remplis, ou une reprise de
    // sauvegarde. Autant le proposer plutôt que de le faire ressaisir.
    let known = StoredValue::new(settings::load());
    let start_from = known.get_value();

    let company = RwSignal::new(start_from.company_name);
    let contact = RwSignal::new(start_from.contact_name);
    let address = RwSignal::new(start_from.address);
    let siret = RwSignal::new(start_from.siret);
    let iban = RwSignal::new(start_from.iban);

    let missing = RwSignal::new(Vec::<&'static str>::new());
    let working = RwSignal::new(false);

    // Le reproche s'efface à mesure que les champs se remplissent : laisser
    // « il manque le SIRET » sous un SIRET saisi n'apprendrait rien à personne.
    let still_missing = move || {
        let values = [company.get(), address.get(), siret.get(), iban.get()];
        missing
            .get()
            .into_iter()
            .filter(|label| {
                REQUIRED
                    .iter()
                    .position(|required| required == label)
                    .is_some_and(|rank| values[rank].trim().is_empty())
            })
            .collect::<Vec<_>>()
    };

    let start = move |seed_demo: bool| {
        let entered = Settings {
            company_name: company.get().trim().to_string(),
            contact_name: contact.get().trim().to_string(),
            address: address.get().trim().to_string(),
            siret: siret.get().trim().to_string(),
            iban: iban.get().trim().to_string(),
            ..known.get_value()
        };

        let blank: Vec<&'static str> = REQUIRED
            .iter()
            .zip([&entered.company_name, &entered.address, &entered.siret, &entered.iban])
            .filter(|(_, value)| value.is_empty())
            .map(|(label, _)| *label)
            .collect();

        if !blank.is_empty() {
            missing.set(blank);
            return;
        }

        missing.set(Vec::new());
        working.set(true);
        // Le cache est écrit sur place, la base par la file d'attente : celle-ci
        // poursuit son écriture même quand l'accueil a disparu de l'écran, et
        // signale en console si elle échoue.
        settings::save(&entered);

        leptos::task::spawn_local(async move {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
                seed_demo: bool,
            }

            match ipc::invoke::<_, bool>("complete_first_run", &Args { seed_demo }).await {
                Ok(_) => {
                    app.first_run.set(false);
                    // L'écran Paramètres repart des valeurs saisies ici.
                    app.settings_epoch.update(|epoch| *epoch += 1);
                    app.reload().await;
                }
                Err(error) => {
                    working.set(false);
                    app.report_as("Démarrage impossible", error);
                }
            }
        });
    };

    view! {
        // Mise en page en ligne, comme ailleurs dans ces vues : le thème repris
        // de la version React ne connaît pas cet écran, et n'a pas à changer
        // pour lui.
        <div style="display: flex; align-items: center; justify-content: center; min-height: 100vh; width: 100%; padding: 2rem">
            <div class="card-glass" style="max-width: 640px; width: 100%">
                <div class="page-title-container" style="margin-bottom: 1.75rem">
                    <h1 style="font-family: var(--font-title); font-size: 1.75rem">
                        "Bienvenue dans Asgard CRM"
                    </h1>
                    <p class="metric-subtext" style="margin-top: 0.75rem; line-height: 1.5">
                        "Ces informations figurent sur chacune de vos factures. \
                         Tant qu'elles manquent, aucune pièce ne peut être éditée — \
                         mieux vaut un refus qu'une facture au nom de personne."
                    </p>
                </div>

                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem">
                    <TextField label="Nom de l'entreprise" value=company />
                    <TextField label="Nom du contact" value=contact />
                    <div style="grid-column: span 2">
                        <TextField label="Adresse professionnelle" value=address />
                    </div>
                    <TextField label="SIRET" value=siret />
                    <TextField label="IBAN bancaire (Règlement)" value=iban />
                </div>

                <Show when=move || !still_missing().is_empty()>
                    <span class="error-text">
                        {move || format!("Il manque {}.", still_missing().join(", "))}
                    </span>
                </Show>

                <p class="metric-subtext" style="margin-top: 1.5rem; line-height: 1.5">
                    "Le jeu d'exemple ajoute trois clients fictifs, des factures, des devis \
                     et des dépenses, pour voir l'application remplie. Ses factures consomment \
                     la numérotation réglementaire : à éviter si vous démarrez votre activité."
                </p>

                <div style="display: flex; gap: 0.75rem; margin-top: 1.5rem">
                    <button
                        type="button"
                        class="btn btn-primary"
                        disabled=move || working.get()
                        on:click=move |_| start(false)
                    >
                        "Commencer"
                    </button>
                    <button
                        type="button"
                        class="btn btn-secondary"
                        disabled=move || working.get()
                        on:click=move |_| start(true)
                    >
                        "Commencer avec un jeu d'exemple"
                    </button>
                </div>
            </div>
        </div>
    }
}
