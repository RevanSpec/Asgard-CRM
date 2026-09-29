//! Réglages de l'entreprise.
//!
//! Toujours dans `localStorage`, sous la même clé et au même format : les
//! réglages existants d'un utilisateur sont relus tels quels par cette version.
//!
//! Le mot de passe SMTP n'y figure pas — il vit dans le trousseau du système
//! depuis la phase 1, et `strip_secret` continue de l'écarter si une ancienne
//! entrée traîne encore.

use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "asgard_crm_settings";

/// Clé du mot de passe tel qu'il était stocké avant la phase 1.
const LEGACY_PASSWORD_KEY: &str = "smtpPass";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub company_name: String,
    pub contact_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub siret: String,
    pub iban: String,

    /// Délai de règlement annoncé sur les factures, en jours. L'hôte en déduit
    /// l'échéance au moment de l'émission, puis ne la change plus.
    pub payment_terms_days: u32,

    pub urssaf_service_bnc: f64,
    pub urssaf_service_bic: f64,
    pub urssaf_vente: f64,
    pub acre_enabled: bool,

    pub smtp_host: String,
    pub smtp_port: String,
    pub smtp_user: String,
    pub smtp_secure: String,

    pub custom_color: String,
    pub logo_base64: String,

    pub email_template_invoice: String,
    pub email_template_estimate: String,
    pub email_template_reminder: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            company_name: "Asgard Solutions".into(),
            contact_name: "Thor Odinson".into(),
            email: "thor@asgard-solutions.fr".into(),
            phone: "06 12 34 56 78".into(),
            address: "1 Rue du Valhalla, 75008 Paris".into(),
            siret: "839 204 123 00019".into(),
            iban: "FR76 3000 2000 0001 2345 6789 012".into(),

            payment_terms_days: 30,

            urssaf_service_bnc: 21.1,
            urssaf_service_bic: 21.1,
            urssaf_vente: 12.3,
            acre_enabled: false,

            smtp_host: "127.0.0.1".into(),
            smtp_port: "1025".into(),
            smtp_user: String::new(),
            smtp_secure: "none".into(),

            custom_color: "#E5A93C".into(),
            logo_base64: String::new(),

            email_template_invoice: "Bonjour {clientName},\n\nVeuillez trouver ci-joint la facture {documentNumber} pour la prestation : {description}.\n\nLe montant total est de {amountTotal} €.\n\nCordialement,\n\n{senderName}\n{senderCompany}".into(),
            email_template_estimate: "Bonjour {clientName},\n\nVeuillez trouver ci-joint le devis {documentNumber} pour la prestation : {description}.\n\nLe montant total est de {amountTotal} €.\n\nCordialement,\n\n{senderName}\n{senderCompany}".into(),
            email_template_reminder: "Bonjour {clientName},\n\nSauf erreur ou omission de notre part, nous n'avons pas reçu le règlement de la facture {documentNumber} d'un montant de {amountTotal} € envoyée le {documentDate}.\n\nNous vous prions de bien vouloir régulariser cette situation dans les plus brefs délais. Vous trouverez la facture en pièce jointe.\n\nCordialement,\n\n{senderName}\n{senderCompany}".into(),
        }
    }
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Lit les réglages, en complétant les champs absents par leur valeur par défaut.
///
/// `#[serde(default)]` sur la structure fait ce travail : une sauvegarde d'une
/// version antérieure, à laquelle il manque des champs, se relit sans erreur.
pub fn load() -> Settings {
    let Some(storage) = storage() else {
        return Settings::default();
    };
    let Ok(Some(raw)) = storage.get_item(STORAGE_KEY) else {
        return Settings::default();
    };

    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(settings: &Settings) {
    let Some(storage) = storage() else { return };
    let Ok(json) = serde_json::to_string(settings) else { return };
    let _ = storage.set_item(STORAGE_KEY, &json);
}

/// Retire un mot de passe SMTP qui traînerait encore en clair.
///
/// La phase 1 l'a déplacé vers le trousseau, mais une sauvegarde importée peut
/// le réintroduire. Le geste est conservé : c'est ce qui empêche le défaut D2
/// de revenir par la porte de derrière.
pub fn strip_legacy_password() {
    let Some(storage) = storage() else { return };
    let Ok(Some(raw)) = storage.get_item(STORAGE_KEY) else { return };

    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&raw) else { return };
    let Some(object) = value.as_object_mut() else { return };

    if object.remove(LEGACY_PASSWORD_KEY).is_some() {
        if let Ok(json) = serde_json::to_string(&value) {
            let _ = storage.set_item(STORAGE_KEY, &json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_javascript_version() {
        let settings = Settings::default();

        assert_eq!(settings.urssaf_service_bnc, 21.1);
        assert_eq!(settings.urssaf_vente, 12.3);
        assert!(!settings.acre_enabled);
        assert_eq!(settings.smtp_host, "127.0.0.1");
        assert_eq!(settings.smtp_port, "1025");
        assert_eq!(settings.custom_color, "#E5A93C");
    }

    /// Une sauvegarde partielle doit se relire : `#[serde(default)]` complète
    /// les champs manquants au lieu d'échouer.
    #[test]
    fn a_partial_record_reads_back_with_defaults() {
        let settings: Settings =
            serde_json::from_str(r#"{"companyName": "Forge du Nord"}"#).unwrap();

        assert_eq!(settings.company_name, "Forge du Nord");
        assert_eq!(settings.urssaf_service_bnc, 21.1);
    }

    /// Le format sur disque est celui de la version JavaScript : camelCase.
    /// Le changer rendrait illisibles les réglages existants.
    #[test]
    fn the_stored_shape_stays_camel_case() {
        let json = serde_json::to_string(&Settings::default()).unwrap();

        for key in ["companyName", "urssafServiceBnc", "acreEnabled", "smtpHost", "customColor"] {
            assert!(json.contains(&format!("\"{key}\"")), "champ manquant : {key}");
        }
    }

    /// Le mot de passe n'a pas de champ : il ne peut plus être écrit, même par
    /// erreur.
    #[test]
    fn there_is_no_field_for_the_smtp_password() {
        let json = serde_json::to_string(&Settings::default()).unwrap();
        assert!(!json.contains("smtpPass"), "{json}");
    }
}
