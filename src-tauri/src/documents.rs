//! Édition et envoi des pièces — phase 4.
//!
//! Deux changements de fond par rapport à l'implémentation JavaScript.
//!
//! **Le PDF ne traverse plus la frontière.** L'interface générait le document
//! avec jsPDF, l'encodait en base64 et le passait à l'hôte pour qu'il l'attache.
//! Désormais l'hôte lit la pièce en base, produit le PDF et l'attache lui-même :
//! `send_document(id, kind)` suffit, et rien de volumineux ne transite par
//! l'IPC.
//!
//! **L'enregistrement passe par un dialogue natif.** L'interface déclenchait un
//! téléchargement via un `<a download>` pointant sur une URL `blob:` — un chemin
//! que la CSP de Tauri rend incertain, et qui était resté non vérifié depuis la
//! phase 1. C'est désormais une écriture de fichier côté Rust, après le
//! sélecteur du système.

use asgard_core::model::{CivilDate, Operation, ServiceType};
use asgard_core::money::from_cents;
use asgard_pdf::{Document, DocumentKind, Issuer, Party};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};

use crate::db::DbError;

/// Réglages d'édition, tels que l'interface les envoie.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuerInput {
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub contact_name: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub siret: String,
    #[serde(default)]
    pub iban: String,
    #[serde(default)]
    pub custom_color: String,
    #[serde(default)]
    pub logo_base64: String,
    /// Médiateur de la consommation (art. L616-1 du code de la consommation).
    #[serde(default)]
    pub mediator: String,
    /// Assurance professionnelle (art. L112-11 du code des assurances).
    #[serde(default)]
    pub insurance: String,
    /// Option pour le paiement de la TVA d'après les débits.
    #[serde(default)]
    pub vat_on_debits: bool,
}

impl From<IssuerInput> for Issuer {
    /// Les champs traversent tels quels.
    ///
    /// L'ancienne conversion remplaçait une raison sociale vide par « Mon
    /// Auto-Entreprise », une adresse vide par « Votre Adresse » : la pièce
    /// sortait, au nom de personne, et rien à l'écran ne l'en distinguait
    /// (défaut D11). Un champ vide reste vide, et `render` refuse alors
    /// d'éditer.
    ///
    /// Seule la couleur d'accentuation garde un repli : un trait doit bien être
    /// tracé dans une teinte, et celle-là n'engage rien.
    fn from(input: IssuerInput) -> Self {
        let colour = input.custom_color.trim();

        Issuer {
            company_name: input.company_name,
            contact_name: input.contact_name,
            address: input.address,
            phone: input.phone,
            email: input.email,
            siret: input.siret,
            iban: input.iban,
            accent_colour: if colour.is_empty() {
                asgard_pdf::DEFAULT_ACCENT.to_string()
            } else {
                colour.to_string()
            },
            logo: input.logo_base64,
            mediator: input.mediator,
            insurance: input.insurance,
            vat_on_debits: input.vat_on_debits,
        }
    }
}

/// Nature de la pièce, telle que l'interface la nomme.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Invoice,
    Estimate,
    /// Une relance porte sur une facture : même document, autre message.
    Reminder,
    /// Avoir sur une facture.
    Credit,
}

impl Kind {
    pub fn document_kind(self) -> DocumentKind {
        match self {
            Kind::Estimate => DocumentKind::Estimate,
            Kind::Credit => DocumentKind::CreditNote,
            _ => DocumentKind::Invoice,
        }
    }

    fn table(self) -> &'static str {
        match self {
            Kind::Estimate => "estimates",
            Kind::Credit => "credit_notes",
            _ => "invoices",
        }
    }

    fn number_column(self) -> &'static str {
        match self {
            Kind::Estimate => "estimate_number",
            Kind::Credit => "credit_number",
            _ => "invoice_number",
        }
    }

    /// Un avoir nomme la facture qu'il corrige : elle se lit par une jointure.
    fn corrected_invoice(self) -> (&'static str, &'static str) {
        match self {
            Kind::Credit => (", i.invoice_number AS corrects", "LEFT JOIN invoices i ON i.id = d.invoice_id"),
            _ => ("", ""),
        }
    }
}

fn service_type(raw: &str) -> ServiceType {
    match raw {
        "service_bic" => ServiceType::ServiceBic,
        "vente" => ServiceType::Vente,
        _ => ServiceType::ServiceBnc,
    }
}

