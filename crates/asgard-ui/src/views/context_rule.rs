//! Une règle que le compilateur ne sait pas vérifier : **`use_app()` ne
//! s'appelle que pendant le rendu**.
//!
//! Le contexte Leptos n'existe que le temps de construire la vue. Dans un
//! gestionnaire d'événement, l'appel panique — et une panique dans le
//! WebAssembly fige l'interface entière jusqu'au redémarrage de l'application.
//!
//! Le défaut s'est produit : en partageant le bouton « Nouvelle Facture » entre
//! les factures et le tableau de bord, l'aide qui l'ouvre a été sortie du corps
//! du composant tout en continuant d'appeler `use_app()`. Cliquer dessus figeait
//! l'application. Le même défaut existait sur l'envoi par e-mail. Rien ne le
//! signalait : ni le compilateur, ni les tests, ni la revue.
//!
//! Ce test relit les sources et refuse qu'une fonction qui ne rend pas de vue
//! appelle `use_app()`. Les fonctions concernées reçoivent `App` en argument.

use std::path::Path;

/// Fonctions appelées depuis le corps d'un composant, donc pendant le rendu,
/// et qui ne renvoient pas elles-mêmes une vue.
const CALLED_WHILE_RENDERING: [&str; 1] = ["client_options"];

/// Une signature qui produit de la vue : le contexte y est disponible.
fn renders(signature: &str) -> bool {
    signature.contains("impl IntoView") || signature.contains("AnyView")
}

fn name_of(signature: &str) -> String {
    signature
        .split("fn ")
        .nth(1)
        .unwrap_or_default()
        .split(['(', '<'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[test]
fn use_app_is_never_called_outside_rendering() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files: Vec<_> = std::fs::read_dir(root.join("views"))
        .expect("le dossier des vues doit être lisible")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        // Ce fichier-ci cite la règle qu'il vérifie : il ne s'analyse pas.
        .filter(|path| path.file_name().is_some_and(|name| name != "context_rule.rs"))
        .collect();
    files.push(root.join("actions.rs"));
    files.push(root.join("backup.rs"));

    let mut offenders = Vec::new();

    for path in files {
        let source = std::fs::read_to_string(&path).expect("source lisible");
        let mut component = false;
        let mut signature = String::new();
        let mut current: Option<(String, bool)> = None;

        for line in source.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("#[component]") {
                component = true;
                continue;
            }

            // Une signature peut tenir sur plusieurs lignes ; on l'accumule
            // jusqu'à l'accolade ouvrante.
            if signature.is_empty() && (trimmed.starts_with("fn ") || trimmed.contains(" fn ")) {
                signature.push_str(trimmed);
            } else if !signature.is_empty() {
                signature.push(' ');
                signature.push_str(trimmed);
            }

            if !signature.is_empty() && trimmed.ends_with('{') {
                current = Some((name_of(&signature), component || renders(&signature)));
                signature.clear();
                component = false;
                continue;
            }

            if trimmed.contains("use_app()") {
                if let Some((name, allowed)) = &current {
                    if !allowed && !CALLED_WHILE_RENDERING.contains(&name.as_str()) {
                        offenders.push(format!("{}::{name}", path.file_name().unwrap().to_string_lossy()));
                    }
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "ces fonctions appellent use_app() sans rendre de vue ; si elles sont \
         déclenchées par un événement, l'appel paniquera et figera l'interface. \
         Passez-leur `App` en argument : {offenders:?}"
    );
}

/// Le test ci-dessus ne vaut que s'il sait reconnaître une faute. Ces cas
/// vérifient sa lecture des signatures.
#[test]
fn the_rule_reads_signatures_correctly() {
    assert!(renders("fn Row(client: Client) -> impl IntoView {"));
    assert!(renders("pub fn dashboard() -> impl IntoView {"));
    assert!(!renders("pub(super) fn open_invoice_form(app: App, creating: RwSignal<bool>) {"));
    assert_eq!(name_of("pub(super) fn open_invoice_form(app: App) {"), "open_invoice_form");
    assert_eq!(name_of("fn Row(client: Client) -> impl IntoView {"), "Row");
    assert_eq!(name_of("pub async fn save_client(app: App) -> bool {"), "save_client");
}
