//! Réglages conservés en base.
//!
//! Ils vivaient dans le `localStorage` de la WebView, que la copie quotidienne
//! ne couvre pas : l'identité, le logo, les gabarits d'e-mail et la
//! configuration SMTP n'étaient dans aucune sauvegarde automatique (défaut
//! D13). La base fait foi ; le `localStorage` reste le cache que l'interface lit
//! pendant le rendu.
//!
//! Le témoin de premier lancement vit ici aussi, parce que la question « cette
//! installation a-t-elle déjà servi ? » se répond en regardant la base, pas le
//! navigateur.

use sqlx::SqlitePool;

use asgard_ipc::StoredSettings;

use super::DbError;

/// Clé du JSON des réglages.
const SETTINGS: &str = "settings";

/// Clé du témoin de premier lancement.
const FIRST_RUN_DONE: &str = "first_run_done";

/// Réglages enregistrés et état du premier lancement, en un aller-retour.
pub async fn load(pool: &SqlitePool) -> Result<StoredSettings, DbError> {
    Ok(StoredSettings {
        settings: read(pool, SETTINGS).await?,
        first_run_done: first_run_done(pool).await?,
    })
}

/// Enregistre les réglages, tels que l'interface les sérialise.
///
/// Le JSON est relu avant d'être écrit : un cache abîmé ne doit pas remplacer
/// des réglages valables par une chaîne que personne ne saura rouvrir.
pub async fn save(pool: &SqlitePool, json: &str) -> Result<(), DbError> {
    let _: serde_json::Value = serde_json::from_str(json)?;
    write(pool, SETTINGS, json).await
}

/// Le premier lancement est-il derrière nous ?
///
/// Vrai dès que l'écran d'accueil a été rempli, **ou** que la base contient déjà
/// du travail : une installation existante, simplement mise à jour, n'a pas à
/// revoir un accueil qui ne lui apprendrait rien. Les pièces supprimées comptent
/// — elles ont bien été saisies un jour.
pub async fn first_run_done(pool: &SqlitePool) -> Result<bool, DbError> {
    if read(pool, FIRST_RUN_DONE).await?.is_some() {
        return Ok(true);
    }

    let rows: i64 = sqlx::query_scalar(
        "SELECT (SELECT COUNT(*) FROM clients)
              + (SELECT COUNT(*) FROM invoices)
              + (SELECT COUNT(*) FROM estimates)
              + (SELECT COUNT(*) FROM expenses)",
    )
    .fetch_one(pool)
    .await?;

    Ok(rows > 0)
}

/// Note que l'accueil a été rempli, pour ne plus le proposer.
pub async fn mark_first_run_done(pool: &SqlitePool) -> Result<(), DbError> {
    write(pool, FIRST_RUN_DONE, "1").await
}

async fn read(pool: &SqlitePool, key: &str) -> Result<Option<String>, DbError> {
    Ok(sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await?)
}

async fn write(pool: &SqlitePool, key: &str, value: &str) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
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

    #[tokio::test]
    async fn a_fresh_database_holds_no_settings_and_awaits_its_first_run() {
        let pool = fresh_pool().await;
        let stored = load(&pool).await.unwrap();

        assert_eq!(stored.settings, None);
        assert!(!stored.first_run_done);
    }

    #[tokio::test]
    async fn settings_survive_a_round_trip_and_the_last_write_wins() {
        let pool = fresh_pool().await;

        save(&pool, r#"{"companyName":"Gjallarhorn SARL"}"#).await.unwrap();
        assert_eq!(
            load(&pool).await.unwrap().settings.as_deref(),
            Some(r#"{"companyName":"Gjallarhorn SARL"}"#)
        );

        // Une seconde écriture remplace la première : une seule ligne, pas une
        // pile d'historique.
        save(&pool, r#"{"companyName":"Gjallarhorn SAS"}"#).await.unwrap();
        assert_eq!(
            load(&pool).await.unwrap().settings.as_deref(),
            Some(r#"{"companyName":"Gjallarhorn SAS"}"#)
        );

        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_settings WHERE key = 'settings'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(rows, 1);
    }

    #[tokio::test]
    async fn illegible_settings_are_refused() {
        let pool = fresh_pool().await;

        save(&pool, r#"{"companyName":"valable"}"#).await.unwrap();
        assert!(save(&pool, "ceci n'est pas du JSON").await.is_err());

        // Les réglages valables sont toujours là : un cache abîmé n'écrase rien.
        assert_eq!(
            load(&pool).await.unwrap().settings.as_deref(),
            Some(r#"{"companyName":"valable"}"#)
        );
    }

    #[tokio::test]
    async fn the_welcome_screen_is_shown_once() {
        let pool = fresh_pool().await;
        assert!(!first_run_done(&pool).await.unwrap());

        mark_first_run_done(&pool).await.unwrap();
        assert!(first_run_done(&pool).await.unwrap());

        // Idempotent : deux passages par l'accueil ne posent pas deux témoins.
        mark_first_run_done(&pool).await.unwrap();
        assert!(first_run_done(&pool).await.unwrap());
    }

    /// Le cas de la mise à jour : la base d'un utilisateur existant n'a pas de
    /// témoin, mais elle contient son travail. Lui proposer un premier
    /// lancement serait absurde, et l'inviter à ressaisir son identité pire
    /// encore.
    #[tokio::test]
    async fn an_existing_installation_never_sees_the_welcome_screen() {
        let pool = fresh_pool().await;

        sqlx::query("INSERT INTO clients (company_name, created_at) VALUES ('Déjà là', '2026-01-01')")
            .execute(&pool)
            .await
            .unwrap();

        assert!(first_run_done(&pool).await.unwrap());
    }

    /// Une dépense seule suffit : le travail ne commence pas forcément par un
    /// client.
    #[tokio::test]
    async fn any_recorded_work_counts_as_an_existing_installation() {
        let pool = fresh_pool().await;

        sqlx::query(
            "INSERT INTO expenses (date, merchant, category, amount_cents, description, payment_method)
             VALUES ('2026-01-02', 'Forge', 'Matériel', 4990, '', 'carte')",
        )
        .execute(&pool)
        .await
        .unwrap();

        assert!(first_run_done(&pool).await.unwrap());
    }
}
