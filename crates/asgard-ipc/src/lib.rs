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
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,
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
}
