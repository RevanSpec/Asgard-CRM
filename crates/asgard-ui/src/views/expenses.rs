//! Dépenses — port de `src/components/ExpensesTab.jsx` et de sa modale.

use asgard_ipc::{Expense, ExpenseInput};
use leptos::prelude::*;

use super::icons;
use super::modals::today_iso;
use super::widgets::{EmptyState, FilterBar, IconButton, Modal, SelectField, TextField};
use crate::state::{use_app, Pending};
use crate::validation::{self, Field, FieldErrors};
use crate::{actions, format};

/// Catégories proposées. Les valeurs sont celles qu'`asgard-core` reconnaît ;
/// une autre valeur y basculerait dans « Autre ».
fn categories() -> Vec<(String, String)> {
    vec![
        ("Achats".into(), "Achats de marchandises".into()),
        ("Déplacements".into(), "Déplacements / Transport".into()),
        ("Logiciels".into(), "Abonnements logiciels / Cloud".into()),
        ("Télécoms".into(), "Télécommunications / Téléphone".into()),
        ("Bureautique".into(), "Fournitures de bureau".into()),
        ("Cotisations".into(), "Autres cotisations / Assurances".into()),
        ("Autre".into(), "Autre frais".into()),
    ]
}

fn methods() -> Vec<(String, String)> {
    vec![
        ("carte".into(), "💳 Carte bancaire".into()),
        ("virement".into(), "🏦 Virement".into()),
        ("prelevement".into(), "🔄 Prélèvement automatique".into()),
        ("especes".into(), "💵 Espèces".into()),
        ("cheque".into(), "✉️ Chèque".into()),
    ]
}

#[derive(Debug, Clone, PartialEq)]
struct Draft {
    id: Option<i64>,
    merchant: String,
    amount: String,
    date: String,
    category: String,
    method: String,
    description: String,
}

impl Draft {
    fn blank() -> Self {
        Self {
            id: None,
            merchant: String::new(),
            amount: String::new(),
            date: today_iso(),
            category: "Logiciels".into(),
            method: "carte".into(),
            description: String::new(),
        }
    }

    fn from_expense(exp: &Expense) -> Self {
        Self {
            id: Some(exp.id),
            merchant: exp.merchant.clone(),
            amount: exp.amount.to_string(),
            date: exp.date.get(..10).unwrap_or(&exp.date).to_string(),
            category: exp.category.clone(),
            method: exp.payment_method.clone(),
            description: exp.description.clone(),
        }
    }
}

