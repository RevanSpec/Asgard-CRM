//! Formes échangées avec l'interface.
//!
//! Elles reproduisent exactement ce que Dexie renvoyait — mêmes noms de champs
//! en camelCase, mêmes types — pour que les composants React n'aient pas à
//! changer. Les montants repassent en décimal à cette frontière ; la base, elle,
//! ne connaît que des centimes.

use serde::{Deserialize, Serialize};

// Chaque type porte `Serialize` **et** `Deserialize` : l'hôte renvoie les uns et
// reçoit les autres, l'interface fait l'inverse. Les séparer obligerait à
// deviner qui sérialise quoi, et c'est exactement l'ambiguïté que ce crate
// supprime.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: i64,
    pub company_name: String,
    pub contact_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    /// SIREN du client. Mention obligatoire des factures entre professionnels
    /// avec la facturation électronique (décret n° 2022-1299).
    #[serde(default)]
    pub siren: String,
    /// Numéro de TVA intracommunautaire, quand le client en a un.
    #[serde(default)]
    pub vat_number: String,
    /// Adresse de livraison, à mentionner lorsqu'elle diffère de l'adresse de
    /// facturation.
    #[serde(default)]
    pub delivery_address: String,
    pub created_at: String,
}

/// Saisie d'un client. `id` absent = création.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInput {
    #[serde(default)]
    pub id: Option<i64>,
    pub company_name: String,
    #[serde(default)]
    pub contact_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub siren: String,
    #[serde(default)]
    pub vat_number: String,
    #[serde(default)]
    pub delivery_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i64,
    pub client_id: Option<i64>,
    pub company_name: String,
    pub invoice_number: String,
    pub service_type: String,
    pub description: String,
    pub amount_ht: f64,
    pub tva_rate: f64,
    pub amount_tva: f64,
    pub amount_total: f64,
    pub date: String,
    /// Date à laquelle le règlement doit intervenir. Absente sur les factures
    /// émises avant que l'échéance ne soit enregistrée.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,
    /// Nature de l'opération : « biens », « services » ou « mixte ». Absente
    /// des pièces émises avant qu'elle ne soit demandée ; le PDF la déduit
    /// alors du type d'activité.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub id: i64,
    pub client_id: Option<i64>,
    pub company_name: String,
    pub estimate_number: String,
    pub service_type: String,
    pub description: String,
    pub amount_ht: f64,
    pub tva_rate: f64,
    pub amount_tva: f64,
    pub amount_total: f64,
    pub date: String,
    pub status: String,
    /// Nature de l'opération : « biens », « services » ou « mixte ». Absente
    /// des pièces émises avant qu'elle ne soit demandée ; le PDF la déduit
    /// alors du type d'activité.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub id: i64,
    pub date: String,
    pub merchant: String,
    pub category: String,
    pub amount: f64,
    pub description: String,
    pub payment_method: String,
}

/// Saisie commune aux factures et aux devis.
///
/// Le numéro de pièce n'y figure pas : il est attribué par la base, dans la
/// transaction d'insertion. L'interface ne peut plus en proposer un, ce qui
/// ferme la porte au défaut D3 côté appelant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    #[serde(default)]
    pub id: Option<i64>,
    pub client_id: i64,
    pub company_name: String,
    pub service_type: String,
    #[serde(default)]
    pub description: String,
    pub amount_ht: f64,
    pub tva_rate: f64,
    pub date: String,
    /// Délai de règlement en jours, tel que les réglages l'annoncent. L'hôte en
    /// déduit l'échéance et l'enregistre : une facture émise ne bouge plus.
    #[serde(default)]
    pub payment_terms_days: Option<u32>,
    /// Nature de l'opération, telle que le formulaire la propose : elle part du
    /// type d'activité et reste modifiable, le cas mixte ne se devinant pas.
    #[serde(default)]
    pub operation_kind: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

impl DocumentInput {
    /// TVA et total, recalculés à l'écriture.
    ///
    /// L'interface envoyait ces deux montants ; elle ne les envoie plus. Les
    /// dériver ici garantit l'invariant `HT + TVA = TTC` dans la base, au lieu
    /// de dépendre de la bonne foi de l'appelant — et le calcul se fait en
    /// arithmétique décimale, pas en virgule flottante.
    pub fn amounts(&self) -> asgard_core::money::ComputedAmounts {
        asgard_core::compute_amounts(
            asgard_core::from_f64(self.amount_ht),
            asgard_core::from_f64(self.tva_rate),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseInput {
    #[serde(default)]
    pub id: Option<i64>,
    pub date: String,
    pub merchant: String,
    #[serde(default)]
    pub category: String,
    pub amount: f64,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub payment_method: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInput {
    pub invoice_id: i64,
    pub payment_date: String,
    pub payment_method: String,
}

/// Tout ce que l'interface charge au démarrage, en un aller-retour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub clients: Vec<Client>,
    pub invoices: Vec<Invoice>,
    pub estimates: Vec<Estimate>,
    pub expenses: Vec<Expense>,
    pub credit_notes: Vec<CreditNote>,
}

/// Avoir : annule ou corrige une facture émise, qui ne se modifie pas.
///
/// Les montants sont **positifs** — c'est ainsi que la pièce se lit. Le signe
/// s'applique à l'agrégation, où l'avoir devient une recette négative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNote {
    pub id: i64,
    pub invoice_id: i64,
    /// Numéro de la facture corrigée, pour l'affichage et le PDF.
    pub invoice_number: String,
    pub client_id: Option<i64>,
    pub company_name: String,
    pub credit_number: String,
    pub service_type: String,
    pub description: String,
    pub amount_ht: f64,
    pub tva_rate: f64,
    pub amount_tva: f64,
    pub amount_total: f64,
    pub date: String,
    /// Date du remboursement effectif. Tant qu'elle est absente, l'avoir ne
    /// diminue que le facturé, pas l'encaissé.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refunded_on: Option<String>,
}

/// Saisie d'un avoir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteInput {
    pub invoice_id: i64,
    #[serde(default)]
    pub description: String,
    pub amount_ht: f64,
    pub date: String,
    #[serde(default)]
    pub refunded_on: Option<String>,
}

/// Réglages tels que la base les conserve, et état du premier lancement.
///
/// Les réglages vivaient dans le `localStorage` de la WebView, hors de portée
/// de toute sauvegarde automatique : la copie quotidienne est un `VACUUM INTO`
/// du fichier SQLite, et l'identité, le logo et la configuration SMTP étaient
/// ailleurs (défaut D13). La base fait désormais foi ; le `localStorage` reste
/// un cache synchrone, parce que l'interface lit les réglages pendant le rendu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    /// JSON des réglages, tel que l'interface l'a écrit. Absent d'une base qui
    /// n'en a jamais reçu.
    #[serde(default)]
    pub settings: Option<String>,
    /// Faux uniquement sur une base vierge dont personne n'a encore rempli
    /// l'écran d'accueil : une installation qui contient déjà du travail n'a
    /// pas à le revoir.
    pub first_run_done: bool,
}
