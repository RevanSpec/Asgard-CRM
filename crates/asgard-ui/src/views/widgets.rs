//! Éléments d'interface partagés par les écrans.
//!
//! Dans la version React, chaque onglet et chaque modale recopiait son propre
//! balisage de champ, de fenêtre et de recherche — six modales dans `App.jsx`
//! reprenaient la même structure à la main. Les réunir ici garantit qu'elles
//! restent identiques entre elles, ce qu'aucune revue ne vérifiait auparavant.

use leptos::prelude::*;

use super::icons;

/// Fenêtre modale : en-tête avec bouton de fermeture, contenu libre.
///
/// `max_width` et `z_index` reprennent les valeurs que chaque modale de
/// l'original fixait en ligne (450 px pour le règlement, 110 pour les
/// dialogues qui s'ouvrent par-dessus une autre fenêtre…).
#[component]
pub fn Modal(
    #[prop(into)] title: Signal<String>,
    on_close: Callback<()>,
    #[prop(optional)] max_width: Option<&'static str>,
    #[prop(optional)] z_index: Option<u32>,
    children: Children,
) -> impl IntoView {
    let overlay = z_index.map(|z| format!("z-index: {z}")).unwrap_or_default();
    let content = max_width.map(|w| format!("max-width: {w}")).unwrap_or_default();

    view! {
        <div class="modal-overlay" style=overlay>
            <div class="modal-content" style=content>
                <div class="modal-header">
                    <h2>{move || title.get()}</h2>
                    <button
                        type="button"
                        class="btn btn-secondary btn-icon-only"
                        style="border-radius: 50%"
                        aria-label="Fermer"
                        on:click=move |_| on_close.run(())
                    >
                        "✕"
                    </button>
                </div>
                {children()}
            </div>
        </div>
    }
}

/// Champ de recherche avec son icône, comme dans l'en-tête des listes.
#[component]
pub fn SearchBox(value: RwSignal<String>, placeholder: &'static str) -> impl IntoView {
    view! {
        <div class="search-container">
            <span class="search-icon">{icons::search()}</span>
            <input
                type="text"
                class="search-input"
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            />
        </div>
    }
}

/// Recherche pleine largeur sous l'en-tête, propre aux Devis et aux Dépenses.
#[component]
pub fn FilterBar(value: RwSignal<String>, placeholder: &'static str) -> impl IntoView {
    view! {
        <div class="filter-bar">
            <div style="position: relative; flex-grow: 1">
                <span class="search-icon">{icons::search()}</span>
                <input
                    type="text"
                    class="search-input"
                    placeholder=placeholder
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev))
                />
            </div>
        </div>
    }
}

/// Champ texte lié à un signal.
#[component]
pub fn TextField(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional, default = "text")] kind: &'static str,
    #[prop(optional)] error: Option<Signal<Option<&'static str>>>,
    /// Mise en forme appliquée à chaque frappe (le téléphone, par exemple).
    #[prop(optional)] format: Option<fn(&str) -> String>,
) -> impl IntoView {
    let has_error = move || error.and_then(|e| e.get()).is_some();
    let write = move |raw: String| {
        value.set(match format {
            Some(f) => f(&raw),
            None => raw,
        })
    };

    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <input
                type=kind
                class="form-input"
                class:error=has_error
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| write(event_target_value(&ev))
            />
            {move || {
                error
                    .and_then(|e| e.get())
                    .map(|message| view! { <span class="error-text">{message}</span> })
            }}
        </div>
    }
}

/// Zone de texte multiligne, extensible verticalement.
#[component]
pub fn TextArea(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional, default = "80px")] min_height: &'static str,
    #[prop(optional)] error: Option<Signal<Option<&'static str>>>,
) -> impl IntoView {
    let has_error = move || error.and_then(|e| e.get()).is_some();

    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <textarea
                class="form-input"
                class:error=has_error
                style=format!("min-height: {min_height}; resize: vertical; font-family: inherit; line-height: 1.5")
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            />
            {move || {
                error
                    .and_then(|e| e.get())
                    .map(|message| view! { <span class="error-text">{message}</span> })
            }}
        </div>
    }
}

/// Liste déroulante : chaque option est une paire (valeur, libellé).
#[component]
pub fn SelectField(
    label: &'static str,
    value: RwSignal<String>,
    #[prop(into)] options: Signal<Vec<(String, String)>>,
) -> impl IntoView {
    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <select
                class="form-input"
                prop:value=move || value.get()
                on:change=move |ev| value.set(event_target_value(&ev))
            >
                {move || {
                    options
                        .get()
                        .into_iter()
                        .map(|(key, text)| {
                            let selected = key == value.get_untracked();
                            view! { <option value=key selected=selected>{text}</option> }
                        })
                        .collect_view()
                }}
            </select>
        </div>
    }
}

/// Bouton d'action sous forme d'icône seule, avec son libellé en infobulle et
/// pour les lecteurs d'écran.
#[component]
pub fn IconButton(
    icon: fn() -> AnyView,
    title: &'static str,
    on_click: Callback<()>,
    #[prop(optional)] danger: bool,
) -> impl IntoView {
    let class = if danger {
        "btn btn-danger btn-icon-only"
    } else {
        "btn btn-secondary btn-icon-only"
    };

    view! {
        <button
            type="button"
            class=class
            title=title
            aria-label=title
            on:click=move |_| on_click.run(())
        >
            {icon()}
        </button>
    }
}

/// Bouton d'action textuel à liseré doré, utilisé pour « Régler », « Relancer »
/// ou « Facturer » dans les lignes de tableau.
#[component]
pub fn GoldButton(label: &'static str, title: &'static str, on_click: Callback<()>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="btn btn-secondary"
            style="padding: 0.4rem 0.75rem; font-size: 0.8rem; border: 1px solid var(--color-gold); color: var(--color-gold)"
            title=title
            on:click=move |_| on_click.run(())
        >
            {label}
        </button>
    }
}

/// État vide d'une liste. Seuls Clients et Factures portaient la rune de la
/// marque dans l'original ; Devis et Dépenses affichaient le message seul.
#[component]
pub fn EmptyState(message: &'static str, #[prop(optional)] rune: bool) -> impl IntoView {
    view! {
        <div class="empty-state">
            {rune.then(|| view! { <div class="empty-state-icon">"ᛟ"</div> })}
            <p>{message}</p>
        </div>
    }
}
