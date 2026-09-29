//! Factures — port de `src/components/InvoicesTab.jsx` et de sa modale.

use std::collections::HashSet;

use asgard_ipc::{DocumentInput, Invoice};
use leptos::prelude::*;

use super::documents::{self, can_remind, invoice_status_badge, type_badge};
use super::icons;
use super::modals::{open_email, Outgoing};
use super::widgets::{EmptyState, GoldButton, IconButton, Modal, SearchBox, SelectField, TextArea, TextField};
use crate::state::{use_app, App, Pending, PaymentForm};
use crate::templates::Kind;
use crate::validation::{self, Field, FieldErrors};
use crate::{actions, format};

pub fn invoices() -> impl IntoView {
    let app = use_app();
    let search = RwSignal::new(String::new());
    let selected = RwSignal::new(HashSet::<i64>::new());
    let creating = RwSignal::new(false);

    let visible = move || {
        let needle = search.get().to_lowercase();
        app.snapshot
            .get()
            .map(|s| {
                s.invoices
                    .into_iter()
                    .filter(|inv| {
                        needle.is_empty()
                            || inv.invoice_number.to_lowercase().contains(&needle)
                            || inv.company_name.to_lowercase().contains(&needle)
                            || inv.description.to_lowercase().contains(&needle)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    // La sélection ne garde que les factures encore présentes : après une
    // suppression confirmée, les lignes disparues en sortent d'elles-mêmes,
    // et une confirmation annulée la laisse intacte.
    Effect::new(move |_| {
        let present: HashSet<i64> = app
            .snapshot
            .with(|s| s.as_ref().map(|s| s.invoices.iter().map(|i| i.id).collect()).unwrap_or_default());
        if selected.with_untracked(|set| set.iter().any(|id| !present.contains(id))) {
            selected.update(|set| set.retain(|id| present.contains(id)));
        }
    });

    let all_selected = move || {
        let rows = visible();
        !rows.is_empty() && rows.iter().all(|inv| selected.get().contains(&inv.id))
    };

    let open_create = move |_| open_invoice_form(app, creating);

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Factures"</h1>
                    <p>"Historique des prestations de services, facturation et exports PDF."</p>
                </div>
                <div class="flex-gap-2" style="align-items: center">
                    <Show when=move || !selected.get().is_empty()>
                        <button
                            class="btn btn-danger"
                            on:click=move |_| {
                                let ids: Vec<i64> = selected.get().into_iter().collect();
                                app.pending.set(Some(Pending::DeleteInvoices(ids)));
                            }
                        >
                            {icons::delete()}
                            {move || format!(" Supprimer ({})", selected.get().len())}
                        </button>
                    </Show>
                    <SearchBox value=search placeholder="Numéro, client, description..." />
                    <button class="btn btn-primary" on:click=open_create>
                        {icons::add()}
                        " Nouvelle Facture"
                    </button>
                </div>
            </div>

            <div class="card-glass" style="padding: 0.5rem 0">
                <Show
                    when=move || !visible().is_empty()
                    fallback=|| view! { <EmptyState message="Aucune facture trouvée." rune=true /> }
                >
                    <div class="table-container">
                        <table class="table-glass">
                            <thead>
                                <tr>
                                    <th style="width: 40px">
                                        <input
                                            type="checkbox"
                                            aria-label="Tout sélectionner"
                                            style="transform: scale(1.15); cursor: pointer; accent-color: var(--color-gold)"
                                            prop:checked=all_selected
                                            on:change=move |ev| {
                                                if event_target_checked(&ev) {
                                                    selected.set(visible().into_iter().map(|i| i.id).collect());
                                                } else {
                                                    selected.set(HashSet::new());
                                                }
                                            }
                                        />
                                    </th>
                                    <th>"Numéro"</th>
                                    <th>"Date"</th>
                                    <th>"Client"</th>
                                    <th>"Type"</th>
                                    <th>"Statut"</th>
                                    <th>"Montant HT"</th>
                                    <th>"TVA"</th>
                                    <th>"Montant TTC"</th>
                                    <th class="text-right">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                // Lignes redessinées à chaque changement : une `<For>` indexée
                                // sur l'identifiant garderait l'ancien contenu d'une ligne modifiée.
                                {move || {
                                    visible()
                                        .into_iter()
                                        .map(|invoice| view! { <Row invoice=invoice selected=selected /> })
                                        .collect_view()
                                }}
                            </tbody>
                        </table>
                    </div>
                </Show>
            </div>

            <Show when=move || creating.get()>
                <CreateForm open=creating />
            </Show>
        </div>
    }
}

#[component]
fn Row(invoice: Invoice, selected: RwSignal<HashSet<i64>>) -> impl IntoView {
    let app = use_app();
    let id = invoice.id;
    let number = invoice.invoice_number.clone();
    let is_selected = move || selected.get().contains(&id);

    let for_email = invoice.clone();
    let for_reminder = invoice.clone();
    let payable = invoice.status != "payee";
    let remindable = can_remind(&invoice.status);


    view! {
        <tr class:selected-row=is_selected>
            <td>
                <input
                    type="checkbox"
                    aria-label=format!("Sélectionner {number}")
                    style="transform: scale(1.15); cursor: pointer; accent-color: var(--color-gold)"
                    prop:checked=is_selected
                    on:change=move |ev| {
                        selected.update(|set| {
                            if event_target_checked(&ev) {
                                set.insert(id);
                            } else {
                                set.remove(&id);
                            }
                        });
                    }
                />
            </td>
            <td style="font-weight: 700; color: var(--color-gold)">{invoice.invoice_number.clone()}</td>
            <td>{format::date(&invoice.date)}</td>
            <td>{invoice.company_name.clone()}</td>
            <td>{type_badge(&invoice.service_type)}</td>
            <td>{invoice_status_badge(&invoice.status)}</td>
            <td>{format::euros(invoice.amount_ht)}</td>
            <td>{format::rate(invoice.tva_rate)}</td>
            <td style="font-weight: 600">{format::euros(invoice.amount_total)}</td>
            <td class="text-right">
                <div class="flex-gap-2" style="justify-content: flex-end">
                    {payable.then(|| {
                        let number = invoice.invoice_number.clone();
                        view! {
                            <GoldButton
                                label="Régler"
                                title="Enregistrer le règlement"
                                on_click=Callback::new(move |_| {
                                    app.payment.set(Some(PaymentForm { invoice_id: id, number: number.clone() }))
                                })
                            />
                        }
                    })}
                    {remindable.then(|| view! {
                        <GoldButton
                            label="Relancer"
                            title="Envoyer un rappel de paiement par e-mail"
                            on_click=Callback::new(move |_| email_invoice(app, &for_reminder, Kind::Reminder))
                        />
                    })}
                    <IconButton
                        icon=icons::email
                        title="Envoyer par e-mail"
                        on_click=Callback::new(move |_| email_invoice(app, &for_email, Kind::Invoice))
                    />
                    <IconButton
                        icon=icons::download
                        title="Télécharger le PDF"
                        on_click=Callback::new(move |_| {
                            leptos::task::spawn_local(actions::export_pdf(app, Kind::Invoice, id))
                        })
                    />
                    <IconButton
                        icon=icons::delete
                        title="Supprimer la facture"
                        danger=true
                        on_click=Callback::new(move |_| app.pending.set(Some(Pending::DeleteInvoices(vec![id]))))
                    />
                </div>
            </td>
        </tr>
    }
}

/// Ouvre la création de facture, s'il existe au moins un client. Partagé avec
/// le tableau de bord, qui porte le même bouton « Nouvelle Facture ».
///
/// L'état arrive en argument : appelée depuis un gestionnaire de clic, cette
/// fonction ne peut pas le lire elle-même (voir `state::use_app`).
pub(super) fn open_invoice_form(app: App, creating: RwSignal<bool>) {
    let has_clients = app.snapshot.get_untracked().is_some_and(|s| !s.clients.is_empty());
    if has_clients {
        creating.set(true);
    } else {
        app.inform(
            "Client requis",
            "Veuillez d'abord créer au moins un client avant de générer une facture.",
        );
    }
}

/// Ouvre l'envoi par e-mail d'une facture, ou sa relance.
pub(super) fn email_invoice(app: App, inv: &Invoice, kind: Kind) {
    open_email(
        app,
        kind,
        Outgoing {
            id: inv.id,
            number: inv.invoice_number.clone(),
            description: inv.description.clone(),
            total: inv.amount_total,
            date: inv.date.clone(),
            due_date: inv.due_date.clone(),
            client_id: inv.client_id,
            company: inv.company_name.clone(),
        },
    );
}

#[component]
pub(super) fn CreateForm(open: RwSignal<bool>) -> impl IntoView {
    let app = use_app();
    let clients = documents::client_options();

    let first_client = app
        .snapshot
        .get_untracked()
        .and_then(|s| s.clients.first().map(|c| c.id.to_string()))
        .unwrap_or_default();

    let client = RwSignal::new(first_client);
    let service_type = RwSignal::new("service_bnc".to_string());
    let tva = RwSignal::new("20".to_string());
    let amount = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let errors = RwSignal::new(FieldErrors::default());
    let error = move |field| Signal::derive(move || errors.with(|e| e.get(field)));

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let client_id = client.get().parse::<i64>().ok();

        let found = validation::validate_invoice(client_id, &description.get(), &amount.get());
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
            id: None,
            client_id: chosen.id,
            company_name: chosen.company_name,
            service_type: service_type.get(),
            description: description.get(),
            // Le numéro, la TVA et le total sont établis par l'hôte : l'interface
            // ne transmet que ce que l'utilisateur a réellement saisi.
            amount_ht: validation::parse_amount(&amount.get()).unwrap_or_default(),
            tva_rate: validation::parse_amount(&tva.get()).unwrap_or_default(),
            date: format!("{}T00:00:00Z", super::modals::today_iso()),
            payment_terms_days: Some(crate::settings::load().payment_terms_days),
            status: None,
        };

        leptos::task::spawn_local(async move {
            if actions::create_invoice(app, input).await {
                open.set(false);
            }
        });
    };

    view! {
        <Modal title="Générer une facture" on_close=Callback::new(move |_| open.set(false))>
            <form on:submit=submit>
                <div class="modal-body">
                    <SelectField label="Client facturé" value=client options=clients />
                    <SelectField
                        label="Type d'activité"
                        value=service_type
                        options=Signal::derive(documents::service_types)
                    />
                    <SelectField label="Taux de TVA (%)" value=tva options=Signal::derive(documents::tva_rates) />
                    <TextField
                        label="Montant Hors Taxes (HT) en €"
                        value=amount
                        placeholder="Ex: 1500.00"
                        error=error(Field::Amount)
                    />
                    <TextArea
                        label="Description de la prestation"
                        value=description
                        placeholder="Détaillez le travail effectué..."
                        error=error(Field::Description)
                    />
                    <Preview amount=amount tva=tva />
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" on:click=move |_| open.set(false)>
                        "Annuler"
                    </button>
                    <button type="submit" class="btn btn-primary">"Générer & Enregistrer"</button>
                </div>
            </form>
        </Modal>
    }
}

/// Montants affichés par l'aperçu, ou `None` tant que le montant est illisible.
///
/// L'original refaisait le calcul en flottants dans le formulaire ; l'aperçu
/// reprend ici celui du noyau, arrondi au centime, pour annoncer exactement
/// le total que l'hôte enregistrera.
fn preview(amount: &str, tva: &str) -> Option<asgard_core::money::ComputedAmounts> {
    let amount_ht = validation::parse_amount(amount)?;
    let rate = validation::parse_amount(tva)?;
    Some(asgard_core::compute_amounts(
        asgard_core::from_f64(amount_ht),
        asgard_core::from_f64(rate),
    ))
}

/// Aperçu du calcul, mis à jour à la frappe.
#[component]
fn Preview(amount: RwSignal<String>, tva: RwSignal<String>) -> impl IntoView {
    let line = "font-size: 0.85rem; margin: 0.25rem 0";
    let euros = |m| format::euros(asgard_core::to_f64(m));

    move || {
        preview(&amount.get(), &tva.get()).map(|c| {
            view! {
                <div style="margin-top: 1rem; padding: 1rem; background-color: rgba(255,255,255,0.02); border-radius: 8px; border: 1px solid var(--border-glass)">
                    <div style="font-weight: 600; font-size: 0.9rem; margin-bottom: 0.5rem; color: var(--color-gold)">
                        "Aperçu du calcul :"
                    </div>
                    <div class="flex-between" style=line>
                        <span>"Montant HT :"</span>
                        <span>{euros(c.amount_ht)}</span>
                    </div>
                    <div class="flex-between" style=line>
                        <span>{format!("Montant TVA ({}%) :", tva.get())}</span>
                        <span>{euros(c.amount_tva)}</span>
                    </div>
                    <div
                        class="flex-between"
                        style="font-size: 0.9rem; font-weight: 700; border-top: 1px solid var(--border-glass); padding-top: 0.5rem; margin-top: 0.5rem"
                    >
                        <span>"Total TTC :"</span>
                        <span>{euros(c.amount_total)}</span>
                    </div>
                </div>
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn preview_waits_for_a_readable_amount() {
        assert!(preview("", "20").is_none());
        assert!(preview("abc", "20").is_none());
    }

    /// L'aperçu arrondit comme l'hôte : 33,33 € à 5,5 % donnent 1,83 € de TVA
    /// (1,83315), et le total est la somme des deux montants arrondis.
    #[test]
    fn preview_matches_the_amounts_the_host_records() {
        let c = preview("33,33", "5.5").unwrap();
        assert_eq!(c.amount_ht, dec!(33.33));
        assert_eq!(c.amount_tva, dec!(1.83));
        assert_eq!(c.amount_total, dec!(35.16));
    }
}
