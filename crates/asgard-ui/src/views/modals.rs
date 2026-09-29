//! Fenêtres partagées — confirmation, envoi par e-mail, règlement.
//!
//! Portées depuis `App.jsx`, où elles étaient déclarées en fin de composant
//! avec leur propre copie du balisage de modale.

use asgard_ipc::PaymentInput;
use leptos::prelude::*;

use super::widgets::{Modal, SelectField, TextArea, TextField};
use crate::actions;
use crate::state::{use_app, App, EmailCompose, PaymentForm};
use crate::templates::{self, Kind};

/// Confirmation d'une action destructrice.
#[component]
pub fn ConfirmModal() -> impl IntoView {
    let app = use_app();

    view! {
        <Show when=move || app.pending.get().is_some()>
            {move || {
                let action = app.pending.get().expect("présence vérifiée par Show");
                let title = action.title().to_string();
                let message = action.message();
                let confirmed = action.clone();

                view! {
                    <Modal
                        title=title
                        max_width="450px"
                        z_index=110
                        on_close=Callback::new(move |_| app.pending.set(None))
                    >
                        <div class="modal-body">
                            <p style="font-size: 0.95rem; color: var(--text-secondary); line-height: 1.5">
                                {message}
                            </p>
                        </div>
                        <div class="modal-footer">
                            <button
                                type="button"
                                class="btn btn-secondary"
                                on:click=move |_| app.pending.set(None)
                            >
                                "Annuler"
                            </button>
                            <button
                                type="button"
                                class="btn btn-danger"
                                on:click={
                                    let confirmed = confirmed.clone();
                                    move |_| {
                                        let action = confirmed.clone();
                                        app.pending.set(None);
                                        leptos::task::spawn_local(actions::execute(app, action));
                                    }
                                }
                            >
                                "Confirmer"
                            </button>
                        </div>
                    </Modal>
                }
            }}
        </Show>
    }
}

/// Ouvre la fenêtre d'envoi pour une pièce, brouillon pré-rempli.
///
/// Le destinataire vient de la fiche client en mémoire ; s'il a été supprimé,
/// le champ reste vide et l'utilisateur le saisit.
/// Pièce dont on prépare l'envoi, telle que la ligne du tableau la connaît.
pub struct Outgoing {
    pub id: i64,
    pub number: String,
    pub description: String,
    pub total: f64,
    pub date: String,
    /// Échéance, pour les factures qui en portent une.
    pub due_date: Option<String>,
    pub client_id: Option<i64>,
    /// Raison sociale recopiée sur la pièce, utilisée si le client a disparu.
    pub company: String,
}

pub fn open_email(app: App, kind: Kind, doc: Outgoing) {
    let Outgoing { id, number, description, total, date, due_date, client_id, company } = doc;
    let settings = crate::settings::load();

    let client = app.snapshot.get_untracked().and_then(|snapshot| {
        snapshot
            .clients
            .into_iter()
            .find(|c| Some(c.id) == client_id)
    });

    let doc = templates::Document {
        number: &number,
        description: &description,
        total,
        date: &date,
        due_date: due_date.as_deref(),
        company: &company,
    };
    let draft = templates::compose(kind, &doc, client.as_ref(), &settings);

    app.email.set(Some(EmailCompose {
        kind,
        id,
        number,
        to: draft.to,
        subject: draft.subject,
        text: draft.text,
    }));
}

/// Fenêtre d'envoi : l'utilisateur relit et modifie le brouillon avant envoi.
#[component]
pub fn EmailModal() -> impl IntoView {
    let app = use_app();

    view! {
        <Show when=move || app.email.get().is_some()>
            {move || {
                let compose = app.email.get().expect("présence vérifiée par Show");
                view! { <EmailForm compose=compose /> }
            }}
        </Show>
    }
}

