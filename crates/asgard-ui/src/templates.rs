//! Gabarits d'e-mail — port de `src/domain/templates.js`.
//!
//! Le message est composé ici, dans l'interface, puis **relu et modifié par
//! l'utilisateur** avant l'envoi. C'est pourquoi il n'est pas composé par
//! l'hôte : un aller-retour pour afficher un brouillon qu'on va retoucher
//! n'apporterait rien. Le PDF, lui, n'est jamais relu avant envoi — d'où son
//! traitement différent, côté Rust dans l'hôte.
//!
//! La substitution est volontairement naïve — un remplacement de jetons
//! `{nom}` — comme dans l'original.

use asgard_ipc::Client;

use crate::settings::Settings;

/// Nature du message. Les noms sérialisés sont ceux qu'attend `send_document`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Invoice,
    Estimate,
    /// Une relance porte sur une facture : même pièce, autre message.
    Reminder,
    /// Avoir. Seul l'export PDF l'emprunte : un avoir ne s'envoie pas encore
    /// par e-mail.
    Credit,
}

/// Valeurs substituées dans un gabarit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateData {
    pub client_name: String,
    pub document_number: String,
    pub description: String,
    pub amount_total: String,
    pub due_date: String,
    pub document_date: String,
    pub sender_name: String,
    pub sender_company: String,
}

/// Remplace chaque jeton par sa valeur. Un jeton inconnu reste en place ; un
/// jeton sans valeur est remplacé par une chaîne vide — comportement d'origine.
pub fn resolve(template: &str, data: &TemplateData) -> String {
    [
        ("{clientName}", &data.client_name),
        ("{documentNumber}", &data.document_number),
        ("{description}", &data.description),
        ("{amountTotal}", &data.amount_total),
        ("{dueDate}", &data.due_date),
        ("{documentDate}", &data.document_date),
        ("{senderName}", &data.sender_name),
        ("{senderCompany}", &data.sender_company),
    ]
    .into_iter()
    .fold(template.to_string(), |text, (token, value)| {
        text.replace(token, value)
    })
}

/// Objet du message, repris mot pour mot.
pub fn subject(kind: Kind, number: &str, company: &str) -> String {
    match kind {
        Kind::Invoice => format!("Facture {number} - {company}"),
        Kind::Reminder => format!("Rappel : Facture impayée {number} - {company}"),
        Kind::Estimate => format!("Devis {number} - {company}"),
        Kind::Credit => format!("Avoir {number} - {company}"),
    }
}

/// Gabarit à utiliser : celui des réglages, avec repli sur le gabarit par
/// défaut s'il a été vidé.
pub fn pick(kind: Kind, settings: &Settings) -> String {
    let defaults = Settings::default();
    let (chosen, fallback) = match kind {
        Kind::Invoice => (&settings.email_template_invoice, defaults.email_template_invoice),
        Kind::Reminder => (&settings.email_template_reminder, defaults.email_template_reminder),
        Kind::Estimate | Kind::Credit => {
            (&settings.email_template_estimate, defaults.email_template_estimate)
        }
    };

    if chosen.trim().is_empty() {
        fallback
    } else {
        chosen.clone()
    }
}

/// Brouillon d'e-mail, prêt à être relu dans la fenêtre d'envoi.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Draft {
    pub to: String,
    pub subject: String,
    pub text: String,
}

/// Pièce envoyée, telle que les gabarits la voient.
pub struct Document<'a> {
    pub number: &'a str,
    pub description: &'a str,
    pub total: f64,
    /// Date ISO de la pièce.
    pub date: &'a str,
    /// Échéance ISO, pour les factures qui en portent une.
    pub due_date: Option<&'a str>,
    /// Raison sociale recopiée sur la pièce à sa création.
    pub company: &'a str,
}

/// Valeurs des jetons — port de `buildTemplateData`.
///
/// **Écart assumé.** Pour un client supprimé depuis, l'original laissait
/// `{clientName}` vide (« Bonjour , … »). La pièce porte pourtant la raison
/// sociale du client : c'est elle qui est reprise.
pub fn template_data(doc: &Document, client: Option<&Client>, settings: &Settings) -> TemplateData {
    TemplateData {
        client_name: client.map_or(doc.company, |c| &c.company_name).to_string(),
        document_number: doc.number.to_string(),
        description: doc.description.to_string(),
        amount_total: format!("{:.2}", doc.total),
        // Le jeton reste vide pour un devis, et pour les factures émises avant
        // que l'échéance ne soit enregistrée.
        due_date: doc.due_date.map(crate::format::date).unwrap_or_default(),
        document_date: crate::format::date(doc.date),
        sender_name: settings.contact_name.clone(),
        sender_company: settings.company_name.clone(),
    }
}

