//! Clients — port de `src/components/ClientsTab.jsx` et de sa modale.

use asgard_ipc::{Client, ClientInput};
use leptos::prelude::*;

use super::icons;
use super::widgets::{EmptyState, IconButton, Modal, SearchBox, TextField};
use crate::state::{use_app, Pending};
use crate::validation::{self, ClientForm, Field, FieldErrors};
use crate::actions;

/// Fiche en cours de saisie. `id` absent signifie création.
#[derive(Debug, Clone, Default, PartialEq)]
struct Draft {
    id: Option<i64>,
    company_name: String,
    contact_name: String,
    email: String,
    phone: String,
    address: String,
    siren: String,
    vat_number: String,
    delivery_address: String,
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
            siren: client.siren.clone(),
            vat_number: client.vat_number.clone(),
            delivery_address: client.delivery_address.clone(),
        }
    }
}

/// Filtre de la recherche : entreprise, contact ou e-mail, comme l'original.
fn matches(client: &Client, needle: &str) -> bool {
    needle.is_empty()
        || client.company_name.to_lowercase().contains(needle)
        || client.contact_name.to_lowercase().contains(needle)
        || client.email.to_lowercase().contains(needle)
}

pub fn clients() -> impl IntoView {
    let app = use_app();
    let search = RwSignal::new(String::new());
    let draft = RwSignal::new(None::<Draft>);

    let visible = move || {
        let needle = search.get().to_lowercase();
        app.snapshot
            .get()
            .map(|s| s.clients.into_iter().filter(|c| matches(c, &needle)).collect::<Vec<_>>())
            .unwrap_or_default()
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Fichiers Clients"</h1>
                    <p>"Gérez les coordonnées de vos clients et partenaires commerciaux."</p>
                </div>
                <div class="flex-gap-2">
                    <SearchBox value=search placeholder="Rechercher un client..." />
                    <button class="btn btn-primary" on:click=move |_| draft.set(Some(Draft::default()))>
                        {icons::add()}
                        " Nouveau Client"
                    </button>
                </div>
            </div>

            <div class="card-glass" style="padding: 0.5rem 0">
                <Show
                    when=move || !visible().is_empty()
                    fallback=|| view! { <EmptyState message="Aucun client trouvé." rune=true /> }
                >
                    <div class="table-container">
                        <table class="table-glass">
                            <thead>
                                <tr>
                                    <th>"Entreprise"</th>
                                    <th>"Contact"</th>
                                    <th>"Email"</th>
                                    <th>"Téléphone"</th>
                                    <th>"Adresse"</th>
                                    <th class="text-right">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                // Lignes redessinées à chaque changement : une `<For>` indexée
                                // sur l'identifiant garderait l'ancien contenu d'une ligne modifiée.
                                {move || {
                                    visible()
                                        .into_iter()
                                        .map(|client| view! { <Row client=client draft=draft /> })
                                        .collect_view()
                                }}
                            </tbody>
                        </table>
                    </div>
                </Show>
            </div>

            <Show when=move || draft.get().is_some()>
                {move || draft.get().map(|current| view! { <Form initial=current draft=draft /> })}
            </Show>
        </div>
    }
}

#[component]
fn Row(client: Client, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let id = client.id;
    let editable = Draft::from(&client);

    view! {
        <tr>
            <td style="font-weight: 700; color: var(--color-gold)">{client.company_name}</td>
            <td>{client.contact_name}</td>
            <td>{client.email}</td>
            <td style="white-space: nowrap">{client.phone}</td>
            <td
                style="max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap"
                title=client.address.clone()
            >
                {client.address.clone()}
            </td>
            <td class="text-right">
                <div class="flex-gap-2" style="justify-content: flex-end">
                    <IconButton
                        icon=icons::edit
                        title="Modifier"
                        on_click=Callback::new(move |_| draft.set(Some(editable.clone())))
                    />
                    // Comme dans l'original, la suppression passe par une
                    // confirmation : les pièces du client restent, détachées.
                    <IconButton
                        icon=icons::delete
                        title="Supprimer"
                        danger=true
                        on_click=Callback::new(move |_| app.pending.set(Some(Pending::DeleteClient(id))))
                    />
                </div>
            </td>
        </tr>
    }
}

