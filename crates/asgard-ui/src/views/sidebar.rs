//! Barre latérale — port de `src/components/Sidebar.jsx`.
//!
//! Le balisage reprend celui de l'original **exactement** : `<li class="nav-item">`
//! dans un `<nav class="nav-menu">`. Une première version utilisait `<button>`,
//! ce qui paraissait plus correct sémantiquement mais faisait surgir les styles
//! par défaut du navigateur — fond blanc, bordure — que la feuille de style ne
//! neutralise pas, puisqu'elle n'a jamais eu à le faire.
//!
//! Les icônes viennent de `icons.rs`, généré depuis `Icons.jsx`. Le fichier
//! `public/icons.svg` n'a rien à voir : c'est le jeu d'icônes sociales livré
//! avec le gabarit Vite, jamais utilisé par l'application.

use leptos::prelude::*;

use crate::state::{use_app, Tab};

#[component]
pub fn Sidebar() -> impl IntoView {
    let app = use_app();

    view! {
        <aside class="sidebar">
            <div class="logo-container">
                // ᛟ (othala), la rune qui sert de marque à l'application.
                <div class="logo-icon">"ᛟ"</div>
                <span class="logo-text">"ASGARD CRM"</span>
            </div>

            <nav class="nav-menu">
                {Tab::ALL
                    .into_iter()
                    .map(|tab| {
                        let active = move || app.tab.get() == tab;
                        view! {
                            <li
                                class="nav-item"
                                class:active=active
                                // L'original n'est pas atteignable au clavier ;
                                // ces deux attributs y remédient sans changer le
                                // rendu d'un pixel.
                                role="button"
                                tabindex="0"
                                on:click=move |_| app.tab.set(tab)
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" || ev.key() == " " {
                                        ev.prevent_default();
                                        app.tab.set(tab);
                                    }
                                }
                            >
                                {icon(tab)}
                                <span>{tab.label()}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </nav>

            <div class="sidebar-footer">
                <p>"© 2026 Asgard CRM"</p>
                <p style="color: var(--color-gold); margin-top: 0.25rem; font-weight: 600">
                    "Mode Bureau Local"
                </p>
            </div>
        </aside>
    }
}

/// Icône d'un onglet. Correspondance reprise de `Sidebar.jsx`, où Devis et
/// Factures avaient été intervertis lors d'une première copie manuelle.
fn icon(tab: Tab) -> AnyView {
    use super::icons;

    match tab {
        Tab::Dashboard => icons::dashboard(),
        Tab::Clients => icons::clients(),
        Tab::Estimates => icons::estimates(),
        Tab::Invoices => icons::invoices(),
        Tab::Expenses => icons::expenses(),
        Tab::Compta => icons::accounting(),
        Tab::Settings => icons::settings(),
    }
}
