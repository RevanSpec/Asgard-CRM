//! Jeu de démonstration du premier lancement.
//!
//! Il vivait dans `App.jsx` (`seedDatabase`), où il souffrait d'un défaut que
//! le passage côté Rust corrige au passage : sous `StrictMode`, React invoque
//! l'effet de montage deux fois en développement, et les deux passes lisaient
//! « base vide » avant qu'aucune n'ait écrit. Les données de démonstration
//! étaient donc insérées en double, visiblement dans le livre des recettes.
//!
//! Ici, le test et l'insertion sont dans la même transaction : un second appel
//! concurrent voit la base déjà remplie et ne fait rien.

use sqlx::SqlitePool;

use super::numbering::{self, DocumentKind};
use super::DbError;

struct DemoInvoice {
    company: &'static str,
    client: i64,
    service_type: &'static str,
    description: &'static str,
    ht_cents: i64,
    tva_rate: f64,
    tva_cents: i64,
    total_cents: i64,
    date: &'static str,
    status: &'static str,
    payment_date: Option<&'static str>,
}

/// Insère le jeu de démonstration si la base est vide. Sans effet sinon.
///
/// La transaction est ouverte en `BEGIN IMMEDIATE`, et c'est essentiel. Une
/// transaction SQLite ordinaire est *deferred* : elle ne prend qu'un verrou de
/// lecture, et ne tente de passer en écriture qu'au premier `INSERT`. Deux
/// appels simultanés lisent donc tous deux « base vide », puis se bloquent
/// mutuellement en tentant d'écrire — SQLite renvoie « database is deadlocked ».
///
/// C'est exactement le scénario que ce module est censé rendre impossible :
/// `BEGIN IMMEDIATE` prend le verrou d'écriture dès l'ouverture, le second
/// appel attend, puis constate que la base est déjà remplie et ne fait rien.
pub async fn seed_demo_data_if_empty(pool: &SqlitePool) -> Result<bool, DbError> {
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;

    let clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(&mut *tx)
        .await?;

    if clients > 0 {
        return Ok(false);
    }

    let demo_clients = [
        ("Stark Industries", "Pepper Potts", "pepper@stark.com", "06 11 22 33 44", "108 route de Malibu, 75008 Paris", "2026-01-10T00:00:00Z"),
        ("Wayne Enterprises", "Lucius Fox", "lucius@wayne.com", "07 88 99 00 11", "12 avenue de Gotham, 75016 Paris", "2026-02-15T00:00:00Z"),
        ("Asgard Coffee", "Valkyrie", "valk@coffee.asgard", "06 55 55 55 55", "45 rue du Bifrost, 75011 Paris", "2026-03-20T00:00:00Z"),
    ];

    for (company, contact, email, phone, address, created) in demo_clients {
        sqlx::query(
            "INSERT INTO clients (company_name, contact_name, email, phone, address, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(company)
        .bind(contact)
        .bind(email)
        .bind(phone)
        .bind(address)
        .bind(created)
        .execute(&mut *tx)
        .await?;
    }

    let demo_invoices = [
        DemoInvoice { company: "Stark Industries", client: 1, service_type: "service_bnc", description: "Consulting en nanotechnologies", ht_cents: 600_000, tva_rate: 20.0, tva_cents: 120_000, total_cents: 720_000, date: "2026-02-05T10:00:00Z", status: "payee", payment_date: Some("2026-02-05T10:00:00Z") },
        DemoInvoice { company: "Wayne Enterprises", client: 2, service_type: "service_bnc", description: "Audit système de défense sonar", ht_cents: 1_250_000, tva_rate: 20.0, tva_cents: 250_000, total_cents: 1_500_000, date: "2026-03-12T10:00:00Z", status: "payee", payment_date: Some("2026-03-12T10:00:00Z") },
        DemoInvoice { company: "Asgard Coffee", client: 3, service_type: "vente", description: "Livraison de grains de café d'Éthiopie", ht_cents: 140_000, tva_rate: 5.5, tva_cents: 7_700, total_cents: 147_700, date: "2026-04-18T10:00:00Z", status: "payee", payment_date: Some("2026-04-18T10:00:00Z") },
        DemoInvoice { company: "Stark Industries", client: 1, service_type: "service_bnc", description: "Optimisation de l'IA Jarvis", ht_cents: 450_000, tva_rate: 20.0, tva_cents: 90_000, total_cents: 540_000, date: "2026-05-02T10:00:00Z", status: "payee", payment_date: Some("2026-05-02T10:00:00Z") },
        DemoInvoice { company: "Wayne Enterprises", client: 2, service_type: "service_bnc", description: "Maintenance des systèmes de sécurité", ht_cents: 900_000, tva_rate: 20.0, tva_cents: 180_000, total_cents: 1_080_000, date: "2026-06-25T10:00:00Z", status: "payee", payment_date: Some("2026-06-25T10:00:00Z") },
        DemoInvoice { company: "Asgard Coffee", client: 3, service_type: "vente", description: "Réassort trimestriel", ht_cents: 280_000, tva_rate: 5.5, tva_cents: 15_400, total_cents: 295_400, date: "2026-07-14T10:00:00Z", status: "envoyee", payment_date: None },
        DemoInvoice { company: "Stark Industries", client: 1, service_type: "service_bnc", description: "Étude de faisabilité réacteur", ht_cents: 320_000, tva_rate: 20.0, tva_cents: 64_000, total_cents: 384_000, date: "2026-08-30T10:00:00Z", status: "brouillon", payment_date: None },
    ];

    for invoice in demo_invoices {
        let year = 2026;
        let sequence = numbering::allocate(&mut tx, DocumentKind::Invoice, year).await?;
        let number =
            numbering::format_number(DocumentKind::Invoice, invoice.company, year, sequence);

        sqlx::query(
            "INSERT INTO invoices (client_id, company_name, invoice_number, service_type,
                description, amount_ht_cents, tva_rate, amount_tva_cents, amount_total_cents,
                date, status, payment_date, payment_method)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )
        .bind(invoice.client)
        .bind(invoice.company)
        .bind(&number)
        .bind(invoice.service_type)
        .bind(invoice.description)
        .bind(invoice.ht_cents)
        .bind(invoice.tva_rate)
        .bind(invoice.tva_cents)
        .bind(invoice.total_cents)
        .bind(invoice.date)
        .bind(invoice.status)
        .bind(invoice.payment_date)
        .bind(invoice.payment_date.map(|_| "virement"))
        .execute(&mut *tx)
        .await?;
    }

    let demo_estimates = [
        ("Stark Industries", 1_i64, "service_bnc", "Refonte du réacteur ARC", 2_400_000_i64, 20.0_f64, 480_000_i64, 2_880_000_i64, "2026-01-20T10:00:00Z", "accepte"),
        ("Wayne Enterprises", 2, "service_bic", "Extension du réseau de capteurs", 1_560_000, 20.0, 312_000, 1_872_000, "2026-06-14T10:00:00Z", "envoye"),
    ];

    for (company, client, service_type, description, ht, rate, tva, total, date, status) in demo_estimates {
        let year = 2026;
        let sequence = numbering::allocate(&mut tx, DocumentKind::Estimate, year).await?;
        let number = numbering::format_number(DocumentKind::Estimate, company, year, sequence);

        sqlx::query(
            "INSERT INTO estimates (client_id, company_name, estimate_number, service_type,
                description, amount_ht_cents, tva_rate, amount_tva_cents, amount_total_cents,
                date, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .bind(client)
        .bind(company)
        .bind(&number)
        .bind(service_type)
        .bind(description)
        .bind(ht)
        .bind(rate)
        .bind(tva)
        .bind(total)
        .bind(date)
        .bind(status)
        .execute(&mut *tx)
        .await?;
    }

    let demo_expenses = [
        ("2026-02-10T00:00:00Z", "OVHcloud", "Logiciels", 4_999_i64, "Hébergement VPS Asgard CRM", "carte"),
        ("2026-03-01T00:00:00Z", "Adobe Creative Cloud", "Logiciels", 3_599, "Abonnement Photoshop/Illustrator", "carte"),
        ("2026-04-05T00:00:00Z", "SNCF", "Déplacements", 12_000, "Trajet Paris-Lyon rendez-vous client", "carte"),
    ];

    for (date, merchant, category, cents, description, method) in demo_expenses {
        sqlx::query(
            "INSERT INTO expenses (date, merchant, category, amount_cents, description,
                payment_method) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(date)
        .bind(merchant)
        .bind(category)
        .bind(cents)
        .bind(description)
        .bind(method)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn seeds_once_and_only_once() {
        let pool = fresh_pool().await;

        assert!(seed_demo_data_if_empty(&pool).await.unwrap());
        assert!(!seed_demo_data_if_empty(&pool).await.unwrap());
        assert!(!seed_demo_data_if_empty(&pool).await.unwrap());

        let clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
            .fetch_one(&pool)
            .await
            .unwrap();
        let invoices: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoices")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!((clients, invoices), (3, 7));
    }

    /// Le bug que le déplacement côté Rust corrige : deux appels concurrents
    /// ne doivent pas produire deux jeux de données.
    #[tokio::test]
    async fn concurrent_calls_do_not_duplicate_the_demo_data() {
        let pool = fresh_pool().await;

        let (a, b) = tokio::join!(
            seed_demo_data_if_empty(&pool),
            seed_demo_data_if_empty(&pool)
        );
        a.unwrap();
        b.unwrap();

        let clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(clients, 3, "les données de démonstration ont été insérées en double");
    }

    #[tokio::test]
    async fn seeded_invoices_carry_a_continuous_sequence() {
        let pool = fresh_pool().await;
        seed_demo_data_if_empty(&pool).await.unwrap();

        let numbers: Vec<String> =
            sqlx::query_scalar("SELECT invoice_number FROM invoices ORDER BY id")
                .fetch_all(&pool)
                .await
                .unwrap();

        let sequences: Vec<i64> = numbers.iter().filter_map(|n| numbering::sequence_of(n)).collect();
        assert_eq!(sequences, vec![1, 2, 3, 4, 5, 6, 7]);
    }
}
