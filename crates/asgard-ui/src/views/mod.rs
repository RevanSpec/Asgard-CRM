//! Vues.
//!
//! Chaque module reprend un composant React, en conservant **exactement** les
//! mêmes classes CSS. Le thème de la version React (`style/index.css`, venu de
//! `src/`) est réutilisé sans modification :
//! c'est la garantie la plus simple que le rendu ne dérive pas, et cela évite de
//! refaire un travail de design qui n'avait aucune raison de recommencer.

mod clients;
mod compta;
mod dashboard;
mod documents;
mod estimates;
mod expenses;
pub mod icons;
mod invoices;
pub mod modals;
mod settings_view;
mod sidebar;
mod widgets;

pub use clients::clients;
pub use compta::compta;
pub use dashboard::dashboard;
pub use estimates::estimates;
pub use expenses::expenses;
pub use invoices::invoices;
pub use modals::{ConfirmModal, EmailModal, PaymentModal};
pub use settings_view::settings;
pub use sidebar::Sidebar;

use leptos::prelude::*;

use crate::state::use_app;
use widgets::Modal;

#[cfg(test)]
mod context_rule;

/// Fenêtre de message, en remplacement de `customAlert`.
#[component]
pub fn NoticeModal() -> impl IntoView {
    let app = use_app();
    let close = Callback::new(move |_| app.notice.set(None));

    view! {
        <Show when=move || app.notice.get().is_some()>
            {move || {
                let notice = app.notice.get().expect("présence vérifiée par Show");
                view! {
                    <Modal title=notice.title max_width="400px" z_index=110 on_close=close>
                        <div class="modal-body">
                            // `white-space: pre-line` conserve les retours à la
                            // ligne des rapports d'import, qui listent les
                            // montants ajustés un par ligne.
                            <p style="font-size: 0.95rem; color: var(--text-secondary); line-height: 1.5; white-space: pre-line">
                                {notice.body}
                            </p>
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="btn btn-primary" on:click=move |_| close.run(())>
                                "OK"
                            </button>
                        </div>
                    </Modal>
                }
            }}
        </Show>
    }
}
