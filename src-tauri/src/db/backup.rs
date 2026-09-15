//! Export, reprise et sauvegarde du fichier.
//!
//! C'est le point le plus risqué de la phase 2 : on fait entrer dans une base
//! neuve des données comptables produites par une autre implémentation. Trois
//! précautions structurent ce module.
//!
//! 1. **L'import est atomique.** Tout passe dans une seule transaction : en cas
//!    d'échec, la base reste exactement dans son état antérieur plutôt que de
//!    se retrouver à moitié reprise.
//!
//! 2. **Les écarts d'arrondi sont rapportés, pas subis.** Passer des flottants
//!    aux centimes modifie certains montants. Le rapport dit lesquels, pour que
//!    l'utilisateur puisse expliquer un chiffre qui a bougé plutôt que de le
//!    découvrir dans une déclaration.
//!
//! 3. **Les séquences repartent du plus grand numéro émis**, jamais du nombre
//!    de lignes reprises — voir `numbering::seed_sequence_from_existing`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::money::{differs_after_rounding, from_cents, to_cents};
use super::numbering::{self, DocumentKind};
use super::DbError;

/// Sauvegarde produite par l'application, dans les deux formes rencontrées :
/// `{ db: {...}, settings: {...} }` depuis l'ajout des réglages, ou les tables
/// à la racine pour les toutes premières versions.
#[derive(Debug, Deserialize)]
pub struct LegacyBackup {
    #[serde(default)]
    pub db: Option<LegacyTables>,
    #[serde(flatten)]
    pub root: LegacyTables,
}

#[derive(Debug, Default, Deserialize)]
pub struct LegacyTables {
    #[serde(default)]
    pub clients: Vec<serde_json::Value>,
    #[serde(default)]
    pub invoices: Vec<serde_json::Value>,
    #[serde(default)]
    pub estimates: Vec<serde_json::Value>,
    #[serde(default)]
    pub expenses: Vec<serde_json::Value>,
}

impl LegacyBackup {
    fn tables(self) -> LegacyTables {
        match self.db {
            Some(inner) if !inner.is_empty() => inner,
            _ => self.root,
        }
    }
}

impl LegacyTables {
    fn is_empty(&self) -> bool {
        self.clients.is_empty()
            && self.invoices.is_empty()
            && self.estimates.is_empty()
            && self.expenses.is_empty()
    }
}

/// Un montant que la conversion en centimes a modifié.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Adjustment {
    pub document: String,
    pub field: String,
    pub before: f64,
    pub after: f64,
}

/// Compte rendu d'une reprise.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub clients: usize,
    pub invoices: usize,
    pub estimates: usize,
    pub expenses: usize,
    /// Montants modifiés par l'arrondi au centime.
    pub adjustments: Vec<Adjustment>,
    /// Pièces écartées, avec la raison — un numéro en doublon, par exemple.
    pub skipped: Vec<String>,
}