/// Ce qu'il faut pour éditer une pièce : le document et son destinataire.
pub struct Rendered {
    pub number: String,
    pub bytes: Vec<u8>,
}

/// Lit la pièce en base, avec les coordonnées du client.
///
/// Un client supprimé ne bloque pas l'édition : la raison sociale est recopiée
/// sur la pièce, et les autres champs retombent sur un libellé explicite —
/// exactement ce que faisait le JavaScript.
async fn load(pool: &SqlitePool, kind: Kind, id: i64) -> Result<(Document, Party), DbError> {
    let (corrects_column, corrects_join) = kind.corrected_invoice();
    let row = sqlx::query(&format!(
        "SELECT d.*, c.contact_name, c.address, c.phone, c.email,
                c.siren, c.vat_number, c.delivery_address{corrects_column}
         FROM {table} d LEFT JOIN clients c ON c.id = d.client_id {corrects_join}
         WHERE d.id = ?1 AND d.deleted_at IS NULL",
        table = kind.table()
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound("pièce"))?;

    let date: String = row.get("date");

    let document = Document {
        number: row.get(kind.number_column()),
        service_type: service_type(&row.get::<String, _>("service_type")),
        description: row.get("description"),
        amount_ht: from_cents(row.get("amount_ht_cents")),
        tva_rate: asgard_core::from_f64(row.get("tva_rate")),
        amount_tva: from_cents(row.get("amount_tva_cents")),
        amount_total: from_cents(row.get("amount_total_cents")),
        date: CivilDate::parse(&date).unwrap_or(CivilDate::new(1970, 1, 1)),
        // Les devis n'ont pas de colonne d'échéance ; les factures d'avant la
        // migration 0002 l'ont nulle.
        due_date: row
            .try_get::<Option<String>, _>("due_date")
            .ok()
            .flatten()
            .and_then(|iso| CivilDate::parse(&iso)),
        corrects: row.try_get::<Option<String>, _>("corrects").ok().flatten(),
        // Les avoirs n'ont pas de colonne de nature : ils reprennent le type
        // d'activité de la facture corrigée, dont le PDF déduit la mention.
        operation: row
            .try_get::<Option<String>, _>("operation_kind")
            .ok()
            .flatten()
            .and_then(|stored| Operation::parse(&stored)),
    };

    let client = Party {
        company_name: row.get("company_name"),
        contact_name: row
            .get::<Option<String>, _>("contact_name")
            .unwrap_or_else(|| "Client supprimé".into()),
        address: row.get::<Option<String>, _>("address").unwrap_or_else(|| "N/A".into()),
        phone: row.get::<Option<String>, _>("phone").unwrap_or_else(|| "N/A".into()),
        email: row.get::<Option<String>, _>("email").unwrap_or_else(|| "N/A".into()),
        // Un client supprimé ne laisse pas d'identifiants : mieux vaut ne rien
        // imprimer qu'un « N/A » à la place d'un SIREN.
        siren: row.get::<Option<String>, _>("siren").unwrap_or_default(),
        vat_number: row.get::<Option<String>, _>("vat_number").unwrap_or_default(),
        delivery_address: row.get::<Option<String>, _>("delivery_address").unwrap_or_default(),
    };

    Ok((document, client))
}

/// Édite une pièce et renvoie le PDF.
pub async fn render(
    pool: &SqlitePool,
    kind: Kind,
    id: i64,
    issuer: Issuer,
) -> Result<Rendered, DbError> {
    // Refus avant même de lire la pièce : ce qui manque n'est pas la facture,
    // c'est l'émetteur. Le garde est ici plutôt que dans chaque commande, parce
    // que l'édition et l'envoi par e-mail passent tous deux par cette fonction.
    let missing = issuer.missing_fields(kind.document_kind());
    if !missing.is_empty() {
        return Err(DbError::Incomplete(missing.join(", ")));
    }

    let (document, client) = load(pool, kind, id).await?;
    let number = document.number.clone();

    let bytes = asgard_pdf::render(
        kind.document_kind(),
        &document,
        &client,
        &issuer,
        today(),
    )
    .map_err(|error| DbError::Pdf(error.to_string()))?;

    Ok(Rendered { number, bytes })
}

/// Adresse e-mail enregistrée du client d'une pièce, si elle existe.
pub async fn recipient(pool: &SqlitePool, kind: Kind, id: i64) -> Result<Option<String>, DbError> {
    let email: Option<String> = sqlx::query_scalar(&format!(
        "SELECT c.email FROM {table} d JOIN clients c ON c.id = d.client_id WHERE d.id = ?1",
        table = kind.table()
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?
    .flatten();

    Ok(email.filter(|value| !value.is_empty()))
}

fn today() -> CivilDate {
    let now = chrono::Local::now().date_naive();
    use chrono::Datelike;
    CivilDate::new(now.year(), now.month(), now.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_document_kinds() {
        assert_eq!(Kind::Credit.document_kind(), DocumentKind::CreditNote);
        assert_eq!(Kind::Invoice.document_kind(), DocumentKind::Invoice);
        assert_eq!(Kind::Estimate.document_kind(), DocumentKind::Estimate);

        // Une relance réédite la facture : même document, seul le message change.
        assert_eq!(Kind::Reminder.document_kind(), DocumentKind::Invoice);
        assert_eq!(Kind::Reminder.table(), "invoices");
    }

    /// Défaut D11 : un champ vide reste vide. Ce test remplace celui qui
    /// vérifiait le contraire — « Mon Auto-Entreprise » et « Votre Adresse »
    /// étaient imprimés sur de vraies factures.
    #[test]
    fn issuer_fields_stay_empty_when_blank() {
        let input: IssuerInput = serde_json::from_str(r#"{"siret": "123"}"#).unwrap();
        let issuer: Issuer = input.into();

        assert_eq!(issuer.company_name, "");
        assert_eq!(issuer.address, "");
        assert_eq!(issuer.siret, "123");

        // La couleur, elle, garde son repli : elle n'engage rien.
        assert_eq!(issuer.accent_colour, "#E5A93C");
    }

    /// Ce que l'interface doit afficher quand les réglages sont incomplets.
    #[test]
    fn an_incomplete_issuer_names_what_is_missing() {
        let input: IssuerInput = serde_json::from_str(r#"{"companyName": "Gjallarhorn SARL"}"#).unwrap();
        let missing = Issuer::from(input).missing_fields(DocumentKind::Invoice);

        assert_eq!(missing, vec!["l'adresse", "le SIRET", "l'IBAN"]);
        assert_eq!(
            DbError::Incomplete(missing.join(", ")).to_string(),
            "Vos informations d'entreprise sont incomplètes : il manque l'adresse, le SIRET, \
             l'IBAN. Renseignez-les dans l'onglet « Paramètres » avant d'éditer une pièce."
        );
    }

    /// Le logo traverse la frontière : c'est ce qui manquait depuis la phase 4,
    /// où le générateur Rust ignorait purement et simplement le réglage.
    #[test]
    fn the_issuer_carries_its_logo() {
        let input: IssuerInput =
            serde_json::from_str(r#"{"logoBase64": "data:image/png;base64,iVBOR"}"#).unwrap();
        let issuer: Issuer = input.into();

        assert_eq!(issuer.logo, "data:image/png;base64,iVBOR");

        // Sans réglage, pas de logo — et pas d'erreur non plus.
        let empty: IssuerInput = serde_json::from_str("{}").unwrap();
        assert!(Issuer::from(empty).logo.is_empty());
    }

    #[test]
    fn issuer_keeps_supplied_values() {
        // Delimiteur allonge : la valeur contient elle-meme un '#'.
        let input: IssuerInput = serde_json::from_str(
            r##"{"companyName": "Asgard Solutions", "customColor": "#7C3AED"}"##,
        )
        .unwrap();
        let issuer: Issuer = input.into();

        assert_eq!(issuer.company_name, "Asgard Solutions");
        assert_eq!(issuer.accent_colour, "#7C3AED");
    }

    #[test]
    fn a_whitespace_only_field_counts_as_blank() {
        let input: IssuerInput = serde_json::from_str(r#"{"companyName": "   "}"#).unwrap();
        let issuer: Issuer = input.into();

        // La valeur n'est pas nettoyée — inutile — mais elle ne remplit rien.
        assert!(issuer
            .missing_fields(DocumentKind::Invoice)
            .contains(&"la raison sociale"));
    }
}
