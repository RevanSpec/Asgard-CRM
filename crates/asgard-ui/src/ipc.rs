//! Appels à l'hôte.
//!
//! Remplace `src/ipc.js` et `src/db.js`. La différence tient en un mot :
//! **types**. Le JavaScript passait des objets anonymes et recevait ce que
//! l'hôte voulait bien renvoyer ; ici les deux côtés compilent le même crate
//! `asgard-ipc`, et un champ renommé d'un seul côté ne compile plus.
//!
//! C'est le seul bénéfice réel de cette phase — le reste n'est qu'une
//! réécriture à rendu constant.

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI_INTERNALS__"], js_name = invoke, catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// L'application tourne-t-elle dans sa coquille de bureau ?
///
/// Faux quand la page est servie seule, par `trunk serve`. Les appels
/// renvoient alors une erreur explicite plutôt que de lever.
pub fn is_desktop() -> bool {
    js_sys::Reflect::has(
        &web_sys::window().expect("pas de fenêtre").into(),
        &JsValue::from_str("__TAURI_INTERNALS__"),
    )
    .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcError(pub String);

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Invoque une commande de l'hôte.
///
/// Les arguments et la réponse passent par `serde`, donc par les types de
/// `asgard-ipc` : une erreur de forme se voit à la compilation, plus à
/// l'exécution sur un `undefined` inattendu.
pub async fn invoke<A, R>(command: &str, args: &A) -> Result<R, IpcError>
where
    A: Serialize + ?Sized,
    R: DeserializeOwned,
{
    if !is_desktop() {
        return Err(IpcError(format!(
            "« {command} » n'est disponible que dans l'application de bureau."
        )));
    }

    let args = serde_wasm_bindgen::to_value(args)
        .map_err(|error| IpcError(format!("argument illisible : {error}")))?;

    let value = tauri_invoke(command, args)
        .await
        .map_err(|error| IpcError(describe(error)))?;

    serde_wasm_bindgen::from_value(value)
        .map_err(|error| IpcError(format!("réponse illisible : {error}")))
}

/// Invoque une commande sans argument.
pub async fn call<R: DeserializeOwned>(command: &str) -> Result<R, IpcError> {
    invoke(command, &()).await
}

/// Met en mots une erreur venue de l'hôte.
///
/// Les commandes renvoient une chaîne : c'est ce que produit `DbError` via sa
/// sérialisation. Une valeur d'une autre forme est affichée telle quelle plutôt
/// que remplacée par un « erreur inconnue » qui n'aiderait personne.
fn describe(error: JsValue) -> String {
    error
        .as_string()
        .or_else(|| {
            js_sys::JSON::stringify(&error)
                .ok()
                .and_then(|text| text.as_string())
        })
        .unwrap_or_else(|| "erreur inattendue de l'hôte".into())
}
