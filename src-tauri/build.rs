fn main() {
    guard_against_cargo_only_release_build();
    tauri_build::build()
}

/// Refuse une compilation release qui ne passe pas par la CLI Tauri.
///
/// La CLI fait deux choses que `cargo build` ignore :
///
/// 1. elle exécute `beforeBuildCommand` (`npm run build`), qui régénère `dist/` ;
/// 2. elle indique à `tauri::generate_context!` d'embarquer ces fichiers plutôt
///    que de pointer sur `devUrl`.
///
/// Sans elle, `cargo build --release` produit un binaire qui se lance
/// normalement mais n'affiche qu'un « localhost a refusé de se connecter » : la
/// webview cherche le serveur de développement. Aucune trace dans les journaux,
/// le symptôme ressemble à un problème de réseau, et la seule différence
/// visible est la taille du binaire — 3,88 Mo sans les assets contre 4,22 Mo
/// avec.
///
/// Mieux vaut refuser de compiler que livrer cet exécutable.
///
/// `TAURI_CLI_VERBOSITY` est renseignée par la CLI et par elle seule : c'est ce
/// qui distingue les deux chemins de compilation. En profil `debug` le garde-fou
/// ne s'applique pas, pour laisser `cargo test` et `cargo check` fonctionner.
fn guard_against_cargo_only_release_build() {
    println!("cargo:rerun-if-env-changed=TAURI_CLI_VERBOSITY");

    let is_release = std::env::var("PROFILE").as_deref() == Ok("release");
    let driven_by_tauri_cli = std::env::var_os("TAURI_CLI_VERBOSITY").is_some();

    if is_release && !driven_by_tauri_cli {
        panic!(
            "\n\n\
             Compilation release lancée sans la CLI Tauri.\n\n\
             Le binaire produit chercherait le serveur de développement au lieu\n\
             des fichiers embarqués, et n'afficherait qu'une page d'erreur\n\
             « localhost a refusé de se connecter ».\n\n\
             Utilisez :  npm run dist        (ou  cargo tauri build)\n\
             Plutôt que : cargo build --release\n\n"
        );
    }
}