fn f(value: &serde_json::Value, key: &str) -> f64 {
    value.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn s(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

fn opt_s(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn i(value: &serde_json::Value, key: &str) -> Option<i64> {
    value.get(key).and_then(|v| v.as_i64())
}

/// Note un montant dont l'arrondi au centime change la valeur.
fn note(adjustments: &mut Vec<Adjustment>, document: &str, field: &str, before: f64) {
    if differs_after_rounding(before) {
        adjustments.push(Adjustment {
            document: document.to_string(),
            field: field.to_string(),
            before,
            after: from_cents(to_cents(before)),
        });
    }
}

/// Reprend une sauvegarde. Remplace intégralement le contenu de la base.
pub async fn import(pool: &SqlitePool, backup: LegacyBackup) -> Result<ImportReport, DbError> {
    let tables = backup.tables();
    let mut report = ImportReport::default();
    let mut tx = pool.begin().await?;

    // Table rase, dans la même transaction que le remplissage.
    for table in ["invoices", "estimates", "expenses", "clients", "document_sequences"] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *tx)
            .await?;
    }

    // Les identifiants de l'ancienne base sont conservés : les pièces y font
    // référence, et les réécrire romprait le lien client ↔ facture.
    for client in &tables.clients {
        let Some(id) = i(client, "id") else { continue };

        sqlx::query(
            "INSERT INTO clients (id, company_name, contact_name, email, phone, address, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(id)
        .bind(s(client, "companyName"))
        .bind(s(client, "contactName"))
        .bind(s(client, "email"))
        .bind(s(client, "phone"))
        .bind(s(client, "address"))
        .bind(opt_s(client, "createdAt").unwrap_or_else(|| chrono::Utc::now().to_rfc3339()))
        .execute(&mut *tx)
        .await?;

        report.clients += 1;
    }

    // Plus grand numéro rencontré, par type de pièce et par exercice.
    let mut highest: HashMap<(&'static str, i32), i64> = HashMap::new();

    for invoice in &tables.invoices {
        let number = s(invoice, "invoiceNumber");
        if number.is_empty() {
            report.skipped.push("facture sans numéro".into());
            continue;
        }

        note(&mut report.adjustments, &number, "amountHt", f(invoice, "amountHt"));
        note(&mut report.adjustments, &number, "amountTva", f(invoice, "amountTva"));
        note(&mut report.adjustments, &number, "amountTotal", f(invoice, "amountTotal"));

        let inserted = sqlx::query(
            "INSERT OR IGNORE INTO invoices (id, client_id, company_name, invoice_number,
                service_type, description, amount_ht_cents, tva_rate, amount_tva_cents,
                amount_total_cents, date, status, payment_date, payment_method)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        )
        .bind(i(invoice, "id"))
        .bind(i(invoice, "clientId"))
        .bind(s(invoice, "companyName"))
        .bind(&number)
        .bind(s(invoice, "serviceType"))
        .bind(s(invoice, "description"))
        .bind(to_cents(f(invoice, "amountHt")))
        .bind(f(invoice, "tvaRate"))
        .bind(to_cents(f(invoice, "amountTva")))
        .bind(to_cents(f(invoice, "amountTotal")))
        .bind(s(invoice, "date"))
        .bind(opt_s(invoice, "status").unwrap_or_else(|| "brouillon".into()))
        .bind(opt_s(invoice, "paymentDate"))
        .bind(opt_s(invoice, "paymentMethod"))
        .execute(&mut *tx)
        .await?;

        if inserted.rows_affected() == 0 {
            report.skipped.push(format!("{number} — numéro déjà présent"));
            continue;
        }

        report.invoices += 1;
        if let (Some(year), Some(seq)) = (numbering::year_of(&number), numbering::sequence_of(&number)) {
            let slot = highest.entry(("invoice", year)).or_insert(0);
            *slot = (*slot).max(seq);
        }
    }

    for estimate in &tables.estimates {
        let number = s(estimate, "estimateNumber");
        if number.is_empty() {
            report.skipped.push("devis sans numéro".into());
            continue;
        }

        note(&mut report.adjustments, &number, "amountHt", f(estimate, "amountHt"));
        note(&mut report.adjustments, &number, "amountTva", f(estimate, "amountTva"));
        note(&mut report.adjustments, &number, "amountTotal", f(estimate, "amountTotal"));

        let inserted = sqlx::query(
            "INSERT OR IGNORE INTO estimates (id, client_id, company_name, estimate_number,
                service_type, description, amount_ht_cents, tva_rate, amount_tva_cents,
                amount_total_cents, date, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(i(estimate, "id"))
        .bind(i(estimate, "clientId"))
        .bind(s(estimate, "companyName"))
        .bind(&number)
        .bind(s(estimate, "serviceType"))
        .bind(s(estimate, "description"))
        .bind(to_cents(f(estimate, "amountHt")))
        .bind(f(estimate, "tvaRate"))
        .bind(to_cents(f(estimate, "amountTva")))
        .bind(to_cents(f(estimate, "amountTotal")))
        .bind(s(estimate, "date"))
        .bind(opt_s(estimate, "status").unwrap_or_else(|| "brouillon".into()))
        .execute(&mut *tx)
        .await?;

        if inserted.rows_affected() == 0 {
            report.skipped.push(format!("{number} — numéro déjà présent"));
            continue;
        }

        report.estimates += 1;
        if let (Some(year), Some(seq)) = (numbering::year_of(&number), numbering::sequence_of(&number)) {
            let slot = highest.entry(("estimate", year)).or_insert(0);
            *slot = (*slot).max(seq);
        }
    }

    for expense in &tables.expenses {
        let label = s(expense, "merchant");
        note(&mut report.adjustments, &label, "amount", f(expense, "amount"));

        sqlx::query(
            "INSERT INTO expenses (id, date, merchant, category, amount_cents, description,
                payment_method) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(i(expense, "id"))
        .bind(s(expense, "date"))
        .bind(&label)
        .bind(opt_s(expense, "category").unwrap_or_else(|| "Autre".into()))
        .bind(to_cents(f(expense, "amount")))
        .bind(s(expense, "description"))
        .bind(opt_s(expense, "paymentMethod").unwrap_or_else(|| "carte".into()))
        .execute(&mut *tx)
        .await?;

        report.expenses += 1;
    }

    // Les séquences repartent du plus grand numéro **émis**, pas du nombre de
    // lignes reprises. Une base amputée de quelques pièces a moins de lignes
    // que de numéros attribués : repartir du compte réattribuerait des numéros
    // déjà utilisés, soit exactement le défaut D3.
    for ((kind, year), seq) in highest {
        let kind = if kind == "invoice" {
            DocumentKind::Invoice
        } else {
            DocumentKind::Estimate
        };
        numbering::seed_sequence_from_existing(&mut tx, kind, year, seq).await?;
    }

    tx.commit().await?;
    Ok(report)
}

/// Exporte les données au format de sauvegarde historique, pour rester
/// compatible avec les fichiers déjà produits par l'application.
pub async fn export(pool: &SqlitePool) -> Result<serde_json::Value, DbError> {
    let snapshot = super::repo::snapshot(pool).await?;

    Ok(serde_json::json!({
        "clients": snapshot.clients,
        "invoices": snapshot.invoices,
        "estimates": snapshot.estimates,
        "expenses": snapshot.expenses,
        "exportedAt": chrono::Utc::now().to_rfc3339(),
        "schema": "sqlite-1",
    }))
}

/// Copie atomique du fichier de base.
///
/// `VACUUM INTO` écrit une base cohérente même si l'application écrit pendant
/// l'opération, ce qu'une copie de fichier ne garantit pas. C'est ce que
/// l'ancienne version ne pouvait pas offrir : IndexedDB n'a pas de fichier.
pub async fn vacuum_into(pool: &SqlitePool, destination: &str) -> Result<(), DbError> {
    sqlx::query("VACUUM INTO ?1")
        .bind(destination)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    fn backup(json: serde_json::Value) -> LegacyBackup {
        serde_json::from_value(json).unwrap()
    }

    #[tokio::test]
    async fn imports_both_backup_shapes() {
        let nested = backup(serde_json::json!({
            "db": { "clients": [{ "id": 1, "companyName": "Stark Industries" }] },
            "settings": { "companyName": "Asgard" }
        }));
        let flat = backup(serde_json::json!({
            "clients": [{ "id": 1, "companyName": "Stark Industries" }]
        }));

        for b in [nested, flat] {
            let pool = fresh_pool().await;
            assert_eq!(import(&pool, b).await.unwrap().clients, 1);
        }
    }

    #[tokio::test]
    async fn reports_amounts_changed_by_rounding() {
        let pool = fresh_pool().await;
        let report = import(
            &pool,
            backup(serde_json::json!({
                "invoices": [{
                    "id": 1, "invoiceNumber": "FAC-WAYNEENTER-2026-0002",
                    "companyName": "Wayne", "serviceType": "service_bic", "date": "2026-03-12",
                    "amountHt": 1899.99, "tvaRate": 20.0,
                    "amountTva": 379.998_000_000_000_05, "amountTotal": 2279.988_000_000_000_3
                }]
            })),
        )
        .await
        .unwrap();

        assert_eq!(report.invoices, 1);
        assert_eq!(report.adjustments.len(), 2, "{:?}", report.adjustments);

        let tva = report.adjustments.iter().find(|a| a.field == "amountTva").unwrap();
        assert_eq!(tva.after, 380.00);
        assert_eq!(tva.document, "FAC-WAYNEENTER-2026-0002");
    }

    #[tokio::test]
    async fn leaves_clean_amounts_alone() {
        let pool = fresh_pool().await;
        let report = import(
            &pool,
            backup(serde_json::json!({
                "invoices": [{
                    "id": 1, "invoiceNumber": "FAC-STARKINDUS-2026-0001",
                    "companyName": "Stark", "serviceType": "service_bnc", "date": "2026-02-05",
                    "amountHt": 8100.0, "tvaRate": 20.0, "amountTva": 1620.0, "amountTotal": 9720.0
                }]
            })),
        )
        .await
        .unwrap();

        assert!(report.adjustments.is_empty(), "{:?}", report.adjustments);
    }

    /// Le point de vigilance de la phase : la séquence doit repartir du plus
    /// grand numéro émis, pas du nombre de lignes reprises.
    #[tokio::test]
    async fn sequences_resume_from_the_highest_issued_number() {
        let pool = fresh_pool().await;

        // Deux factures subsistent mais la plus haute porte le numéro 7.
        import(
            &pool,
            backup(serde_json::json!({
                "invoices": [
                    { "id": 1, "invoiceNumber": "FAC-STARKINDUS-2026-0002", "companyName": "Stark",
                      "serviceType": "service_bnc", "date": "2026-02-05", "amountHt": 100.0,
                      "tvaRate": 20.0, "amountTva": 20.0, "amountTotal": 120.0 },
                    { "id": 2, "invoiceNumber": "FAC-STARKINDUS-2026-0007", "companyName": "Stark",
                      "serviceType": "service_bnc", "date": "2026-06-05", "amountHt": 100.0,
                      "tvaRate": 20.0, "amountTva": 20.0, "amountTotal": 120.0 }
                ]
            })),
        )
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let next = numbering::allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap();
        tx.commit().await.unwrap();

        assert_eq!(next, 8, "la séquence est repartie du nombre de lignes au lieu du plus haut numéro");
    }

    #[tokio::test]
    async fn skips_duplicate_numbers_instead_of_failing() {
        let pool = fresh_pool().await;
        let report = import(
            &pool,
            backup(serde_json::json!({
                "invoices": [
                    { "id": 1, "invoiceNumber": "FAC-STARKINDUS-2026-0001", "companyName": "Stark",
                      "serviceType": "service_bnc", "date": "2026-01-05", "amountHt": 100.0,
                      "tvaRate": 20.0, "amountTva": 20.0, "amountTotal": 120.0 },
                    { "id": 2, "invoiceNumber": "FAC-STARKINDUS-2026-0001", "companyName": "Stark",
                      "serviceType": "service_bnc", "date": "2026-02-05", "amountHt": 200.0,
                      "tvaRate": 20.0, "amountTva": 40.0, "amountTotal": 240.0 }
                ]
            })),
        )
        .await
        .unwrap();

        assert_eq!(report.invoices, 1);
        assert_eq!(report.skipped.len(), 1);
        assert!(report.skipped[0].contains("déjà présent"));
    }

    #[tokio::test]
    async fn import_replaces_previous_content() {
        let pool = fresh_pool().await;

        import(&pool, backup(serde_json::json!({
            "clients": [{ "id": 1, "companyName": "Ancien" }]
        }))).await.unwrap();

        import(&pool, backup(serde_json::json!({
            "clients": [{ "id": 9, "companyName": "Nouveau" }]
        }))).await.unwrap();

        let clients = super::super::repo::list_clients(&pool).await.unwrap();
        assert_eq!(clients.len(), 1);
        assert_eq!(clients[0].company_name, "Nouveau");
    }

    #[tokio::test]
    async fn export_round_trips_through_import() {
        let pool = fresh_pool().await;
        import(&pool, backup(serde_json::json!({
            "clients": [{ "id": 1, "companyName": "Stark Industries", "email": "a@b.fr" }],
            "expenses": [{ "id": 1, "date": "2026-01-15", "merchant": "OVH", "amount": 119.0 }]
        }))).await.unwrap();

        let exported = export(&pool).await.unwrap();
        let second = fresh_pool().await;
        let report = import(&second, serde_json::from_value(exported).unwrap()).await.unwrap();

        assert_eq!((report.clients, report.expenses), (1, 1));
        assert!(report.adjustments.is_empty());
    }
}
