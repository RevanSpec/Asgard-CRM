//! Avoirs — la pièce qui corrige une facture émise.
//!
//! Une facture émise ne se modifie pas : elle s'annule ou se corrige par un
//! avoir. L'application le disait déjà dans ses messages de suppression sans
//! offrir le moyen de le faire ; c'est cet écran-là qui manquait.

use asgard_ipc::{CreditNote, CreditNoteInput, Invoice};
use leptos::prelude::*;

use super::icons;
use super::modals::today_iso;
use super::widgets::{IconButton, Modal, TextArea, TextField};
use crate::state::{use_app, App};
use crate::templates::Kind;
use crate::{actions, format, validation};

/// Avoir en cours de saisie, rattaché à sa facture.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub invoice_id: i64,
    pub invoice_number: String,
    pub amount_ht: String,
    pub description: String,
    pub date: String,
    pub refunded: bool,
    pub refunded_on: String,
}

impl Draft {
    /// Avoir total, prêt à être réduit : le cas courant est l'annulation.
    pub fn for_invoice(invoice: &Invoice) -> Self {
        Self {
            invoice_id: invoice.id,
            invoice_number: invoice.invoice_number.clone(),
            amount_ht: invoice.amount_ht.to_string(),
            description: String::new(),
            date: today_iso(),
            refunded: false,
            refunded_on: today_iso(),
        }
    }
}

/// Un avoir ne s'émet que sur une facture émise : un brouillon se modifie ou
/// se supprime.
pub fn can_be_credited(status: &str) -> bool {
    status == "envoyee" || status == "payee"
}

#[component]
pub fn CreditForm(initial: Draft, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let amount = RwSignal::new(initial.amount_ht);
    let description = RwSignal::new(initial.description);
    let date = RwSignal::new(initial.date);
    let refunded = RwSignal::new(initial.refunded);
    let refunded_on = RwSignal::new(initial.refunded_on);
    let error = RwSignal::new(None::<&'static str>);
    let (invoice_id, number) = (initial.invoice_id, initial.invoice_number);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let Some(amount_ht) = validation::parse_amount(&amount.get()).filter(|v| *v > 0.0) else {
            error.set(Some("Montant supérieur à 0 requis"));
            return;
        };
        error.set(None);

        let input = CreditNoteInput {
            invoice_id,
            description: description.get(),
            amount_ht,
            date: format!("{}T00:00:00Z", date.get()),
            // Le remboursement est facultatif : un avoir peut rester à valoir
            // sur une prochaine facture.
            refunded_on: refunded
                .get()
                .then(|| format!("{}T00:00:00Z", refunded_on.get())),
        };

        leptos::task::spawn_local(async move {
            if actions::create_credit_note(app, input).await {
                draft.set(None);
            }
        });
    };

    view! {
        <Modal
            title="Émettre un avoir"
            max_width="520px"
            on_close=Callback::new(move |_| draft.set(None))
        >
            <form on:submit=submit>
                <div class="modal-body">
                    <p style="margin-bottom: 1.25rem; font-size: 0.9rem; color: var(--text-secondary)">
                        "Cet avoir corrige la facture "
                        <strong>{number.clone()}</strong>
                        ". Le taux de TVA et le type d'activité sont repris de la facture."
                    </p>

                    <TextField
                        label="Montant HT de l'avoir (€)"
                        value=amount
                        placeholder="Ex: 1500.00"
                        error=Signal::derive(move || error.get())
                    />
                    <TextField label="Date de l'avoir" value=date kind="date" />
                    <TextArea
                        label="Motif"
                        value=description
                        placeholder="Ex: prestation annulée, geste commercial…"
                    />

                    <div class="form-group" style="display: flex; align-items: center; gap: 0.75rem">
                        <input
                            type="checkbox"
                            id="creditRefunded"
                            style="transform: scale(1.25); accent-color: var(--color-gold); cursor: pointer"
                            prop:checked=move || refunded.get()
                            on:change=move |ev| refunded.set(event_target_checked(&ev))
                        />
                        <label for="creditRefunded" style="font-weight: 600; font-size: 0.9rem; cursor: pointer">
                            "Montant déjà remboursé au client"
                        </label>
                    </div>
                    {move || {
                        refunded.get().then(|| {
                            view! { <TextField label="Date du remboursement" value=refunded_on kind="date" /> }
                        })
                    }}
                    <span class="metric-subtext" style="display: block; line-height: 1.5">
                        "Tant que le montant n'est pas remboursé, l'avoir diminue le chiffre d'affaires facturé, pas l'encaissé : les cotisations suivent l'argent."
                    </span>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary" on:click=move |_| draft.set(None)>
                        "Annuler"
                    </button>
                    <button type="submit" class="btn btn-primary">"Émettre l'avoir"</button>
                </div>
            </form>
        </Modal>
    }
}

/// Liste des avoirs émis, sous le tableau des factures.
#[component]
pub fn CreditNotes() -> impl IntoView {
    let app = use_app();
    let rows = move || {
        app.snapshot
            .with(|s| s.as_ref().map(|s| s.credit_notes.clone()).unwrap_or_default())
    };

    view! {
        <Show when=move || !rows().is_empty()>
            <div class="card-glass" style="padding: 0.5rem 0; margin-top: 1.5rem">
                <h3 style="font-family: var(--font-title); margin: 0.75rem 1.5rem 1rem">
                    "Avoirs émis"
                </h3>
                <div class="table-container">
                    <table class="table-glass">
                        <thead>
                            <tr>
                                <th>"Numéro"</th>
                                <th>"Facture corrigée"</th>
                                <th>"Date"</th>
                                <th>"Motif"</th>
                                <th>"Montant TTC"</th>
                                <th>"Remboursement"</th>
                                <th class="text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                rows()
                                    .into_iter()
                                    .map(|credit| view! { <Row credit=credit app=app /> })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn Row(credit: CreditNote, app: App) -> impl IntoView {
    let id = credit.id;

    view! {
        <tr>
            <td style="font-weight: 600; color: var(--color-gold)">{credit.credit_number.clone()}</td>
            <td>{credit.invoice_number.clone()}</td>
            <td>{format::date(&credit.date)}</td>
            <td>{credit.description.clone()}</td>
            // Le signe rappelle qu'un avoir se retranche du chiffre d'affaires.
            <td style="font-weight: 600; color: #FF6B8B">
                {format!("- {}", format::euros(credit.amount_total))}
            </td>
            <td>
                {match credit.refunded_on.as_deref() {
                    Some(iso) => view! {
                        <span class="badge badge-success">{format!("Remboursé le {}", format::date(iso))}</span>
                    }
                        .into_any(),
                    None => view! { <span class="badge badge-secondary">"À valoir"</span> }.into_any(),
                }}
            </td>
            <td class="text-right">
                <div class="flex-gap-2" style="justify-content: flex-end">
                    <IconButton
                        icon=icons::download
                        title="Télécharger le PDF"
                        on_click=Callback::new(move |_| {
                            leptos::task::spawn_local(actions::export_pdf(app, Kind::Credit, id))
                        })
                    />
                </div>
            </td>
        </tr>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un brouillon se corrige en le modifiant ; un avoir sur un brouillon
    /// n'aurait aucun sens, et consommerait un numéro pour rien.
    #[test]
    fn only_issued_invoices_can_be_credited() {
        assert!(can_be_credited("envoyee"));
        assert!(can_be_credited("payee"));
        assert!(!can_be_credited("brouillon"));
    }
}
