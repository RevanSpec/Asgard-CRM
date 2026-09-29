//! Interface d'Asgard CRM, compilée en WebAssembly.
//!
//! Phase 5 du plan de migration — la seule que le plan classait **optionnelle**,
//! et dont il estimait le retour faible : l'utilisateur ne verra aucune
//! différence. Le gain est ailleurs, et tient en un point : l'interface et
//! l'hôte compilent désormais **le même crate de types** (`asgard-ipc`). Un
//! champ renommé d'un côté ne compile plus, là où le JSX et le Rust pouvaient
//! diverger sans que rien ne le signale jusqu'à l'exécution.
//!
//! Le CSS est repris **tel quel** : le thème n'a pas à être refait, et le
//! reprendre à l'identique est la meilleure garantie que le rendu ne dérive pas.

mod actions;
mod backup;
mod format;
mod ipc;
mod settings;
mod state;
mod templates;
mod validation;
mod views;

use leptos::prelude::*;
use state::{App, Tab};

fn main() {
    // Sans ce crochet, une panique WebAssembly ne laisse qu'un « unreachable »
    // dans la console, sans pile d'appel ni message.
    console_error_panic_hook::set_once();
    announce_panics();
    leptos::mount::mount_to_body(Root);
}

/// Affiche un bandeau quand l'interface panique.
///
/// Une panique WebAssembly arrête l'instance : plus aucun bouton ne répond,
/// mais l'écran reste affiché, intact. Rien ne distingue alors une application
/// morte d'une application lente — c'est ce qui s'est produit avec le bouton
/// « Nouvelle Facture », et l'utilisateur n'avait aucun moyen de le savoir.
///
/// Le bandeau est posé en DOM brut : après une panique, Leptos ne peut plus
/// rien redessiner.
fn announce_panics() {
    let previous = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        previous(info);

        let Some(document) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        // Une seconde panique ne doit pas empiler les bandeaux.
        if document.get_element_by_id(PANIC_BANNER_ID).is_some() {
            return;
        }

        if let Ok(banner) = document.create_element("div") {
            banner.set_id(PANIC_BANNER_ID);
            let _ = banner.set_attribute(
                "style",
                "position: fixed; inset: 0 0 auto 0; z-index: 9999; padding: 0.9rem 1.25rem; \
                 background: #EF4444; color: white; font-weight: 600; text-align: center; \
                 font-family: system-ui, sans-serif",
            );
            banner.set_text_content(Some(
                "Asgard CRM a rencontré une erreur interne et ne répond plus. \
                 Fermez puis rouvrez l'application — vos données sont intactes.",
            ));

            if let Some(body) = document.body() {
                let _ = body.append_child(&banner);
            }
        }
    }));
}

/// Identifiant du bandeau, pour ne l'afficher qu'une fois.
const PANIC_BANNER_ID: &str = "asgard-panic-banner";

#[component]
fn Root() -> impl IntoView {
    let app = App::new();
    provide_context(app);

    // Une entrée héritée aurait pu survivre à la migration du mot de passe vers
    // le trousseau : on la retire au démarrage, comme le faisait la version
    // React (défaut D2).
    settings::strip_legacy_password();

    leptos::task::spawn_local(async move {
        app.reload().await;
    });

    view! {
        <div class="app-container">
            <views::Sidebar />
            <main class="main-content">
                <Show
                    when=move || !app.loading.get()
                    fallback=|| view! { <p class="metric-subtext">"Chargement…"</p> }
                >
                    {move || match app.tab.get() {
                        Tab::Dashboard => views::dashboard().into_any(),
                        Tab::Clients => views::clients().into_any(),
                        Tab::Invoices => views::invoices().into_any(),
                        Tab::Estimates => views::estimates().into_any(),
                        Tab::Expenses => views::expenses().into_any(),
                        Tab::Compta => views::compta().into_any(),
                        Tab::Settings => views::settings().into_any(),
                    }}
                </Show>
            </main>
            <views::NoticeModal />
            <views::ConfirmModal />
            <views::EmailModal />
            <views::PaymentModal />
        </div>
    }
}
