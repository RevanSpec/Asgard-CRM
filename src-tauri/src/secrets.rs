//! Trousseau de l'OS — corrige le défaut D2.
//!
//! Le mot de passe SMTP était écrit en clair dans `localStorage`
//! (`App.jsx:335`) et réexporté tel quel dans le fichier de sauvegarde JSON,
//! destiné à être copié et partagé. Il vit désormais dans le trousseau du
//! système : Credential Manager sous Windows, Keychain sous macOS, Secret
//! Service sous Linux.
//!
//! Le secret ne transite plus par l'interface : les commandes d'envoi le lisent
//! directement ici.

use keyring::Entry;

const SERVICE: &str = "com.asgard.crm";

/// Un seul jeu d'identifiants SMTP par installation, comme dans les réglages.
const ACCOUNT: &str = "smtp";

#[derive(Debug, thiserror::Error)]
#[error("Trousseau de l'OS inaccessible : {0}")]
pub struct SecretError(#[from] keyring::Error);

fn entry() -> Result<Entry, SecretError> {
    Ok(Entry::new(SERVICE, ACCOUNT)?)
}

pub fn store(password: &str) -> Result<(), SecretError> {
    if password.is_empty() {
        return clear();
    }
    entry()?.set_password(password)?;
    Ok(())
}

/// Renvoie `None` plutôt qu'une erreur quand aucun secret n'est enregistré :
/// c'est un état normal (première utilisation, ou envoi sans authentification).
pub fn read() -> Result<Option<String>, SecretError> {
    match entry()?.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(other) => Err(SecretError(other)),
    }
}

pub fn clear() -> Result<(), SecretError> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(other) => Err(SecretError(other)),
    }
}

pub fn exists() -> Result<bool, SecretError> {
    Ok(read()?.is_some())
}