/// Brouillon complet — port de `buildEmailDraft`. Sans client, le
/// destinataire reste à saisir.
pub fn compose(kind: Kind, doc: &Document, client: Option<&Client>, settings: &Settings) -> Draft {
    let data = template_data(doc, client, settings);
    Draft {
        to: client.map(|c| c.email.clone()).unwrap_or_default(),
        subject: subject(kind, &data.document_number, &settings.company_name),
        text: resolve(&pick(kind, settings), &data),
    }
}

/// Tests repris de `src/domain/templates.test.js`.
#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> TemplateData {
        TemplateData {
            client_name: "Stark Industries".into(),
            document_number: "FAC-STARKINDUS-2026-0001".into(),
            description: "Audit".into(),
            amount_total: "9720.00".into(),
            document_date: "05/02/2026".into(),
            sender_name: "Thor Odinson".into(),
            sender_company: "Asgard Solutions".into(),
            ..Default::default()
        }
    }

    #[test]
    fn replaces_every_token() {
        assert_eq!(
            resolve("Bonjour {clientName}, facture {documentNumber}.", &data()),
            "Bonjour Stark Industries, facture FAC-STARKINDUS-2026-0001."
        );
    }

    #[test]
    fn replaces_every_occurrence_of_a_token() {
        assert_eq!(resolve("{clientName} / {clientName}", &data()), "Stark Industries / Stark Industries");
    }

    #[test]
    fn a_token_without_value_becomes_empty() {
        assert_eq!(resolve("Échéance : {dueDate}.", &data()), "Échéance : .");
    }

    #[test]
    fn an_unknown_token_is_left_alone() {
        assert_eq!(resolve("Solde {montantRestant}.", &data()), "Solde {montantRestant}.");
    }

    #[test]
    fn line_breaks_are_preserved() {
        assert_eq!(resolve("A\n\n{clientName}", &data()), "A\n\nStark Industries");
    }

    #[test]
    fn subjects_match_the_original_wording() {
        assert_eq!(subject(Kind::Invoice, "FAC-1", "Asgard"), "Facture FAC-1 - Asgard");
        assert_eq!(subject(Kind::Reminder, "FAC-1", "Asgard"), "Rappel : Facture impayée FAC-1 - Asgard");
        assert_eq!(subject(Kind::Estimate, "DEV-1", "Asgard"), "Devis DEV-1 - Asgard");
    }

    #[test]
    fn a_blank_template_falls_back_to_the_default() {
        let blank = Settings {
            email_template_invoice: "   ".into(),
            ..Settings::default()
        };
        assert_eq!(pick(Kind::Invoice, &blank), Settings::default().email_template_invoice);
    }

    #[test]
    fn a_custom_template_takes_precedence() {
        let custom = Settings {
            email_template_estimate: "Voici le devis {documentNumber}.".into(),
            ..Settings::default()
        };
        assert_eq!(pick(Kind::Estimate, &custom), "Voici le devis {documentNumber}.");
    }

    #[test]
    fn an_absent_template_resolves_to_nothing() {
        assert_eq!(resolve("", &data()), "");
    }

    // --- Données et brouillons, sur le jeu de référence de la phase 0 ------

    const DATASET: &str = include_str!("../../../fixtures/reference-dataset.json");

    fn dataset() -> serde_json::Value {
        serde_json::from_str(DATASET).unwrap()
    }

    fn text(value: &serde_json::Value, key: &str) -> String {
        value[key].as_str().unwrap_or_default().to_string()
    }

    fn client(index: usize) -> Client {
        let c = &dataset()["clients"][index];
        Client {
            id: c["id"].as_i64().unwrap(),
            company_name: text(c, "companyName"),
            contact_name: text(c, "contactName"),
            email: text(c, "email"),
            phone: text(c, "phone"),
            address: text(c, "address"),
            created_at: text(c, "createdAt"),
        }
    }

    /// Pièce du jeu de référence, sous la forme qu'attend `compose`.
    struct Piece {
        number: String,
        description: String,
        total: f64,
        date: String,
        due_date: Option<String>,
        company: String,
    }

    impl Piece {
        fn from(value: &serde_json::Value, number_key: &str) -> Self {
            Self {
                number: text(value, number_key),
                due_date: value["dueDate"].as_str().map(str::to_string),
                description: text(value, "description"),
                total: value["amountTotal"].as_f64().unwrap(),
                date: text(value, "date"),
                company: text(value, "companyName"),
            }
        }

        fn invoice(index: usize) -> Self {
            Self::from(&dataset()["invoices"][index], "invoiceNumber")
        }

        fn unpaid_invoice() -> Self {
            let data = dataset();
            let unpaid = data["invoices"]
                .as_array()
                .unwrap()
                .iter()
                .find(|i| i["status"] == "envoyee")
                .unwrap()
                .clone();
            Self::from(&unpaid, "invoiceNumber")
        }

        fn estimate(index: usize) -> Self {
            Self::from(&dataset()["estimates"][index], "estimateNumber")
        }

        fn doc(&self) -> Document<'_> {
            Document {
                number: &self.number,
                due_date: self.due_date.as_deref(),
                description: &self.description,
                total: self.total,
                date: &self.date,
                company: &self.company,
            }
        }
    }

    fn standard_settings() -> Settings {
        let s = &dataset()["settings"]["standard"];
        Settings {
            contact_name: text(s, "contactName"),
            company_name: text(s, "companyName"),
            ..Settings::default()
        }
    }

    fn with_templates() -> Settings {
        Settings {
            email_template_invoice: "Bonjour {clientName},\n\nFacture {documentNumber} : {amountTotal} €.\n\n{senderName}\n{senderCompany}".into(),
            email_template_estimate: "Bonjour {clientName}, voici le devis {documentNumber}.".into(),
            email_template_reminder: "Bonjour {clientName}, la facture {documentNumber} du {documentDate} reste impayée.".into(),
            ..standard_settings()
        }
    }

    #[test]
    fn data_formats_the_amount_and_the_french_date() {
        let data = template_data(&Piece::invoice(1).doc(), Some(&client(0)), &standard_settings());

        assert_eq!(data.amount_total, "9720.00");
        assert_eq!(data.document_date, "05/02/2026");
        assert_eq!(data.client_name, "Stark Industries");
        assert_eq!(data.sender_company, "Asgard Solutions");
    }

    #[test]
    fn data_rounds_a_float_amount_to_the_cent() {
        let data = template_data(&Piece::invoice(2).doc(), Some(&client(1)), &standard_settings());
        assert_eq!(data.amount_total, "2279.99");
    }

    /// Le jeu de référence est antérieur aux échéances : ses factures n'en
    /// portent pas, et le jeton reste vide plutôt que d'afficher une date
    /// inventée.
    #[test]
    fn data_leaves_an_unknown_due_date_empty() {
        let data = template_data(&Piece::invoice(1).doc(), Some(&client(0)), &standard_settings());
        assert_eq!(data.due_date, "");
    }

    #[test]
    fn a_recorded_due_date_fills_the_token() {
        let mut piece = Piece::invoice(1);
        piece.due_date = Some("2026-03-07T00:00:00Z".into());

        let data = template_data(&piece.doc(), Some(&client(0)), &standard_settings());
        assert_eq!(data.due_date, "07/03/2026");
        assert_eq!(resolve("Échéance : {dueDate}.", &data), "Échéance : 07/03/2026.");
    }

    /// Écart assumé avec l'original, qui laissait le nom vide : la raison
    /// sociale recopiée sur la pièce prend le relais.
    #[test]
    fn a_deleted_client_is_named_from_the_document() {
        let data = template_data(&Piece::invoice(1).doc(), None, &standard_settings());
        assert_eq!(data.client_name, "Stark Industries");
    }

    #[test]
    fn composes_an_invoice() {
        let d = compose(Kind::Invoice, &Piece::invoice(1).doc(), Some(&client(0)), &with_templates());

        assert_eq!(d.to, "pepper@stark.com");
        assert_eq!(d.subject, "Facture FAC-STARKINDUS-2026-0001 - Asgard Solutions");
        assert_eq!(
            d.text,
            "Bonjour Stark Industries,\n\nFacture FAC-STARKINDUS-2026-0001 : 9720.00 €.\n\nThor Odinson\nAsgard Solutions"
        );
    }

    #[test]
    fn composes_a_reminder_for_an_unpaid_invoice() {
        let d = compose(Kind::Reminder, &Piece::unpaid_invoice().doc(), Some(&client(1)), &with_templates());

        assert_eq!(d.subject, "Rappel : Facture impayée FAC-WAYNEENTER-2026-0005 - Asgard Solutions");
        assert_eq!(
            d.text,
            "Bonjour Wayne Enterprises, la facture FAC-WAYNEENTER-2026-0005 du 11/06/2026 reste impayée."
        );
    }

    #[test]
    fn composes_an_estimate() {
        let d = compose(Kind::Estimate, &Piece::estimate(0).doc(), Some(&client(0)), &with_templates());

        assert_eq!(d.subject, "Devis DEV-STARKINDUS-2026-0001 - Asgard Solutions");
        assert_eq!(d.text, "Bonjour Stark Industries, voici le devis DEV-STARKINDUS-2026-0001.");
    }

    #[test]
    fn a_deleted_client_leaves_the_recipient_empty() {
        let d = compose(Kind::Invoice, &Piece::invoice(1).doc(), None, &with_templates());
        assert_eq!(d.to, "");
    }

    #[test]
    fn kinds_serialise_as_the_host_expects() {
        assert_eq!(serde_json::to_string(&Kind::Invoice).unwrap(), "\"invoice\"");
        assert_eq!(serde_json::to_string(&Kind::Reminder).unwrap(), "\"reminder\"");
        assert_eq!(serde_json::to_string(&Kind::Estimate).unwrap(), "\"estimate\"");
    }
}
