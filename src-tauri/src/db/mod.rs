//! Base de données — phase 2 de la migration.
//!
//! Remplace Dexie / IndexedDB par un fichier SQLite dans le répertoire de
//! données de l'application. Ce que le changement apporte, au-delà du langage :
//!
//! - **un fichier**, que l'utilisateur peut copier, inspecter et sauvegarder.
//!   IndexedDB vivait dans le profil du moteur de rendu et disparaissait avec
//!   lui (défaut D6) ;
//! - **des montants entiers** en centimes, au lieu de flottants (défaut D5) ;
//! - **une suppression logique** des pièces émises (défaut D4) ;
//! - **une numérotation transactionnelle** protégée par une contrainte
//!   d'unicité (défaut D3).

pub mod autosave;
pub mod backup;
pub mod money;
pub mod numbering;
pub mod repo;
pub(crate) mod seed;

use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub use seed::seed_demo_data_if_empty;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Erreur de base de données : {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration impossible : {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("Sauvegarde illisible : {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0} introuvable")]
    NotFound(&'static str),

    #[error("Répertoire de données inaccessible : {0}")]
    Io(#[from] std::io::Error),

    #[error("Édition du PDF impossible : {0}")]
    Pdf(String),
}

/// Les commandes Tauri renvoient des erreurs sérialisables.
impl serde::Serialize for DbError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Nom du fichier, visible par l'utilisateur dans le répertoire de données.
pub const DATABASE_FILE: &str = "asgard-crm.sqlite";

pub struct Db {
    pub pool: SqlitePool,
    pub path: PathBuf,
}

/// Ouvre la base, la crée au besoin et applique les migrations en attente.
pub async fn open(data_dir: PathBuf) -> Result<Db, DbError> {
    std::fs::create_dir_all(&data_dir)?;
    let path = data_dir.join(DATABASE_FILE);

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        // Journalisation WAL : une lecture longue ne bloque plus une écriture,
        // et une coupure d'alimentation laisse un fichier récupérable.
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        // Les clés étrangères ne sont pas actives par défaut sous SQLite ;
        // sans cela, `ON DELETE SET NULL` sur les pièces ne s'appliquerait pas.
        .foreign_keys(true)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        // Sans délai d'attente, une écriture qui tombe sur un verrou échoue
        // immédiatement au lieu de patienter. Cinq secondes couvrent largement
        // les transactions de cette application, toutes très courtes.
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .connect_with(options)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(Db { pool, path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrations_apply_to_a_fresh_database() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
             AND name != '_sqlx_migrations' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(
            tables,
            vec![
                "clients",
                "credit_notes",
                "document_sequences",
                "estimates",
                "expenses",
                "invoices"
            ]
        );
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
    }
}
