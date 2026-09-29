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

    /// Médiateur de la consommation, obligatoire dès qu'on facture des
    /// particuliers (art. L616-1 du code de la consommation).
    pub mediator: String,
    /// Assurance professionnelle, obligatoire pour les activités qui y sont
    /// soumises (art. L112-11 du code des assurances).
    pub insurance: String,
    /// Option pour le paiement de la TVA d'après les débits, à mentionner sur
    /// les factures lorsqu'elle a été exercée.
    pub vat_on_debits: bool,

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
    /// **Aucune identité par défaut.**
    ///
    /// Ces champs décrivaient jusqu'ici une société fictive — « Asgard
    /// Solutions », un SIRET et un IBAN d'apparence crédible. Rien à l'écran ne
    /// signalait qu'ils n'étaient pas ceux de l'utilisateur : une facture émise
    /// avant d'avoir ouvert les réglages partait au nom de quelqu'un d'autre,
    /// vers un compte inexistant (défaut D11). L'écran d'accueil les demande
    /// désormais au premier lancement, et l'hôte refuse d'éditer sans eux.
    ///
    /// Ce qui garde une valeur par défaut est ce qui n'engage personne : délai
    /// de règlement, taux de cotisation, couleur, gabarits de message.
    fn default() -> Self {
        Self {
            company_name: String::new(),
            contact_name: String::new(),
            email: String::new(),
            phone: String::new(),
            address: String::new(),
            siret: String::new(),
            iban: String::new(),

            payment_terms_days: 30,

            mediator: String::new(),
            insurance: String::new(),
            vat_on_debits: false,

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

/// Enregistre les réglages : le cache tout de suite, la base ensuite.
///
/// Le `localStorage` est écrit sur place parce que l'interface lit les réglages
/// pendant le rendu, sans pouvoir attendre l'hôte. La base est écrite en
/// arrière-plan : c'est elle que la copie quotidienne emporte (défaut D13).
pub fn save(settings: &Settings) {
    let Ok(json) = serde_json::to_string(settings) else { return };
    cache(&json);
    queue_host_write(json);
}

/// Adopte les réglages que la base vient de rendre.
///
/// Au démarrage, la base fait foi : elle a été sauvegardée, le cache non. Un
/// JSON illisible est ignoré plutôt qu'écrit — il vaut mieux repartir des
/// valeurs par défaut que d'installer un cache que personne ne saura relire.
pub fn adopt(json: &str) {
    if serde_json::from_str::<Settings>(json).is_ok() {
        cache(json);
    }
}

/// Envoie à l'hôte ce que le cache contient, si tant est qu'il contienne
/// quelque chose.
///
/// C'est la reprise des installations existantes : leurs réglages n'ont jamais
/// vu la base. Même geste que la migration du mot de passe SMTP vers le
/// trousseau en phase 1 — lire l'ancien emplacement, écrire le nouveau.
pub fn adopt_cache_into_database() {
    let Some(storage) = storage() else { return };
    let Ok(Some(json)) = storage.get_item(STORAGE_KEY) else { return };

    queue_host_write(json);
}

fn cache(json: &str) {
    let Some(storage) = storage() else { return };
    let _ = storage.set_item(STORAGE_KEY, json);
}

/// Écriture vers l'hôte : au plus une en cours, la plus récente en attente.
///
/// L'écran Paramètres enregistre à **chaque frappe**. Lancer autant d'écritures
/// concurrentes ne garantirait pas leur ordre d'arrivée en base, et la valeur
/// affichée pourrait ne pas être celle enregistrée. Une frappe qui survient
/// pendant une écriture remplace donc celle qui attendait : la dernière valeur
/// gagne, et une rafale ne coûte que deux écritures.
struct Queue {
    writing: bool,
    pending: Option<String>,
}

thread_local! {
    static QUEUE: std::cell::RefCell<Queue> =
        const { std::cell::RefCell::new(Queue { writing: false, pending: None }) };
}

fn queue_host_write(json: String) {
    let begin = QUEUE.with_borrow_mut(|queue| {
        queue.pending = Some(json);
        let idle = !queue.writing;
        queue.writing = true;
        idle
    });

    if !begin {
        return;
    }

    leptos::task::spawn_local(async move {
        while let Some(json) = QUEUE.with_borrow_mut(|queue| queue.pending.take()) {
            #[derive(Serialize)]
            struct Args {
                json: String,
            }

            if let Err(error) = crate::ipc::invoke::<_, ()>("save_stored_settings", &Args { json }).await
            {
                // Le parcours de l'application échoue sur toute erreur de
                // console : une écriture perdue se voit donc en intégration
                // continue, au lieu de passer inaperçue jusqu'à la sauvegarde
                // suivante.
                web_sys::console::error_1(
                    &format!("Réglages non enregistrés en base : {error}").into(),
                );
                break;
            }
        }

        QUEUE.with_borrow_mut(|queue| queue.writing = false);
    });
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

    /// Défaut D11. Ce test tomberait si une identité revenait un jour dans les
    /// valeurs par défaut — et c'est bien ce qu'on veut : une facture ne doit
    /// jamais pouvoir sortir au nom d'une société que l'utilisateur n'a pas
    /// saisie.
    #[test]
    fn the_default_identity_is_empty() {
        let settings = Settings::default();

        for (field, value) in [
            ("companyName", &settings.company_name),
            ("contactName", &settings.contact_name),
            ("email", &settings.email),
            ("phone", &settings.phone),
            ("address", &settings.address),
            ("siret", &settings.siret),
            ("iban", &settings.iban),
        ] {
            assert!(value.is_empty(), "{field} ne doit pas avoir de valeur par défaut : « {value} »");
        }

        // Ce qui n'engage personne garde la sienne.
        assert_eq!(settings.payment_terms_days, 30);
        assert_eq!(settings.custom_color, "#E5A93C");
        assert!(settings.email_template_invoice.contains("Bonjour"));
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
