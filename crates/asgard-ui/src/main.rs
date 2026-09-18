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

mod format;
mod ipc;
mod settings;
mod state;
mod views;

use leptos::prelude::*;
use state::{App, Tab};

fn main() {
    // Sans ce crochet, une panique WebAssembly ne laisse qu'un « unreachable »
    // dans la console, sans pile d'appel ni message.
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(Root);
}

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
                    fallback=|| view! { <p class="loading-note">"Chargement…"</p> }
                >
                    {move || match app.tab.get() {
                        Tab::Dashboard => views::dashboard().into_any(),
                        Tab::Clients => views::clients().into_any(),
                        other => views::pending(other).into_any(),
                    }}
                </Show>
            </main>
            <views::NoticeModal />
        </div>
    }
}
