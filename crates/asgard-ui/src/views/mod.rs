//! Vues.
//!
//! Chaque module reprend un composant React, en conservant **exactement** les
//! mêmes classes CSS. Le thème `src/index.css` est réutilisé sans modification :
//! c'est la garantie la plus simple que le rendu ne dérive pas, et cela évite de
//! refaire un travail de design qui n'avait aucune raison de recommencer.

mod clients;
mod dashboard;
mod sidebar;

pub use clients::clients;
pub use dashboard::dashboard;
pub use sidebar::Sidebar;

use leptos::prelude::*;

use crate::state::{use_app, Tab};

/// Fenêtre de message, en remplacement de `customAlert`.
#[component]
pub fn NoticeModal() -> impl IntoView {
    let app = use_app();

    view! {
        <Show when=move || app.notice.get().is_some()>
            {move || {
                let notice = app.notice.get().expect("présence vérifiée par Show");
                view! {
                    <div class="modal-overlay">
                        <div class="modal-content" style="max-width: 520px">
                            <div class="modal-header">
                                <h2>{notice.title}</h2>
                            </div>
                            <div class="modal-body">
                                // `white-space: pre-line` conserve les retours à
                                // la ligne des rapports d'import, qui listent les
                                // montants ajustés un par ligne.
                                <p style="white-space: pre-line; line-height: 1.6">
                                    {notice.body}
                                </p>
                            </div>
                            <div class="modal-footer">
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| app.notice.set(None)
                                >
                                    "Fermer"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            }}
        </Show>
    }
}

/// Espace réservé d'un onglet non encore porté.
///
/// Affiché plutôt qu'un écran vide : un onglet qui ne montre rien ressemble à
/// une panne, et l'utilisateur doit savoir où trouver la fonctionnalité en
/// attendant.
pub fn pending(tab: Tab) -> impl IntoView {
    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>{tab.label()}</h1>
                    <p>"Cet onglet n'est pas encore porté vers la nouvelle interface."</p>
                </div>
            </div>
            <div class="card-glass" style="padding: 2rem; text-align: center">
                <p style="color: var(--text-secondary); line-height: 1.7">
                    "La migration de l'interface se fait onglet par onglet. "
                    "Celui-ci reste disponible dans la version React, que "
                    "l'application continue de servir."
                </p>
            </div>
        </div>
    }
}
