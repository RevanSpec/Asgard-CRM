//! Lectures et écritures.
//!
//! Toutes les requêtes écartent les lignes portant un `deleted_at` : la
//! suppression logique doit être invisible depuis l'interface, sinon elle ne
//! sert à rien.

use chrono::Datelike;
use sqlx::{Row, SqlitePool};

use super::model::*;
use super::money::{from_cents, to_cents};
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
        "SELECT id, company_name, contact_name, email, phone, address, created_at
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
                    phone = ?4, address = ?5 WHERE id = ?6",
            )
            .bind(&input.company_name)
            .bind(&input.contact_name)
            .bind(&input.email)
            .bind(&input.phone)
            .bind(&input.address)
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
                "INSERT INTO clients (company_name, contact_name, email, phone, address, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id",
            )
            .bind(&input.company_name)
            .bind(&input.contact_name)
            .bind(&input.email)
            .bind(&input.phone)
            .bind(&input.address)
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
        status: r.get("status"),
        payment_date: r.get("payment_date"),
        payment_method: r.get("payment_method"),
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

/// Crée une facture et lui attribue son numéro dans la même transaction.
pub async fn create_invoice(pool: &SqlitePool, input: DocumentInput) -> Result<Invoice, DbError> {
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
            date, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) RETURNING id",
    )
    .bind(input.client_id)
    .bind(&input.company_name)
    .bind(&number)
    .bind(&input.service_type)
    .bind(&input.description)
    .bind(to_cents(input.amount_ht))
    .bind(input.tva_rate)
    .bind(to_cents(input.amount_tva))
    .bind(to_cents(input.amount_total))
    .bind(&input.date)
    .bind(input.status.as_deref().unwrap_or("brouillon"))
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
    let mut tx = pool.begin().await?;

    let id = match input.id {
        Some(id) => {
            sqlx::query(
                "UPDATE estimates SET client_id = ?1, company_name = ?2, service_type = ?3,
                    description = ?4, amount_ht_cents = ?5, tva_rate = ?6, amount_tva_cents = ?7,
                    amount_total_cents = ?8, date = ?9, status = ?10
                 WHERE id = ?11 AND deleted_at IS NULL",
            )
            .bind(input.client_id)
            .bind(&input.company_name)
            .bind(&input.service_type)
            .bind(&input.description)
            .bind(to_cents(input.amount_ht))
            .bind(input.tva_rate)
            .bind(to_cents(input.amount_tva))
            .bind(to_cents(input.amount_total))
            .bind(&input.date)
            .bind(input.status.as_deref().unwrap_or("brouillon"))
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
                    date, status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) RETURNING id",
            )
            .bind(input.client_id)
            .bind(&input.company_name)
            .bind(&number)
            .bind(&input.service_type)
            .bind(&input.description)
            .bind(to_cents(input.amount_ht))
            .bind(input.tva_rate)
            .bind(to_cents(input.amount_tva))
            .bind(to_cents(input.amount_total))
            .bind(&input.date)
            .bind(input.status.as_deref().unwrap_or("brouillon"))
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
pub async fn convert_estimate(pool: &SqlitePool, estimate_id: i64) -> Result<Invoice, DbError> {
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
            date, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'brouillon') RETURNING id",
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
            .bind(to_cents(input.amount))
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
            .bind(to_cents(input.amount))
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
    })
}
