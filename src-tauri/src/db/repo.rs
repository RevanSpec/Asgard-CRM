//! Lectures et écritures.
//!
//! Toutes les requêtes écartent les lignes portant un `deleted_at` : la
//! suppression logique doit être invisible depuis l'interface, sinon elle ne
//! sert à rien.

use chrono::Datelike;
use sqlx::{Row, SqlitePool};

use asgard_ipc::*;
use super::money::from_cents;
use asgard_core::model::{Operation, ServiceType};
use asgard_core::money::to_cents as decimal_to_cents;
use asgard_core::CivilDate;
use super::numbering::{self, DocumentKind};
use super::DbError;

/// Année civile d'une date ISO, en heure locale.
///
/// L'ancienne implémentation lisait l'année en heure locale tout en construisant
/// ses bornes en UTC — une facture du 1er janvier à 00 h 30 sortait de la
/// fenêtre de son propre exercice. Ici il n'y a plus de bornes : la séquence est
/// indexée par l'année, lue une seule fois et de façon cohérente.
fn year_of(date: &str) -> i32 {
    chrono::DateTime::parse_from_rfc3339(date)
        .map(|d| d.with_timezone(&chrono::Local).year())
        .unwrap_or_else(|_| chrono::Local::now().year())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------- clients

pub async fn list_clients(pool: &SqlitePool) -> Result<Vec<Client>, DbError> {
    let rows = sqlx::query(
        "SELECT id, company_name, contact_name, email, phone, address,
                siren, vat_number, delivery_address, created_at
         FROM clients WHERE deleted_at IS NULL ORDER BY company_name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Client {
            id: r.get("id"),
            company_name: r.get("company_name"),
            contact_name: r.get("contact_name"),
            email: r.get("email"),
            phone: r.get("phone"),
            address: r.get("address"),
            siren: r.get("siren"),
            vat_number: r.get("vat_number"),
            delivery_address: r.get("delivery_address"),
            created_at: r.get("created_at"),
        })
        .collect())
}

pub async fn save_client(pool: &SqlitePool, input: ClientInput) -> Result<i64, DbError> {
    let mut tx = pool.begin().await?;

    let id = match input.id {
        Some(id) => {
            sqlx::query(
                "UPDATE clients SET company_name = ?1, contact_name = ?2, email = ?3,
                    phone = ?4, address = ?5, siren = ?6, vat_number = ?7,
                    delivery_address = ?8 WHERE id = ?9",
            )
            .bind(&input.company_name)
            .bind(&input.contact_name)
            .bind(&input.email)
            .bind(&input.phone)
            .bind(&input.address)
            .bind(&input.siren)
            .bind(&input.vat_number)
            .bind(&input.delivery_address)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            // La raison sociale est recopiée sur les pièces : elles doivent
            // suivre le renommage, comme le faisait déjà l'implémentation Dexie.
            for table in ["invoices", "estimates"] {
                sqlx::query(&format!(
                    "UPDATE {table} SET company_name = ?1 WHERE client_id = ?2"
                ))
                .bind(&input.company_name)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }

            id
        }
        None => {
            sqlx::query_scalar(
                "INSERT INTO clients (company_name, contact_name, email, phone, address,
                    siren, vat_number, delivery_address, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) RETURNING id",
            )
            .bind(&input.company_name)
            .bind(&input.contact_name)
            .bind(&input.email)
            .bind(&input.phone)
            .bind(&input.address)
            .bind(&input.siren)
            .bind(&input.vat_number)
            .bind(&input.delivery_address)
            .bind(now())
            .fetch_one(&mut *tx)
            .await?
        }
    };

    tx.commit().await?;
    Ok(id)
}

