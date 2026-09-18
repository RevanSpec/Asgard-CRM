//! Barre latérale — port de `src/components/Sidebar.jsx`.
//!
//! Le balisage reprend celui de l'original **exactement** : `<li class="nav-item">`
//! dans un `<nav class="nav-menu">`. Une première version utilisait `<button>`,
//! ce qui paraissait plus correct sémantiquement mais faisait surgir les styles
//! par défaut du navigateur — fond blanc, bordure — que la feuille de style ne
//! neutralise pas, puisqu'elle n'a jamais eu à le faire.
//!
//! Les icônes sont les tracés de `Icons.jsx`, recopiés tels quels. Le fichier
//! `public/icons.svg` n'a rien à voir : c'est le jeu d'icônes sociales livré
//! avec le gabarit Vite, jamais utilisé par l'application.

use leptos::prelude::*;
use leptos::svg;

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

/// Enveloppe commune aux icônes : mêmes attributs que dans `Icons.jsx`, dont
/// `stroke="currentColor"` qui leur fait suivre la couleur de l'élément actif.
fn frame(children: AnyView) -> AnyView {
    svg::svg()
        .attr("width", "20")
        .attr("height", "20")
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none")
        .attr("stroke", "currentColor")
        .attr("stroke-width", "2")
        .attr("stroke-linecap", "round")
        .attr("stroke-linejoin", "round")
        .child(children)
        .into_any()
}

fn path(d: &'static str) -> AnyView {
    svg::path().attr("d", d).into_any()
}

fn rect(x: &'static str, y: &'static str, w: &'static str, h: &'static str) -> AnyView {
    svg::rect()
        .attr("x", x)
        .attr("y", y)
        .attr("width", w)
        .attr("height", h)
        .into_any()
}

fn line(x1: &'static str, y1: &'static str, x2: &'static str, y2: &'static str) -> AnyView {
    svg::line()
        .attr("x1", x1)
        .attr("y1", y1)
        .attr("x2", x2)
        .attr("y2", y2)
        .into_any()
}

fn circle(cx: &'static str, cy: &'static str, r: &'static str) -> AnyView {
    svg::circle()
        .attr("cx", cx)
        .attr("cy", cy)
        .attr("r", r)
        .into_any()
}

/// Tracés repris un à un de `src/components/Icons.jsx`.
fn icon(tab: Tab) -> AnyView {
    match tab {
        Tab::Dashboard => frame(
            (
                rect("3", "3", "7", "9"),
                rect("14", "3", "7", "5"),
                rect("14", "12", "7", "9"),
                rect("3", "16", "7", "5"),
            )
                .into_any(),
        ),
        Tab::Clients => frame(
            (
                path("M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"),
                circle("9", "7", "4"),
                path("M23 21v-2a4 4 0 0 0-3-3.87"),
            )
                .into_any(),
        ),
        Tab::Estimates => frame(
            (
                path("M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"),
                svg::polyline().attr("points", "14 2 14 8 20 8").into_any(),
                line("16", "13", "8", "13"),
            )
                .into_any(),
        ),
        Tab::Invoices => frame(
            (
                path("M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"),
                svg::rect()
                    .attr("x", "8")
                    .attr("y", "2")
                    .attr("width", "8")
                    .attr("height", "4")
                    .attr("rx", "1")
                    .attr("ry", "1")
                    .into_any(),
                path("M9 14h6"),
            )
                .into_any(),
        ),
        Tab::Expenses => frame(
            (
                svg::rect()
                    .attr("x", "1")
                    .attr("y", "4")
                    .attr("width", "22")
                    .attr("height", "16")
                    .attr("rx", "2")
                    .attr("ry", "2")
                    .into_any(),
                line("1", "10", "23", "10"),
            )
                .into_any(),
        ),
        Tab::Compta => frame(
            (
                line("18", "20", "18", "10"),
                line("12", "20", "12", "4"),
                line("6", "20", "6", "14"),
            )
                .into_any(),
        ),
        Tab::Settings => frame(
            (
                circle("12", "12", "3"),
                path(
                    "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 \
                     1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 \
                     0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 \
                     1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 \
                     0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 \
                     1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 \
                     0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 \
                     1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 \
                     0 0 0-1.51 1z",
                ),
            )
                .into_any(),
        ),
    }
}
