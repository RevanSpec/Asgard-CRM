//! Envoi SMTP — remplace `nodemailer`, qui tournait dans le processus principal
//! Electron (`main.js`).
//!
//! La logique TLS reproduit exactement celle de l'implémentation JavaScript,
//! y compris son exception pour Proton Mail Bridge. Toute divergence ici casse
//! silencieusement l'envoi chez les utilisateurs de Bridge, qui est le cas
//! d'usage documenté dans le README.

use base64::Engine;
use lettre::message::{header::ContentType, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtpConfig {
    pub host: String,
    /// Le formulaire de réglages fournit une chaîne : `main.js` faisait
    /// `parseInt(port)`. On accepte les deux formes.
    pub port: PortValue,
    pub user: String,
    /// Absent lorsque le mot de passe vient du trousseau plutôt que du formulaire.
    #[serde(default)]
    pub pass: Option<String>,
    /// `"ssl"` pour un TLS implicite, toute autre valeur pour STARTTLS.
    pub secure: String,
    #[serde(default)]
    pub from: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PortValue {
    Number(u16),
    Text(String),
}

impl PortValue {
    fn resolve(&self) -> Result<u16, MailError> {
        match self {
            PortValue::Number(port) => Ok(*port),
            PortValue::Text(text) => text
                .trim()
                .parse()
                .map_err(|_| MailError::InvalidPort(text.clone())),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailData {
    pub to: String,
    pub subject: String,
    pub text: String,
    #[serde(default)]
    pub filename: Option<String>,
    /// PDF encodé en base64, produit par jsPDF côté interface.
    /// Disparaîtra en phase 4, quand le PDF sera généré côté Rust.
    #[serde(default)]
    pub pdf_base64: Option<String>,
}

/// Résultat renvoyé à l'interface. La forme est identique à celle que
/// renvoyaient les handlers IPC d'Electron, pour ne pas toucher au JSX.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendOutcome {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl SendOutcome {
    pub fn ok(message_id: Option<String>) -> Self {
        Self { success: true, message_id, error: None }
    }

    pub fn failed(error: impl std::fmt::Display) -> Self {
        Self { success: false, message_id: None, error: Some(error.to_string()) }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("Port SMTP invalide : {0}")]
    InvalidPort(String),

    #[error("Adresse e-mail invalide ({field}) : {source}")]
    Address {
        field: &'static str,
        #[source]
        source: lettre::address::AddressError,
    },

    #[error("Pièce jointe illisible : {0}")]
    Attachment(#[from] base64::DecodeError),

    #[error("Message mal formé : {0}")]
    Build(#[from] lettre::error::Error),

    #[error("{0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

/// Proton Mail Bridge présente un certificat auto-signé sur la boucle locale.
///
/// `main.js` désactivait la vérification avec :
/// `rejectUnauthorized: host !== '127.0.0.1' && host !== 'localhost'`.
/// La règle est reprise telle quelle : l'assouplissement ne vaut que pour ces
/// deux hôtes, jamais pour un serveur distant.
fn is_loopback(host: &str) -> bool {
    host == "127.0.0.1" || host == "localhost"
}

fn build_transport(config: &SmtpConfig, password: &str) -> Result<SmtpTransport, MailError> {
    let port = config.port.resolve()?;
    let local = is_loopback(&config.host);

    let tls_parameters = TlsParameters::builder(config.host.clone())
        .dangerous_accept_invalid_certs(local)
        .dangerous_accept_invalid_hostnames(local)
        .build()?;

    // `secure: 'ssl'` correspondait à `secure: true` chez nodemailer, c'est-à-dire
    // un TLS implicite (port 465). Toute autre valeur laissait nodemailer tenter
    // STARTTLS si le serveur l'annonce — d'où `Opportunistic` et non `Required`,
    // qui échouerait sur un serveur ne l'annonçant pas.
    let tls = if config.secure == "ssl" {
        Tls::Wrapper(tls_parameters)
    } else {
        Tls::Opportunistic(tls_parameters)
    };

    let mut builder = SmtpTransport::builder_dangerous(config.host.as_str())
        .port(port)
        .tls(tls);

    // Un Bridge local sans identifiants reste possible : on n'authentifie que
    // si un utilisateur est renseigné, comme le faisait nodemailer.
    if !config.user.is_empty() {
        builder = builder.credentials(Credentials::new(
            config.user.clone(),
            password.to_string(),
        ));
    }

    Ok(builder.build())
}

fn build_message(config: &SmtpConfig, data: &EmailData) -> Result<Message, MailError> {
    let from = config.from.clone().unwrap_or_else(|| config.user.clone());

    let builder = Message::builder()
        .from(from.parse().map_err(|source| MailError::Address { field: "expéditeur", source })?)
        .to(data.to.parse().map_err(|source| MailError::Address { field: "destinataire", source })?)
        .subject(&data.subject);

    let body = SinglePart::plain(data.text.clone());

    match (&data.pdf_base64, &data.filename) {
        (Some(encoded), Some(filename)) if !encoded.is_empty() => {
            let bytes = base64::engine::general_purpose::STANDARD.decode(encoded)?;
            let attachment = Attachment::new(filename.clone())
                .body(bytes, ContentType::parse("application/pdf").expect("type MIME constant"));

            Ok(builder.multipart(MultiPart::mixed().singlepart(body).singlepart(attachment))?)
        }
        _ => Ok(builder.singlepart(body)?),
    }
}

pub fn send(config: &SmtpConfig, data: &EmailData, password: &str) -> Result<String, MailError> {
    let transport = build_transport(config, password)?;
    let message = build_message(config, data)?;
    let response = transport.send(&message)?;

    // lettre n'expose pas le Message-ID du serveur séparément : on renvoie la
    // première ligne de sa réponse, qui le contient chez la plupart des MTA.
    // La ligne est copiée avant le retour : l'itérateur emprunte `response`,
    // qui est une variable locale.
    let message_id = response.message().next().unwrap_or_default().to_string();

    Ok(message_id)
}

pub fn verify(config: &SmtpConfig, password: &str) -> Result<(), MailError> {
    build_transport(config, password)?.test_connection()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts_match_the_javascript_rule() {
        assert!(is_loopback("127.0.0.1"));
        assert!(is_loopback("localhost"));

        // La règle JavaScript comparait par égalité stricte : ces hôtes-là
        // n'étaient pas exemptés et ne doivent pas l'être davantage ici.
        assert!(!is_loopback("::1"));
        assert!(!is_loopback("LOCALHOST"));
        assert!(!is_loopback("localhost.evil.com"));
        assert!(!is_loopback("smtp.protonmail.ch"));
    }

    #[test]
    fn port_accepts_both_string_and_number() {
        assert_eq!(PortValue::Text("1025".into()).resolve().unwrap(), 1025);
        assert_eq!(PortValue::Text(" 465 ".into()).resolve().unwrap(), 465);
        assert_eq!(PortValue::Number(587).resolve().unwrap(), 587);
        assert!(PortValue::Text("".into()).resolve().is_err());
        assert!(PortValue::Text("abc".into()).resolve().is_err());
    }

    fn config(host: &str, secure: &str) -> SmtpConfig {
        SmtpConfig {
            host: host.into(),
            port: PortValue::Number(1025),
            user: "thor@asgard.fr".into(),
            pass: None,
            secure: secure.into(),
            from: None,
        }
    }

    #[test]
    fn transports_build_for_every_security_mode() {
        for (host, secure) in [
            ("127.0.0.1", "none"),
            ("localhost", "none"),
            ("smtp.example.com", "ssl"),
            ("smtp.example.com", "none"),
        ] {
            assert!(
                build_transport(&config(host, secure), "secret").is_ok(),
                "transport {host} / {secure}"
            );
        }
    }

    #[test]
    fn message_carries_the_pdf_attachment() {
        let data = EmailData {
            to: "pepper@stark.com".into(),
            subject: "Facture FAC-STARKINDUS-2026-0001".into(),
            text: "Bonjour,".into(),
            filename: Some("FAC-STARKINDUS-2026-0001.pdf".into()),
            pdf_base64: Some(base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.3 fake")),
        };

        let raw = String::from_utf8(
            build_message(&config("localhost", "none"), &data)
                .unwrap()
                .formatted(),
        )
        .unwrap();

        assert!(raw.contains("multipart/mixed"));
        assert!(raw.contains("application/pdf"));
        assert!(raw.contains("FAC-STARKINDUS-2026-0001.pdf"));
    }

    #[test]
    fn message_without_attachment_stays_single_part() {
        let data = EmailData {
            to: "pepper@stark.com".into(),
            subject: "Relance".into(),
            text: "Bonjour,".into(),
            filename: None,
            pdf_base64: None,
        };

        let raw = String::from_utf8(
            build_message(&config("localhost", "none"), &data).unwrap().formatted(),
        )
        .unwrap();

        assert!(!raw.contains("multipart/mixed"));
        assert!(raw.contains("Relance"));
    }

    #[test]
    fn sender_falls_back_to_the_smtp_user() {
        let mut cfg = config("localhost", "none");
        cfg.from = None;

        let data = EmailData {
            to: "pepper@stark.com".into(),
            subject: "Test".into(),
            text: "corps".into(),
            filename: None,
            pdf_base64: None,
        };

        let raw = String::from_utf8(build_message(&cfg, &data).unwrap().formatted()).unwrap();
        assert!(raw.contains("From: thor@asgard.fr"));
    }

    #[test]
    fn invalid_addresses_are_reported_by_field() {
        let data = EmailData {
            to: "pas-une-adresse".into(),
            subject: "Test".into(),
            text: "corps".into(),
            filename: None,
            pdf_base64: None,
        };

        let error = build_message(&config("localhost", "none"), &data).unwrap_err();
        assert!(error.to_string().contains("destinataire"), "{error}");
    }
}