/// Supprime un client. Ses pièces restent, détachées — c'est le comportement
/// annoncé par l'interface depuis toujours (« resteront dans l'historique mais
/// déconnectées »), et la clé étrangère `ON DELETE SET NULL` le garantit.
pub async fn delete_client(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("UPDATE clients SET deleted_at = ?1 WHERE id = ?2")
        .bind(now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// --------------------------------------------------------------- invoices

fn invoice_from_row(r: &sqlx::sqlite::SqliteRow) -> Invoice {
    Invoice {
        id: r.get("id"),
        client_id: r.get("client_id"),
        company_name: r.get("company_name"),
        invoice_number: r.get("invoice_number"),
        service_type: r.get("service_type"),
        description: r.get("description"),
        amount_ht: from_cents(r.get("amount_ht_cents")),
        tva_rate: r.get("tva_rate"),
        amount_tva: from_cents(r.get("amount_tva_cents")),
        amount_total: from_cents(r.get("amount_total_cents")),
        date: r.get("date"),
        due_date: r.get("due_date"),
        status: r.get("status"),
        payment_date: r.get("payment_date"),
        payment_method: r.get("payment_method"),
        operation_kind: r.get("operation_kind"),
    }
}

pub async fn list_invoices(pool: &SqlitePool) -> Result<Vec<Invoice>, DbError> {
    let rows = sqlx::query(
        "SELECT * FROM invoices WHERE deleted_at IS NULL ORDER BY date DESC, id DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(invoice_from_row).collect())
}

/// Échéance de règlement d'une facture émise à `date`.
///
/// Le délai vient des réglages de l'interface ; trente jours à défaut, la durée
/// que le pied de page annonçait déjà. Une date illisible ne produit pas
/// d'échéance plutôt qu'une fausse.
fn due_date(date: &str, terms_days: Option<u32>) -> Option<String> {
    let days = terms_days.unwrap_or(DEFAULT_PAYMENT_TERMS_DAYS);
    CivilDate::parse(date).map(|d| d.plus_days(days).to_iso())
}

/// Délai de règlement retenu quand les réglages n'en donnent pas.
pub const DEFAULT_PAYMENT_TERMS_DAYS: u32 = 30;

/// Nature de l'opération enregistrée avec la pièce.
///
/// Le formulaire la propose, déduite du type d'activité, et l'utilisateur peut
/// la corriger — une opération mixte ne se devine pas. À défaut de saisie, la
/// déduction s'applique : une pièce créée aujourd'hui ne doit pas naître sans
/// cette mention, qui devient obligatoire avec la facturation électronique.
///
/// Les pièces antérieures, elles, gardent leur nature nulle : le PDF la déduit
/// à l'impression plutôt que de réécrire une pièce déjà émise.
fn operation_kind(input: &DocumentInput) -> String {
    input
        .operation_kind
        .as_deref()
        .and_then(Operation::parse)
        .unwrap_or_else(|| {
            Operation::from_service_type(ServiceType::from_stored(&input.service_type))
        })
        .as_str()
        .to_string()
}

/// Crée une facture et lui attribue son numéro dans la même transaction.
pub async fn create_invoice(pool: &SqlitePool, input: DocumentInput) -> Result<Invoice, DbError> {
    let amounts = input.amounts();
    let mut tx = pool.begin().await?;

    let year = year_of(&input.date);
    let sequence = numbering::allocate(&mut tx, DocumentKind::Invoice, year).await?;
    let number = numbering::format_number(
        DocumentKind::Invoice,
        &input.company_name,
        year,
        sequence,
    );

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO invoices (client_id, company_name, invoice_number, service_type,
            description, amount_ht_cents, tva_rate, amount_tva_cents, amount_total_cents,
            date, due_date, status, operation_kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) RETURNING id",
    )
    .bind(input.client_id)
    .bind(&input.company_name)
    .bind(&number)
    .bind(&input.service_type)
    .bind(&input.description)
    .bind(decimal_to_cents(amounts.amount_ht))
    .bind(input.tva_rate)
    .bind(decimal_to_cents(amounts.amount_tva))
    .bind(decimal_to_cents(amounts.amount_total))
    .bind(&input.date)
    .bind(due_date(&input.date, input.payment_terms_days))
    .bind(input.status.as_deref().unwrap_or("brouillon"))
    .bind(operation_kind(&input))
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    let row = sqlx::query("SELECT * FROM invoices WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(invoice_from_row(&row))
}

pub async fn set_invoice_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
    sqlx::query("UPDATE invoices SET status = ?1 WHERE id = ?2 AND deleted_at IS NULL")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn record_payment(pool: &SqlitePool, input: PaymentInput) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE invoices SET status = 'payee', payment_date = ?1, payment_method = ?2
         WHERE id = ?3 AND deleted_at IS NULL",
    )
    .bind(&input.payment_date)
    .bind(&input.payment_method)
    .bind(input.invoice_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Supprime des factures.
///
/// Correction du défaut D4 : une facture émise n'est jamais effacée, seulement
/// marquée. Un brouillon, lui, n'a aucune valeur comptable et disparaît pour de
/// bon. Le décompte renvoyé permet à l'interface de dire ce qui s'est passé.
pub async fn delete_invoices(pool: &SqlitePool, ids: &[i64]) -> Result<DeleteOutcome, DbError> {
    let mut tx = pool.begin().await?;
    let mut outcome = DeleteOutcome::default();

    for &id in ids {
        let status: Option<String> =
            sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?1 AND deleted_at IS NULL")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;

        match status.as_deref() {
            None => continue,
            Some("brouillon") => {
                sqlx::query("DELETE FROM invoices WHERE id = ?1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                outcome.discarded += 1;
            }
            Some(_) => {
                sqlx::query("UPDATE invoices SET deleted_at = ?1 WHERE id = ?2")
                    .bind(now())
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                outcome.archived += 1;
            }
        }
    }

    tx.commit().await?;
    Ok(outcome)
}

/// Ce qu'une suppression a réellement fait.
#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOutcome {
    /// Brouillons réellement effacés.
    pub discarded: usize,
    /// Pièces émises, conservées mais retirées de l'affichage.
    pub archived: usize,
}

// -------------------------------------------------------------- estimates

fn estimate_from_row(r: &sqlx::sqlite::SqliteRow) -> Estimate {
    Estimate {
        id: r.get("id"),
        client_id: r.get("client_id"),
        company_name: r.get("company_name"),
        estimate_number: r.get("estimate_number"),
        service_type: r.get("service_type"),
        description: r.get("description"),
        amount_ht: from_cents(r.get("amount_ht_cents")),
        tva_rate: r.get("tva_rate"),
        amount_tva: from_cents(r.get("amount_tva_cents")),
        amount_total: from_cents(r.get("amount_total_cents")),
        date: r.get("date"),
        status: r.get("status"),
        operation_kind: r.get("operation_kind"),
    }
}

pub async fn list_estimates(pool: &SqlitePool) -> Result<Vec<Estimate>, DbError> {
    let rows = sqlx::query(
        "SELECT * FROM estimates WHERE deleted_at IS NULL ORDER BY date DESC, id DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(estimate_from_row).collect())
}

pub async fn save_estimate(pool: &SqlitePool, input: DocumentInput) -> Result<Estimate, DbError> {
    let amounts = input.amounts();
    let mut tx = pool.begin().await?;

    let id = match input.id {
        Some(id) => {
            sqlx::query(
                "UPDATE estimates SET client_id = ?1, company_name = ?2, service_type = ?3,
                    description = ?4, amount_ht_cents = ?5, tva_rate = ?6, amount_tva_cents = ?7,
                    amount_total_cents = ?8, date = ?9, status = ?10, operation_kind = ?11
                 WHERE id = ?12 AND deleted_at IS NULL",
            )
            .bind(input.client_id)
            .bind(&input.company_name)
            .bind(&input.service_type)
            .bind(&input.description)
            .bind(decimal_to_cents(amounts.amount_ht))
            .bind(input.tva_rate)
            .bind(decimal_to_cents(amounts.amount_tva))
            .bind(decimal_to_cents(amounts.amount_total))
            .bind(&input.date)
            .bind(input.status.as_deref().unwrap_or("brouillon"))
            .bind(operation_kind(&input))
            .bind(id)
            .execute(&mut *tx)
            .await?;
            id
        }
        None => {
            let year = year_of(&input.date);
            let sequence = numbering::allocate(&mut tx, DocumentKind::Estimate, year).await?;
            let number = numbering::format_number(
                DocumentKind::Estimate,
                &input.company_name,
                year,
                sequence,
            );

            sqlx::query_scalar(
                "INSERT INTO estimates (client_id, company_name, estimate_number, service_type,
                    description, amount_ht_cents, tva_rate, amount_tva_cents, amount_total_cents,
                    date, status, operation_kind)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) RETURNING id",
            )
            .bind(input.client_id)
            .bind(&input.company_name)
            .bind(&number)
            .bind(&input.service_type)
            .bind(&input.description)
            .bind(decimal_to_cents(amounts.amount_ht))
            .bind(input.tva_rate)
            .bind(decimal_to_cents(amounts.amount_tva))
            .bind(decimal_to_cents(amounts.amount_total))
            .bind(&input.date)
            .bind(input.status.as_deref().unwrap_or("brouillon"))
            .bind(operation_kind(&input))
            .fetch_one(&mut *tx)
            .await?
        }
    };

    tx.commit().await?;

    let row = sqlx::query("SELECT * FROM estimates WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(estimate_from_row(&row))
}

pub async fn set_estimate_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
    sqlx::query("UPDATE estimates SET status = ?1 WHERE id = ?2 AND deleted_at IS NULL")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Même règle que pour les factures : un devis accepté a engagé le client, il
/// s'archive au lieu de disparaître.
pub async fn delete_estimate(pool: &SqlitePool, id: i64) -> Result<DeleteOutcome, DbError> {
    let mut outcome = DeleteOutcome::default();

    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM estimates WHERE id = ?1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(pool)
            .await?;

    match status.as_deref() {
        None => {}
        Some("brouillon") => {
            sqlx::query("DELETE FROM estimates WHERE id = ?1")
                .bind(id)
                .execute(pool)
                .await?;
            outcome.discarded += 1;
        }
        Some(_) => {
            sqlx::query("UPDATE estimates SET deleted_at = ?1 WHERE id = ?2")
                .bind(now())
                .bind(id)
                .execute(pool)
                .await?;
            outcome.archived += 1;
        }
    }

    Ok(outcome)
}

/// Convertit un devis en facture : la facture reçoit un numéro neuf, le devis
/// passe en « accepté ». Les deux dans la même transaction, pour qu'un échec ne
/// laisse pas un devis accepté sans facture.
pub async fn convert_estimate(
    pool: &SqlitePool,
    estimate_id: i64,
    payment_terms_days: Option<u32>,
) -> Result<Invoice, DbError> {
    let mut tx = pool.begin().await?;

    let est = sqlx::query("SELECT * FROM estimates WHERE id = ?1 AND deleted_at IS NULL")
        .bind(estimate_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(DbError::NotFound("devis"))?;

    let company_name: String = est.get("company_name");
    let date = now();
    let year = year_of(&date);
    let sequence = numbering::allocate(&mut tx, DocumentKind::Invoice, year).await?;
    let number = numbering::format_number(DocumentKind::Invoice, &company_name, year, sequence);

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO invoices (client_id, company_name, invoice_number, service_type,
            description, amount_ht_cents, tva_rate, amount_tva_cents, amount_total_cents,
            date, due_date, status, operation_kind)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'brouillon', ?12) RETURNING id",
    )
    .bind(est.get::<Option<i64>, _>("client_id"))
    .bind(&company_name)
    .bind(&number)
    .bind(est.get::<String, _>("service_type"))
    .bind(est.get::<String, _>("description"))
    .bind(est.get::<i64, _>("amount_ht_cents"))
    .bind(est.get::<f64, _>("tva_rate"))
    .bind(est.get::<i64, _>("amount_tva_cents"))
    .bind(est.get::<i64, _>("amount_total_cents"))
    .bind(&date)
    .bind(due_date(&date, payment_terms_days))
    // La facture dit ce que le devis disait : accepter un devis ne change pas
    // la nature de ce qui est vendu.
    .bind(est.get::<Option<String>, _>("operation_kind"))
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("UPDATE estimates SET status = 'accepte' WHERE id = ?1")
        .bind(estimate_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    let row = sqlx::query("SELECT * FROM invoices WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(invoice_from_row(&row))
}

// --------------------------------------------------------------- expenses

pub async fn list_expenses(pool: &SqlitePool) -> Result<Vec<Expense>, DbError> {
    let rows =
        sqlx::query("SELECT * FROM expenses WHERE deleted_at IS NULL ORDER BY date DESC, id DESC")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .into_iter()
        .map(|r| Expense {
            id: r.get("id"),
            date: r.get("date"),
            merchant: r.get("merchant"),
            category: r.get("category"),
            amount: from_cents(r.get("amount_cents")),
            description: r.get("description"),
            payment_method: r.get("payment_method"),
        })
        .collect())
}

pub async fn save_expense(pool: &SqlitePool, input: ExpenseInput) -> Result<i64, DbError> {
    let id = match input.id {
        Some(id) => {
            sqlx::query(
                "UPDATE expenses SET date = ?1, merchant = ?2, category = ?3, amount_cents = ?4,
                    description = ?5, payment_method = ?6 WHERE id = ?7 AND deleted_at IS NULL",
            )
            .bind(&input.date)
            .bind(&input.merchant)
            .bind(&input.category)
            .bind(decimal_to_cents(asgard_core::from_f64(input.amount)))
            .bind(&input.description)
            .bind(&input.payment_method)
            .bind(id)
            .execute(pool)
            .await?;
            id
        }
        None => {
            sqlx::query_scalar(
                "INSERT INTO expenses (date, merchant, category, amount_cents, description,
                    payment_method) VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id",
            )
            .bind(&input.date)
            .bind(&input.merchant)
            .bind(&input.category)
            .bind(decimal_to_cents(asgard_core::from_f64(input.amount)))
            .bind(&input.description)
            .bind(&input.payment_method)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(id)
}

/// Une dépense n'est pas une pièce émise : elle se supprime réellement.
pub async fn delete_expense(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
    sqlx::query("DELETE FROM expenses WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// --------------------------------------------------------------- snapshot

pub async fn snapshot(pool: &SqlitePool) -> Result<Snapshot, DbError> {
    Ok(Snapshot {
        clients: list_clients(pool).await?,
        invoices: list_invoices(pool).await?,
        estimates: list_estimates(pool).await?,
        expenses: list_expenses(pool).await?,
        credit_notes: list_credit_notes(pool).await?,
    })
}

// ----------------------------------------------------------- credit notes

fn credit_note_from_row(r: &sqlx::sqlite::SqliteRow) -> CreditNote {
    CreditNote {
        id: r.get("id"),
        invoice_id: r.get("invoice_id"),
        invoice_number: r.get("invoice_number"),
        client_id: r.get("client_id"),
        company_name: r.get("company_name"),
        credit_number: r.get("credit_number"),
        service_type: r.get("service_type"),
        description: r.get("description"),
        amount_ht: from_cents(r.get("amount_ht_cents")),
        tva_rate: r.get("tva_rate"),
        amount_tva: from_cents(r.get("amount_tva_cents")),
        amount_total: from_cents(r.get("amount_total_cents")),
        date: r.get("date"),
        refunded_on: r.get("refunded_on"),
    }
}

pub async fn list_credit_notes(pool: &SqlitePool) -> Result<Vec<CreditNote>, DbError> {
    let rows = sqlx::query(
        "SELECT a.*, i.invoice_number FROM credit_notes a
         JOIN invoices i ON i.id = a.invoice_id
         WHERE a.deleted_at IS NULL ORDER BY a.date DESC, a.id DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(credit_note_from_row).collect())
}

/// Émet un avoir sur une facture.
///
/// Le taux de TVA et le type d'activité sont **repris de la facture** plutôt
/// que saisis : un avoir corrige une pièce précise, et un taux différent du
/// sien fausserait aussi bien la TVA que l'assiette des cotisations.
pub async fn create_credit_note(
    pool: &SqlitePool,
    input: CreditNoteInput,
) -> Result<CreditNote, DbError> {
    let mut tx = pool.begin().await?;

    let invoice = sqlx::query(
        "SELECT * FROM invoices WHERE id = ?1 AND deleted_at IS NULL",
    )
    .bind(input.invoice_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(DbError::NotFound("facture"))?;

    let company_name: String = invoice.get("company_name");
    let tva_rate: f64 = invoice.get("tva_rate");
    let amounts = asgard_core::compute_amounts(
        asgard_core::from_f64(input.amount_ht),
        asgard_core::from_f64(tva_rate),
    );

    let year = year_of(&input.date);
    let sequence = numbering::allocate(&mut tx, DocumentKind::CreditNote, year).await?;
    let number =
        numbering::format_number(DocumentKind::CreditNote, &company_name, year, sequence);

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO credit_notes (invoice_id, client_id, company_name, credit_number,
            service_type, description, amount_ht_cents, tva_rate, amount_tva_cents,
            amount_total_cents, date, refunded_on)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) RETURNING id",
    )
    .bind(input.invoice_id)
    .bind(invoice.get::<Option<i64>, _>("client_id"))
    .bind(&company_name)
    .bind(&number)
    .bind(invoice.get::<String, _>("service_type"))
    .bind(&input.description)
    .bind(decimal_to_cents(amounts.amount_ht))
    .bind(tva_rate)
    .bind(decimal_to_cents(amounts.amount_tva))
    .bind(decimal_to_cents(amounts.amount_total))
    .bind(&input.date)
    .bind(&input.refunded_on)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    let row = sqlx::query(
        "SELECT a.*, i.invoice_number FROM credit_notes a
         JOIN invoices i ON i.id = a.invoice_id WHERE a.id = ?1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(credit_note_from_row(&row))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Base vierge, avec le client que les pièces de test facturent — la clé
    /// étrangère l'exige.
    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        save_client(
            &pool,
            ClientInput {
                id: None,
                company_name: "Asgard Coffee & Co".into(),
                contact_name: "Valkyrie".into(),
                email: "valk@coffee.asgard".into(),
                phone: "06 55 55 55 55".into(),
                address: "45 rue du Bifrost, 75011 Paris".into(),
                siren: String::new(),
                vat_number: String::new(),
                delivery_address: String::new(),
            },
        )
        .await
        .unwrap();

        pool
    }

    fn input(date: &str, terms: Option<u32>) -> DocumentInput {
        DocumentInput {
            id: None,
            client_id: 1,
            company_name: "Asgard Coffee & Co".into(),
            service_type: "vente".into(),
            description: "Grains de café".into(),
            amount_ht: 1000.0,
            tva_rate: 20.0,
            date: date.into(),
            payment_terms_days: terms,
            operation_kind: None,
            status: None,
        }
    }


    // ----------------------------------- identifiants et nature de l'opération

    /// Les identifiants du client font l'aller-retour : ils seront exigés sur
    /// les factures avec la facturation électronique, et devoir les ressaisir le
    /// jour venu serait le seul vrai coût de cette migration.
    #[tokio::test]
    async fn client_identifiers_survive_a_round_trip() {
        let pool = fresh_pool().await;

        save_client(
            &pool,
            ClientInput {
                id: Some(1),
                company_name: "Asgard Coffee & Co".into(),
                contact_name: "Valkyrie".into(),
                email: "valk@coffee.asgard".into(),
                phone: "06 55 55 55 55".into(),
                address: "45 rue du Bifrost, 75011 Paris".into(),
                siren: "552 100 554".into(),
                vat_number: "FR 12 552100554".into(),
                delivery_address: "7 quai de Nidavellir, 29200 Brest".into(),
            },
        )
        .await
        .unwrap();

        let client = list_clients(&pool).await.unwrap().remove(0);
        assert_eq!(client.siren, "552 100 554");
        assert_eq!(client.vat_number, "FR 12 552100554");
        assert_eq!(client.delivery_address, "7 quai de Nidavellir, 29200 Brest");
    }

    /// Une pièce créée aujourd'hui porte toujours la mention : à défaut de
    /// saisie, elle se déduit du type d'activité.
    #[tokio::test]
    async fn a_new_document_always_records_its_operation() {
        let pool = fresh_pool().await;

        // « vente » : une livraison de biens.
        let invoice = create_invoice(&pool, input("2026-04-18T10:00:00Z", None)).await.unwrap();
        assert_eq!(invoice.operation_kind.as_deref(), Some("biens"));

        let services = DocumentInput {
            service_type: "service_bnc".into(),
            ..input("2026-04-18T10:00:00Z", None)
        };
        let invoice = create_invoice(&pool, services).await.unwrap();
        assert_eq!(invoice.operation_kind.as_deref(), Some("services"));
    }

    /// Le cas mixte ne se déduit d'aucun type d'activité : il ne peut venir que
    /// de la saisie, et doit donc l'emporter sur la déduction.
    #[tokio::test]
    async fn a_chosen_operation_overrides_the_deduction() {
        let pool = fresh_pool().await;

        let mixed = DocumentInput {
            operation_kind: Some("mixte".into()),
            ..input("2026-04-18T10:00:00Z", None)
        };
        let invoice = create_invoice(&pool, mixed).await.unwrap();
        assert_eq!(invoice.operation_kind.as_deref(), Some("mixte"));

        // Une valeur que personne ne sait lire ne s'enregistre pas telle quelle.
        let nonsense = DocumentInput {
            operation_kind: Some("marchandises".into()),
            ..input("2026-04-18T10:00:00Z", None)
        };
        let invoice = create_invoice(&pool, nonsense).await.unwrap();
        assert_eq!(invoice.operation_kind.as_deref(), Some("biens"));
    }

    /// Accepter un devis ne change pas la nature de ce qui est vendu.
    #[tokio::test]
    async fn conversion_carries_the_operation_over() {
        let pool = fresh_pool().await;

        let estimate = DocumentInput {
            operation_kind: Some("mixte".into()),
            ..input("2026-04-18T10:00:00Z", None)
        };
        let estimate = save_estimate(&pool, estimate).await.unwrap();
        assert_eq!(estimate.operation_kind.as_deref(), Some("mixte"));

        let invoice = convert_estimate(&pool, estimate.id, None).await.unwrap();
        assert_eq!(invoice.operation_kind.as_deref(), Some("mixte"));
    }

    #[test]
    fn the_due_date_follows_the_configured_delay() {
        assert_eq!(
            due_date("2026-04-18T10:00:00Z", Some(45)),
            Some("2026-06-02".to_string())
        );
    }

    /// Sans réglage, le délai reste celui que le pied de page annonçait déjà.
    #[test]
    fn an_absent_delay_falls_back_to_thirty_days() {
        assert_eq!(due_date("2026-04-18T10:00:00Z", None), Some("2026-05-18".to_string()));
    }

    /// Une date illisible ne produit pas d'échéance : mieux vaut aucune date
    /// qu'une date fausse sur une pièce qui ne se corrige que par avoir.
    #[test]
    fn an_unreadable_date_yields_no_due_date() {
        assert_eq!(due_date("pas une date", Some(30)), None);
    }

    #[tokio::test]
    async fn a_created_invoice_carries_its_due_date() {
        let pool = fresh_pool().await;

        let invoice = create_invoice(&pool, input("2026-04-18T10:00:00Z", Some(45)))
            .await
            .unwrap();

        assert_eq!(invoice.due_date.as_deref(), Some("2026-06-02"));

        // Et elle ressort telle quelle de la base, pas seulement de l'insertion.
        let listed = list_invoices(&pool).await.unwrap();
        assert_eq!(listed[0].due_date.as_deref(), Some("2026-06-02"));
    }

    /// Le délai est figé à l'émission : changer le réglage ensuite ne déplace
    /// pas l'échéance d'une facture déjà envoyée.
    #[tokio::test]
    async fn changing_the_delay_leaves_past_invoices_alone() {
        let pool = fresh_pool().await;

        let first = create_invoice(&pool, input("2026-04-18T10:00:00Z", Some(30)))
            .await
            .unwrap();
        let second = create_invoice(&pool, input("2026-04-18T10:00:00Z", Some(60)))
            .await
            .unwrap();

        assert_eq!(first.due_date.as_deref(), Some("2026-05-18"));
        assert_eq!(second.due_date.as_deref(), Some("2026-06-17"));

        let listed = list_invoices(&pool).await.unwrap();
        let first_again = listed.iter().find(|i| i.id == first.id).unwrap();
        assert_eq!(first_again.due_date.as_deref(), Some("2026-05-18"));
    }

    /// Un avoir reprend le taux de TVA et le type d'activité de sa facture :
    /// l'assiette des cotisations en dépend.
    #[tokio::test]
    async fn a_credit_note_inherits_the_invoice_rate_and_activity() {
        let pool = fresh_pool().await;
        let invoice = create_invoice(&pool, input("2026-04-18T10:00:00Z", None))
            .await
            .unwrap();

        let credit = create_credit_note(
            &pool,
            CreditNoteInput {
                invoice_id: invoice.id,
                description: "Geste commercial".into(),
                amount_ht: 250.0,
                date: "2026-05-02T00:00:00Z".into(),
                refunded_on: None,
            },
        )
        .await
        .unwrap();

        assert!(credit.credit_number.starts_with("AVO-ASGARDCOFF-2026-"));
        assert_eq!(credit.invoice_number, invoice.invoice_number);
        assert_eq!(credit.service_type, invoice.service_type);
        assert_eq!(credit.tva_rate, invoice.tva_rate);
        // Montants positifs sur la pièce : c'est ainsi qu'elle se lit.
        assert_eq!(credit.amount_ht, 250.0);
        assert_eq!(credit.amount_tva, 50.0);
        assert_eq!(credit.amount_total, 300.0);
        assert!(credit.refunded_on.is_none());
    }

    /// Les avoirs ont leur propre série : ils ne consomment pas les numéros de
    /// facture, et la suite des factures reste sans rupture.
    #[tokio::test]
    async fn credit_notes_are_numbered_in_their_own_series() {
        let pool = fresh_pool().await;
        let invoice = create_invoice(&pool, input("2026-04-18T10:00:00Z", None))
            .await
            .unwrap();

        let make = |n: f64| CreditNoteInput {
            invoice_id: invoice.id,
            description: String::new(),
            amount_ht: n,
            date: "2026-05-02T00:00:00Z".into(),
            refunded_on: None,
        };

        let first = create_credit_note(&pool, make(100.0)).await.unwrap();
        let second = create_credit_note(&pool, make(50.0)).await.unwrap();
        let next_invoice = create_invoice(&pool, input("2026-05-03T10:00:00Z", None))
            .await
            .unwrap();

        assert!(first.credit_number.ends_with("-0001"));
        assert!(second.credit_number.ends_with("-0002"));
        assert!(next_invoice.invoice_number.ends_with("-0002"), "{}", next_invoice.invoice_number);
    }

    #[tokio::test]
    async fn a_credit_note_needs_an_existing_invoice() {
        let pool = fresh_pool().await;
        let outcome = create_credit_note(
            &pool,
            CreditNoteInput {
                invoice_id: 4242,
                description: String::new(),
                amount_ht: 10.0,
                date: "2026-05-02T00:00:00Z".into(),
                refunded_on: None,
            },
        )
        .await;

        assert!(matches!(outcome, Err(DbError::NotFound("facture"))));
    }

    #[tokio::test]
    async fn a_converted_estimate_gets_a_due_date_too() {
        let pool = fresh_pool().await;

        let estimate = save_estimate(
            &pool,
            DocumentInput {
                status: Some("envoye".into()),
                ..input("2026-04-18T10:00:00Z", None)
            },
        )
        .await
        .unwrap();

        let invoice = convert_estimate(&pool, estimate.id, Some(30)).await.unwrap();

        assert!(invoice.due_date.is_some(), "la facture issue d'un devis porte une échéance");
        assert!(invoice.invoice_number.starts_with("FAC-"));
    }
}
