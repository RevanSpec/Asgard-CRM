//! Clients — port de `src/components/ClientsTab.jsx` et de ses modales.
//!
//! Sert de gabarit aux onglets restants : recherche, tableau, formulaire, appel
//! à l'hôte, rechargement. Le schéma se répète pour les factures, devis et
//! dépenses.

use asgard_ipc::{Client, ClientInput};
use leptos::prelude::*;
use serde::Serialize;

use crate::ipc;
use crate::state::use_app;

/// Saisie en cours. `id` absent signifie création.
#[derive(Debug, Clone, Default, PartialEq)]
struct Draft {
    id: Option<i64>,
    company_name: String,
    contact_name: String,
    email: String,
    phone: String,
    address: String,
}

impl From<&Client> for Draft {
    fn from(client: &Client) -> Self {
        Self {
            id: Some(client.id),
            company_name: client.company_name.clone(),
            contact_name: client.contact_name.clone(),
            email: client.email.clone(),
            phone: client.phone.clone(),
            address: client.address.clone(),
        }
    }
}

pub fn clients() -> impl IntoView {
    let app = use_app();
    let search = RwSignal::new(String::new());
    let draft = RwSignal::new(None::<Draft>);

    let visible = move || {
        let needle = search.get().to_lowercase();
        app.snapshot
            .get()
            .map(|snapshot| {
                snapshot
                    .clients
                    .into_iter()
                    .filter(|client| {
                        needle.is_empty()
                            || client.company_name.to_lowercase().contains(&needle)
                            || client.contact_name.to_lowercase().contains(&needle)
                            || client.email.to_lowercase().contains(&needle)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Clients"</h1>
                    <p>"Fiches complètes de vos clients et historique d'affaires."</p>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| draft.set(Some(Draft::default()))
                >
                    "+ Ajouter un client"
                </button>
            </div>

            <input
                class="form-input"
                style="max-width: 420px; margin-bottom: 1.25rem"
                placeholder="Rechercher par nom, contact ou e-mail…"
                prop:value=move || search.get()
                on:input=move |ev| search.set(event_target_value(&ev))
            />

            <div class="table-container">
                <table class="table-glass">
                    <thead>
                        <tr>
                            <th>"Entreprise"</th>
                            <th>"Contact"</th>
                            <th>"Email"</th>
                            <th>"Téléphone"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=visible
                            key=|client| client.id
                            let:client
                        >
                            <Row client=client draft=draft />
                        </For>
                    </tbody>
                </table>

                <Show when=move || visible().is_empty()>
                    <p style="text-align: center; padding: 2rem; color: var(--text-secondary)">
                        "Aucun client ne correspond à cette recherche."
                    </p>
                </Show>
            </div>

            <Show when=move || draft.get().is_some()>
                <Form draft=draft />
            </Show>
        </div>
    }
}

#[component]
fn Row(client: Client, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let id = client.id;
    let editable = client.clone();

    let remove = move |_| {
        leptos::task::spawn_local(async move {
            #[derive(Serialize)]
            struct Args {
                id: i64,
            }

            match ipc::invoke::<_, ()>("delete_client", &Args { id }).await {
                // Les pièces du client restent, détachées : c'est ce que
                // l'interface annonce depuis toujours, et la clé étrangère le
                // garantit côté base.
                Ok(()) => app.reload().await,
                Err(error) => app.report(error),
            }
        });
    };

    view! {
        <tr>
            <td><strong>{client.company_name}</strong></td>
            <td>{client.contact_name}</td>
            <td>{client.email}</td>
            <td>{client.phone}</td>
            <td>
                <div class="flex-gap-2">
                    <button
                        class="btn btn-secondary btn-sm"
                        on:click=move |_| draft.set(Some(Draft::from(&editable)))
                    >
                        "Modifier"
                    </button>
                    <button class="btn btn-danger btn-sm" on:click=remove>
                        "Supprimer"
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn Form(draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let errors = RwSignal::new(Vec::<&'static str>::new());

    let field = move |read: fn(&Draft) -> String, write: fn(&mut Draft, String)| {
        (
            Signal::derive(move || draft.get().map(|d| read(&d)).unwrap_or_default()),
            move |value: String| {
                draft.update(|slot| {
                    if let Some(current) = slot {
                        write(current, value);
                    }
                });
            },
        )
    };

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(current) = draft.get() else { return };

        // La validation reste ici : elle doit répondre à la frappe, sans
        // aller-retour, et ses messages sont du texte d'interface.
        let mut found = Vec::new();
        if current.company_name.trim().is_empty() {
            found.push("Nom d'entreprise requis");
        }
        if current.contact_name.trim().is_empty() {
            found.push("Nom du contact requis");
        }
        if !current.email.contains('@') {
            found.push("Format e-mail invalide");
        }
        if current.phone.chars().filter(char::is_ascii_digit).count() != 10 {
            found.push("Le numéro doit faire exactement 10 chiffres");
        }

        errors.set(found.clone());
        if !found.is_empty() {
            return;
        }

        leptos::task::spawn_local(async move {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
                client: ClientInput,
            }

            let args = Args {
                client: ClientInput {
                    id: current.id,
                    company_name: current.company_name,
                    contact_name: current.contact_name,
                    email: current.email,
                    phone: current.phone,
                    address: current.address,
                },
            };

            match ipc::invoke::<_, i64>("save_client", &args).await {
                Ok(_) => {
                    draft.set(None);
                    app.reload().await;
                }
                Err(error) => app.report(error),
            }
        });
    };

    let (company, set_company) = field(|d| d.company_name.clone(), |d, v| d.company_name = v);
    let (contact, set_contact) = field(|d| d.contact_name.clone(), |d, v| d.contact_name = v);
    let (email, set_email) = field(|d| d.email.clone(), |d, v| d.email = v);
    let (phone, set_phone) = field(|d| d.phone.clone(), |d, v| d.phone = v);
    let (address, set_address) = field(|d| d.address.clone(), |d, v| d.address = v);

    let is_edit = move || draft.get().and_then(|d| d.id).is_some();

    view! {
        <div class="modal-overlay">
            <div class="modal-content">
                <div class="modal-header">
                    <h2>
                        {move || if is_edit() { "Modifier le client" } else { "Ajouter un client" }}
                    </h2>
                    <button
                        class="btn btn-secondary btn-icon-only"
                        style="border-radius: 50%"
                        on:click=move |_| draft.set(None)
                    >
                        "✕"
                    </button>
                </div>

                <form on:submit=submit>
                    <div class="modal-body">
                        <Field label="Nom de l'entreprise" value=company on_input=set_company />
                        <Field label="Nom du contact" value=contact on_input=set_contact />
                        <Field label="Email" value=email on_input=set_email />
                        <Field label="Téléphone" value=phone on_input=set_phone />
                        <Field label="Adresse" value=address on_input=set_address />

                        <Show when=move || !errors.get().is_empty()>
                            <ul class="error-list">
                                {move || {
                                    errors
                                        .get()
                                        .into_iter()
                                        .map(|message| view! { <li class="error-text">{message}</li> })
                                        .collect_view()
                                }}
                            </ul>
                        </Show>
                    </div>

                    <div class="modal-footer">
                        <button
                            type="button"
                            class="btn btn-secondary"
                            on:click=move |_| draft.set(None)
                        >
                            "Annuler"
                        </button>
                        <button type="submit" class="btn btn-primary">"Enregistrer"</button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
fn Field(
    label: &'static str,
    value: Signal<String>,
    on_input: impl Fn(String) + 'static,
) -> impl IntoView {
    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <input
                class="form-input"
                prop:value=move || value.get()
                on:input=move |ev| on_input(event_target_value(&ev))
            />
        </div>
    }
}
