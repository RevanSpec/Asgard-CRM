//! Devis — port de `src/components/EstimatesTab.jsx` et de sa modale.

use asgard_ipc::{DocumentInput, Estimate};
use leptos::prelude::*;

use super::documents::{self, estimate_status_badge};
use super::icons;
use super::modals::{open_email, today_iso};
use super::widgets::{EmptyState, FilterBar, GoldButton, IconButton, Modal, SelectField, TextArea, TextField};
use crate::state::{use_app, Pending};
use crate::templates::Kind;
use crate::validation::{self, Field, FieldErrors};
use crate::{actions, format};

/// Devis en cours d'édition. `None` en identifiant signifie création.
#[derive(Debug, Clone, PartialEq)]
struct Draft {
    id: Option<i64>,
    client_id: String,
    service_type: String,
    tva_rate: String,
    amount_ht: String,
    date: String,
    status: String,
    description: String,
}

impl Draft {
    fn blank(first_client: String) -> Self {
        Self {
            id: None,
            client_id: first_client,
            service_type: "service_bnc".into(),
            tva_rate: "20".into(),
            amount_ht: String::new(),
            date: today_iso(),
            status: "brouillon".into(),
            description: String::new(),
        }
    }

    fn from_estimate(est: &Estimate) -> Self {
        Self {
            id: Some(est.id),
            client_id: est.client_id.map(|c| c.to_string()).unwrap_or_default(),
            service_type: est.service_type.clone(),
            tva_rate: est.tva_rate.to_string(),
            amount_ht: est.amount_ht.to_string(),
            date: est.date.get(..10).unwrap_or(&est.date).to_string(),
            status: est.status.clone(),
            description: est.description.clone(),
        }
    }
}