#[component]
fn Form(initial: Draft, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let id = initial.id;
    let is_edit = id.is_some();

    let company = RwSignal::new(initial.company_name);
    let contact = RwSignal::new(initial.contact_name);
    let email = RwSignal::new(initial.email);
    let phone = RwSignal::new(initial.phone);
    let address = RwSignal::new(initial.address);
    let siren = RwSignal::new(initial.siren);
    let vat_number = RwSignal::new(initial.vat_number);
    let delivery_address = RwSignal::new(initial.delivery_address);
    let errors = RwSignal::new(FieldErrors::default());
    let error = move |field| Signal::derive(move || errors.with(|e| e.get(field)));

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let (company, contact, email, phone, address) =
            (company.get(), contact.get(), email.get(), phone.get(), address.get());

        let found = validation::validate_client(&ClientForm {
            company_name: &company,
            contact_name: &contact,
            email: &email,
            phone: &phone,
            address: &address,
        });
        let valid = found.is_empty();
        errors.set(found);
        if !valid {
            return;
        }

        // La raison sociale est recopiée sur les pièces côté hôte, dans la
        // même transaction que la mise à jour du client.
        let input = ClientInput {
            id,
            company_name: company,
            contact_name: contact,
            email,
            phone,
            address,
            siren: siren.get().trim().to_string(),
            vat_number: vat_number.get().trim().to_string(),
            delivery_address: delivery_address.get().trim().to_string(),
        };
        leptos::task::spawn_local(async move {
            if actions::save_client(app, input).await {
                draft.set(None);
            }
        });
    };

    view! {
        <Modal
            title=if is_edit { "Modifier le client" } else { "Ajouter un client" }
            on_close=Callback::new(move |_| draft.set(None))
        >
            <form on:submit=submit>
                <div class="modal-body">
                    <TextField
                        label="Nom de l'entreprise"
                        value=company
                        placeholder="Ex: Stark Industries"
                        error=error(Field::CompanyName)
                    />
                    <TextField
                        label="Nom du contact"
                        value=contact
                        placeholder="Ex: Pepper Potts"
                        error=error(Field::ContactName)
                    />
                    <TextField
                        label="Email"
                        value=email
                        placeholder="Ex: contact@entreprise.fr"
                        error=error(Field::Email)
                    />
                    <TextField
                        label="Numéro de téléphone"
                        value=phone
                        placeholder="Ex: 06 12 34 56 78"
                        error=error(Field::Phone)
                        format=validation::format_phone
                    />
                    <TextField
                        label="Adresse de l'entreprise"
                        value=address
                        placeholder="Ex: 12 Rue de la Paix, 75002 Paris"
                        error=error(Field::Address)
                    />

                    // Mentions que la facturation électronique rendra
                    // obligatoires (décret n° 2022-1299). Facultatives ici : un
                    // client particulier n'a pas de SIREN, et une prestation ne
                    // se livre nulle part.
                    <div class="settings-section-title" style="margin-top: 0.5rem">
                        "Mentions de facturation"
                    </div>
                    <TextField
                        label="SIREN"
                        value=siren
                        placeholder="Ex: 839 204 123 — vide pour un particulier"
                    />
                    <TextField
                        label="N° de TVA intracommunautaire"
                        value=vat_number
                        placeholder="Ex: FR 12 839204123"
                    />
                    <TextField
                        label="Adresse de livraison"
                        value=delivery_address
                        placeholder="Seulement si elle diffère de l'adresse ci-dessus"
                    />
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" on:click=move |_| draft.set(None)>
                        "Annuler"
                    </button>
                    <button type="submit" class="btn btn-primary">
                        {if is_edit { "Modifier" } else { "Ajouter" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> Client {
        Client {
            id: 1,
            company_name: "Stark Industries".into(),
            contact_name: "Pepper Potts".into(),
            email: "pepper@stark.com".into(),
            phone: "06 11 22 33 44".into(),
            address: "12 rue de Paris".into(),
            siren: "839204123".into(),
            vat_number: String::new(),
            delivery_address: String::new(),
            created_at: String::new(),
        }
    }

    /// La recherche porte sur l'entreprise, le contact et l'e-mail — pas sur
    /// le téléphone ni l'adresse, comme dans l'original.
    #[test]
    fn search_covers_company_contact_and_email_only() {
        let c = client();
        assert!(matches(&c, ""));
        assert!(matches(&c, "stark ind"));
        assert!(matches(&c, "pepper"));
        assert!(matches(&c, "@stark.com"));
        assert!(!matches(&c, "06 11"));
        assert!(!matches(&c, "rue de paris"));
    }
}