pub fn expenses() -> impl IntoView {
    let app = use_app();
    let search = RwSignal::new(String::new());
    let draft = RwSignal::new(None::<Draft>);

    let has_any = move || app.snapshot.with(|s| s.as_ref().is_some_and(|s| !s.expenses.is_empty()));

    let visible = move || {
        let needle = search.get().to_lowercase();
        app.snapshot
            .get()
            .map(|s| {
                s.expenses
                    .into_iter()
                    .filter(|exp| {
                        needle.is_empty()
                            || exp.merchant.to_lowercase().contains(&needle)
                            || exp.category.to_lowercase().contains(&needle)
                            || exp.description.to_lowercase().contains(&needle)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Registre des Dépenses"</h1>
                    <p>"Enregistrez vos frais et achats professionnels pour calculer votre bénéfice réel."</p>
                </div>
                <div class="flex-gap-2">
                    <button class="btn btn-primary" on:click=move |_| draft.set(Some(Draft::blank()))>
                        {icons::add()}
                        " Nouvelle Dépense"
                    </button>
                </div>
            </div>

            <FilterBar value=search placeholder="Rechercher une dépense (Fournisseur, catégorie, description...)" />

            <Show
                when=has_any
                fallback=|| view! {
                    <EmptyState message="Aucune dépense enregistrée. Cliquez sur \"Nouvelle Dépense\" pour commencer." />
                }
            >
                <div class="table-container">
                    <table class="table-glass">
                        <thead>
                            <tr>
                                <th>"Date"</th>
                                <th>"Fournisseur"</th>
                                <th>"Catégorie"</th>
                                <th>"Description"</th>
                                <th>"Moyen"</th>
                                <th>"Montant HT"</th>
                                <th class="text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            // Lignes redessinées à chaque changement : une `<For>` indexée
                            // sur l'identifiant garderait l'ancien contenu d'une ligne modifiée.
                            {move || {
                                visible()
                                    .into_iter()
                                    .map(|expense| view! { <Row expense=expense draft=draft /> })
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
fn Row(expense: Expense, draft: RwSignal<Option<Draft>>) -> impl IntoView {
    let app = use_app();
    let id = expense.id;
    let for_edit = expense.clone();

    view! {
        <tr>
            <td>{format::date(&expense.date)}</td>
            <td style="font-weight: 600; color: var(--color-gold)">{expense.merchant.clone()}</td>
            <td><span class="badge badge-secondary">{expense.category.clone()}</span></td>
            <td>{expense.description.clone()}</td>
            <td>{format::payment_method(Some(&expense.payment_method))}</td>
            <td style="font-weight: 600; color: #FF6B8B">{format::euros(expense.amount)}</td>
            <td class="text-right">
                <div class="flex-gap-2" style="justify-content: flex-end">
                    <IconButton
                        icon=icons::edit
                        title="Modifier la dépense"
                        on_click=Callback::new(move |_| draft.set(Some(Draft::from_expense(&for_edit))))
                    />
                    <IconButton
                        icon=icons::delete
                        title="Supprimer la dépense"
                        danger=true
                        on_click=Callback::new(move |_| app.pending.set(Some(Pending::DeleteExpense(id))))
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
    let id = initial.id;

    let merchant = RwSignal::new(initial.merchant);
    let amount = RwSignal::new(initial.amount);
    let date = RwSignal::new(initial.date);
    let category = RwSignal::new(initial.category);
    let method = RwSignal::new(initial.method);
    let description = RwSignal::new(initial.description);
    let errors = RwSignal::new(FieldErrors::default());
    let error = move |field| Signal::derive(move || errors.with(|e| e.get(field)));

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let found = validation::validate_expense(&merchant.get(), &amount.get());
        let valid = found.is_empty();
        errors.set(found);
        if !valid {
            return;
        }

        let input = ExpenseInput {
            id,
            date: format!("{}T00:00:00Z", date.get()),
            merchant: merchant.get(),
            category: category.get(),
            amount: validation::parse_amount(&amount.get()).unwrap_or_default(),
            description: description.get(),
            payment_method: method.get(),
        };

        leptos::task::spawn_local(async move {
            if actions::save_expense(app, input).await {
                draft.set(None);
            }
        });
    };

    view! {
        <Modal
            title=if is_edit { "Modifier la dépense" } else { "Enregistrer une dépense" }
            on_close=Callback::new(move |_| draft.set(None))
        >
            <form on:submit=submit>
                <div class="modal-body">
                    <TextField
                        label="Fournisseur"
                        value=merchant
                        placeholder="Ex: OVHcloud, SNCF..."
                        error=error(Field::Merchant)
                    />
                    <TextField
                        label="Montant (€)"
                        value=amount
                        placeholder="Ex: 50.00"
                        error=error(Field::Amount)
                    />
                    <TextField label="Date de la dépense" value=date kind="date" />
                    <SelectField label="Catégorie" value=category options=Signal::derive(categories) />
                    <SelectField label="Moyen de paiement" value=method options=Signal::derive(methods) />
                    <TextField
                        label="Description (facultative)"
                        value=description
                        placeholder="Ex: VPS mois de juin..."
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Les valeurs de catégorie doivent correspondre à celles que le noyau
    /// reconnaît : une faute ici ferait basculer la dépense dans « Autre ».
    #[test]
    fn categories_match_the_core() {
        let keys: Vec<String> = categories().into_iter().map(|(k, _)| k).collect();
        assert_eq!(keys, asgard_core::reporting::EXPENSE_CATEGORIES);
    }

    /// Chaque moyen proposé doit avoir son propre libellé en tableau. Sans
    /// cela, un prélèvement s'afficherait comme un virement — ce qui arrivait
    /// avant que le formateur ne connaisse `prelevement`.
    #[test]
    fn every_payment_method_has_its_own_label() {
        for (key, _) in methods() {
            let label = crate::format::payment_method(Some(&key));
            if key == "virement" {
                assert_eq!(label, "🏦 Virement");
            } else {
                assert_ne!(label, "🏦 Virement", "« {key} » retombe sur le libellé par défaut");
            }
        }
    }
}