pub fn estimates() -> impl IntoView {
    let app = use_app();
    let search = RwSignal::new(String::new());
    let draft = RwSignal::new(None::<Draft>);

    let has_any = move || app.snapshot.with(|s| s.as_ref().is_some_and(|s| !s.estimates.is_empty()));

    let visible = move || {
        let needle = search.get().to_lowercase();
        app.snapshot
            .get()
            .map(|s| {
                s.estimates
                    .into_iter()
                    .filter(|est| {
                        needle.is_empty()
                            || est.estimate_number.to_lowercase().contains(&needle)
                            || est.company_name.to_lowercase().contains(&needle)
                            || est.description.to_lowercase().contains(&needle)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    let open_create = move |_| {
        match app.snapshot.get().and_then(|s| s.clients.first().map(|c| c.id.to_string())) {
            Some(first) => draft.set(Some(Draft::blank(first))),
            None => app.inform(
                "Client requis",
                "Veuillez d'abord créer au moins un client avant de générer un devis.",
            ),
        }
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Gestion des Devis"</h1>
                    <p>"Créez, éditez, exportez et convertissez vos devis en factures."</p>
                </div>
                <div class="flex-gap-2">
                    <button class="btn btn-primary" on:click=open_create>
                        {icons::add()}
                        " Nouveau Devis"
                    </button>
                </div>
            </div>

            <FilterBar value=search placeholder="Rechercher un devis (Numéro, client, description...)" />

            // Comme l'original, l'état vide ne s'affiche qu'en l'absence de
            // tout devis ; une recherche sans résultat laisse le tableau vide.
            <Show
                when=has_any
                fallback=|| view! {
                    <EmptyState message="Aucun devis créé pour le moment. Cliquez sur \"Nouveau Devis\" pour commencer." />
                }
            >
                <div class="table-container">
                    <table class="table-glass">
                        <thead>
                            <tr>
                                <th>"Numéro"</th>
                                <th>"Client"</th>
                                <th>"Date"</th>
                                <th>"Statut"</th>
                                <th>"Montant HT"</th>
                                <th>"Total TTC"</th>
                                <th class="text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            // Lignes redessinées à chaque changement : une `<For>` indexée
                            // sur l'identifiant garderait l'ancien contenu d'une ligne modifiée.
                            {move || {
                                visible()
                                    .into_iter()
                                    .map(|estimate| view! { <Row estimate=estimate draft=draft /> })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </Show>

            <Show when=move || draft.get().is_some()>
                {move || draft.get().map(|current| view! { <Form initial=current draft=draft /> })}
            </Show>
        </div>
    }
}

#[component]
fn Row(estimate: Estimate, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let id = estimate.id;
    let convertible = estimate.status != "accepte";
    let number = estimate.estimate_number.clone();
    let for_email = estimate.clone();
    let for_edit = estimate.clone();

    view! {
        <tr>
            <td style="font-weight: 600; color: var(--color-gold)">{estimate.estimate_number.clone()}</td>
            <td>{estimate.company_name.clone()}</td>
            <td>{format::date(&estimate.date)}</td>
            <td>{estimate_status_badge(&estimate.status)}</td>
            <td>{format::euros(estimate.amount_ht)}</td>
            <td style="font-weight: 600">{format::euros(estimate.amount_total)}</td>
            <td class="text-right">
                <div class="flex-gap-2" style="justify-content: flex-end">
                    {convertible.then(|| view! {
                        <GoldButton
                            label="Facturer"
                            title="Convertir en Facture"
                            on_click=Callback::new(move |_| {
                                app.pending.set(Some(Pending::ConvertEstimate { id, number: number.clone() }))
                            })
                        />
                    })}
                    <IconButton
                        icon=icons::email
                        title="Envoyer par e-mail"
                        on_click=Callback::new(move |_| {
                            let e = &for_email;
                            open_email(
                                app,
                                Kind::Estimate,
                                e.id,
                                e.estimate_number.clone(),
                                e.description.clone(),
                                e.amount_total,
                                e.date.clone(),
                                e.client_id,
                                e.company_name.clone(),
                            );
                        })
                    />
                    <IconButton
                        icon=icons::download
                        title="Télécharger le PDF"
                        on_click=Callback::new(move |_| {
                            leptos::task::spawn_local(actions::export_pdf(app, Kind::Estimate, id))
                        })
                    />
                    <IconButton
                        icon=icons::edit
                        title="Modifier le devis"
                        on_click=Callback::new(move |_| draft.set(Some(Draft::from_estimate(&for_edit))))
                    />
                    <IconButton
                        icon=icons::delete
                        title="Supprimer le devis"
                        danger=true
                        on_click=Callback::new(move |_| app.pending.set(Some(Pending::DeleteEstimate(id))))
                    />
                </div>
            </td>
        </tr>
    }
}

#[component]
fn Form(initial: Draft, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let is_edit = initial.id.is_some();

    let client = RwSignal::new(initial.client_id);
    let service_type = RwSignal::new(initial.service_type);
    let tva = RwSignal::new(initial.tva_rate);
    let amount = RwSignal::new(initial.amount_ht);
    let date = RwSignal::new(initial.date);
    let status = RwSignal::new(initial.status);
    let description = RwSignal::new(initial.description);
    let errors = RwSignal::new(FieldErrors::default());
    let error = move |field| Signal::derive(move || errors.with(|e| e.get(field)));
    let id = initial.id;

    let statuses = Signal::derive(|| {
        vec![
            ("brouillon".into(), "Brouillon".into()),
            ("envoye".into(), "Envoyé".into()),
            ("accepte".into(), "Accepté".into()),
            ("refuse".into(), "Refusé".into()),
        ]
    });

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let client_id = client.get().parse::<i64>().ok();

        let found = validation::validate_estimate(client_id, &description.get(), &amount.get());
        let valid = found.is_empty();
        errors.set(found);
        if !valid {
            return;
        }

        let Some(chosen) = app
            .snapshot
            .get_untracked()
            .and_then(|s| s.clients.into_iter().find(|c| Some(c.id) == client_id))
        else {
            return;
        };

        let input = DocumentInput {
            id,
            client_id: chosen.id,
            company_name: chosen.company_name,
            service_type: service_type.get(),
            description: description.get(),
            amount_ht: validation::parse_amount(&amount.get()).unwrap_or_default(),
            tva_rate: validation::parse_amount(&tva.get()).unwrap_or_default(),
            date: format!("{}T00:00:00Z", date.get()),
            // Un nouveau devis naît toujours en brouillon, comme dans l'original.
            status: Some(if id.is_some() { status.get() } else { "brouillon".into() }),
        };

        leptos::task::spawn_local(async move {
            if actions::save_estimate(app, input).await {
                draft.set(None);
            }
        });
    };

    view! {
        <Modal
            title=if is_edit { "Modifier le devis" } else { "Générer un devis" }
            on_close=Callback::new(move |_| draft.set(None))
        >
            <form on:submit=submit>
                <div class="modal-body">
                    <SelectField label="Client" value=client options=documents::client_options() />
                    <SelectField
                        label="Type d'activité"
                        value=service_type
                        options=Signal::derive(documents::service_types)
                    />
                    <SelectField label="Taux de TVA (%)" value=tva options=Signal::derive(documents::tva_rates) />
                    <TextField
                        label="Montant HT (€)"
                        value=amount
                        placeholder="Ex: 1500.00"
                        error=error(Field::Amount)
                    />
                    <TextField label="Date du devis" value=date kind="date" />
                    {is_edit.then(|| view! {
                        <SelectField label="Statut du devis" value=status options=statuses />
                    })}
                    <TextArea
                        label="Description des prestations"
                        value=description
                        placeholder="Détaillez les travaux prévus..."
                        error=error(Field::Description)
                    />
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" on:click=move |_| draft.set(None)>
                        "Annuler"
                    </button>
                    <button type="submit" class="btn btn-primary">"Enregistrer"</button>
                </div>
            </form>
        </Modal>
    }
}