#[component]
fn EmailForm(compose: EmailCompose) -> impl IntoView {
    let app = use_app();
    let to = RwSignal::new(compose.to);
    let subject = RwSignal::new(compose.subject);
    let text = RwSignal::new(compose.text);
    let sending = RwSignal::new(false);

    let (kind, id) = (compose.kind, compose.id);
    let attachment = format!("{}.pdf", compose.number);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if to.get().trim().is_empty() {
            app.inform("Erreur", "L'adresse e-mail du destinataire est requise.");
            return;
        }

        sending.set(true);
        leptos::task::spawn_local(async move {
            let sent =
                actions::send_document(app, kind, id, to.get_untracked(), subject.get_untracked(), text.get_untracked())
                    .await;
            sending.set(false);
            if sent {
                app.email.set(None);
            }
        });
    };

    view! {
        <Modal title=email_title(kind) max_width="600px" on_close=Callback::new(move |_| app.email.set(None))>
            <form on:submit=submit>
                <div class="modal-body">
                    <TextField
                        label="Destinataire (E-mail)"
                        value=to
                        kind="email"
                        placeholder="client@entreprise.fr"
                    />
                    <TextField label="Objet du message" value=subject />
                    <TextArea label="Corps du message" value=text min_height="180px" />

                    // Pièce jointe. L'original lisait `invoiceNumber` même
                    // pour un devis, et affichait alors « undefined.pdf ».
                    <div style="display: flex; align-items: center; gap: 0.5rem; padding: 0.75rem 1rem; background-color: rgba(229, 169, 60, 0.05); border-radius: 8px; border: 1px solid rgba(229, 169, 60, 0.15); font-size: 0.85rem">
                        <span style="font-size: 1.15rem">"📎"</span>
                        <div style="flex-grow: 1">
                            <div style="font-weight: 600; color: var(--color-gold)">{attachment}</div>
                            <div style="color: var(--text-muted); font-size: 0.75rem">
                                "Document PDF généré automatiquement et joint au message"
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <button
                        type="button"
                        class="btn btn-secondary"
                        disabled=move || sending.get()
                        on:click=move |_| app.email.set(None)
                    >
                        "Annuler"
                    </button>
                    <button
                        type="submit"
                        class="btn btn-primary"
                        style="min-width: 120px"
                        disabled=move || sending.get()
                    >
                        {move || if sending.get() { "Envoi en cours..." } else { "Envoyer" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

/// Titre de la fenêtre d'envoi. L'original disait « la facture » pour toute
/// pièce ; un devis est désormais nommé comme tel.
fn email_title(kind: Kind) -> &'static str {
    match kind {
        Kind::Estimate => "Envoyer le devis par e-mail",
        Kind::Invoice | Kind::Reminder => "Envoyer la facture par e-mail",
    }
}

/// Fenêtre de règlement d'une facture.
#[component]
pub fn PaymentModal() -> impl IntoView {
    let app = use_app();

    view! {
        <Show when=move || app.payment.get().is_some()>
            {move || {
                let form = app.payment.get().expect("présence vérifiée par Show");
                view! { <PaymentFields form=form /> }
            }}
        </Show>
    }
}

#[component]
fn PaymentFields(form: PaymentForm) -> impl IntoView {
    let app = use_app();
    let date = RwSignal::new(today_iso());
    let method = RwSignal::new("virement".to_string());

    let methods = Signal::derive(|| {
        vec![
            ("virement".into(), "🏦 Virement bancaire".into()),
            ("carte".into(), "💳 Carte bancaire".into()),
            ("especes".into(), "💵 Espèces".into()),
            ("cheque".into(), "✉️ Chèque".into()),
        ]
    });

    let (invoice_id, number) = (form.invoice_id, form.number.clone());

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if date.get().is_empty() {
            return;
        }
        let payment = PaymentInput {
            invoice_id,
            payment_date: format!("{}T00:00:00Z", date.get()),
            payment_method: method.get(),
        };
        let number = number.clone();
        leptos::task::spawn_local(async move {
            if actions::record_payment(app, payment, number).await {
                app.payment.set(None);
            }
        });
    };

    view! {
        <Modal
            title="Enregistrer le règlement"
            max_width="450px"
            z_index=105
            on_close=Callback::new(move |_| app.payment.set(None))
        >
            <form on:submit=submit>
                <div class="modal-body">
                    <p style="margin-bottom: 1.25rem; font-size: 0.9rem; color: var(--text-secondary)">
                        "Veuillez renseigner la date et le moyen de paiement pour marquer la facture "
                        <strong>{form.number.clone()}</strong>
                        " comme payée."
                    </p>
                    <TextField label="Date d'encaissement" value=date kind="date" />
                    <SelectField label="Moyen de règlement" value=method options=methods />
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" on:click=move |_| app.payment.set(None)>
                        "Annuler"
                    </button>
                    <button
                        type="submit"
                        class="btn btn-primary"
                        style="background: linear-gradient(135deg, var(--color-gold), var(--color-gold-hover)); color: var(--bg-primary)"
                    >
                        "Valider le paiement"
                    </button>
                </div>
            </form>
        </Modal>
    }
}

/// Date du jour au format `AAAA-MM-JJ`, valeur par défaut des champs de date.
pub fn today_iso() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}
