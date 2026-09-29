//! Copie quotidienne de la base, conservée par roulement.
//!
//! L'export JSON existait déjà, mais il faut y penser. Un fichier SQLite peut
//! se corrompre — disque plein, coupure pendant une écriture, antivirus trop
//! curieux — et l'utilisateur ne s'en aperçoit qu'au moment où il ouvre
//! l'application, c'est-à-dire trop tard.
//!
//! La copie est prise au démarrage, une fois par jour, par `VACUUM INTO` :
//! SQLite écrit alors une base complète et cohérente, journal WAL compris, ce
//! qu'une copie de fichier ne garantit pas.
//!
//! Elle ne remplace pas l'export : les sauvegardes vivent à côté de la base,
//! donc sur le même disque. Contre une panne matérielle, seul l'export, rangé
//! ailleurs, protège — c'est ce que disent les réglages.

use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use super::DbError;

/// Nombre de copies conservées.
pub const KEPT_COPIES: usize = 7;

/// Dossier des copies, à côté de la base.
pub fn directory(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

fn file_name(day: &str) -> String {
    format!("asgard-crm-{day}.sqlite")
}

/// Prend la copie du jour si elle n'existe pas déjà, puis efface les plus
/// anciennes.
///
/// Renvoie le chemin écrit, ou `None` si la copie du jour existait déjà.
/// **Un échec ne bloque pas le démarrage** : l'appelant se contente de le
/// signaler. Perdre une copie est ennuyeux ; ne pas pouvoir ouvrir
/// l'application le serait davantage.
pub async fn keep_daily_copy(
    pool: &SqlitePool,
    data_dir: &Path,
    day: &str,
) -> Result<Option<PathBuf>, DbError> {
    let directory = directory(data_dir);
    std::fs::create_dir_all(&directory)?;

    let destination = directory.join(file_name(day));
    if destination.exists() {
        return Ok(None);
    }

    // `VACUUM INTO` n'accepte pas de paramètre lié : le chemin est inséré dans
    // le texte de la requête, avec les apostrophes doublées comme SQLite
    // l'exige.
    let quoted = destination.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{quoted}'"))
        .execute(pool)
        .await?;

    prune(&directory, KEPT_COPIES)?;
    Ok(Some(destination))
}

/// Ne garde que les `keep` copies les plus récentes.
///
/// Le tri se fait sur le nom, qui porte la date en ISO : l'ordre
/// alphabétique y est l'ordre chronologique, et aucune horloge de système de
/// fichiers n'entre en jeu.
fn prune(directory: &Path, keep: usize) -> Result<(), DbError> {
    let mut copies: Vec<PathBuf> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("asgard-crm-") && name.ends_with(".sqlite"))
        })
        .collect();

    copies.sort();
    let excess = copies.len().saturating_sub(keep);
    for path in copies.into_iter().take(excess) {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("asgard-autosave-{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// Base **sur fichier** : `VACUUM INTO` n'écrit rien depuis une base en
    /// mémoire, et c'est bien une base fichier que l'application copie.
    async fn open_in(data_dir: &Path) -> SqlitePool {
        let file = data_dir.join("asgard-crm.sqlite");
        let url = format!("sqlite:{}?mode=rwc", file.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"));
        let pool = SqlitePool::connect(&url).await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn writes_one_copy_per_day() {
        let data_dir = temp_dir("daily");
        let pool = open_in(&data_dir).await;

        let path = keep_daily_copy(&pool, &data_dir, "2026-09-29")
            .await
            .unwrap()
            .expect("la première copie du jour est écrite");

        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);

        // Le même jour, la copie n'est pas refaite.
        assert!(keep_daily_copy(&pool, &data_dir, "2026-09-29").await.unwrap().is_none());
        // Le lendemain, si.
        assert!(keep_daily_copy(&pool, &data_dir, "2026-09-30").await.unwrap().is_some());
    }

    /// La copie doit être une base lisible, pas un fichier tronqué.
    #[tokio::test]
    async fn the_copy_is_a_readable_database() {
        let data_dir = temp_dir("readable");
        let pool = open_in(&data_dir).await;
        super::super::seed::seed_demo_data_if_empty(&pool).await.unwrap();

        let path = keep_daily_copy(&pool, &data_dir, "2026-09-29")
            .await
            .unwrap()
            .unwrap();

        let url = format!("sqlite:{}", path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"));
        let copy = SqlitePool::connect(&url).await.unwrap();
        let clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
            .fetch_one(&copy)
            .await
            .unwrap();

        assert!(clients > 0, "la copie contient les données");
    }

    #[test]
    fn only_the_most_recent_copies_survive() {
        let directory = temp_dir("rotation");
        for day in ["2026-09-20", "2026-09-21", "2026-09-22", "2026-09-23"] {
            std::fs::write(directory.join(file_name(day)), b"x").unwrap();
        }
        // Un fichier étranger ne doit pas être emporté par le ménage.
        std::fs::write(directory.join("notes.txt"), b"x").unwrap();

        prune(&directory, 2).unwrap();

        let mut left: Vec<String> = std::fs::read_dir(&directory)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();

        assert_eq!(
            left,
            vec![
                "asgard-crm-2026-09-22.sqlite".to_string(),
                "asgard-crm-2026-09-23.sqlite".to_string(),
                "notes.txt".to_string(),
            ]
        );
    }

    /// Un dossier de copies qui n'existe pas encore est créé, pas une erreur.
    #[tokio::test]
    async fn the_directory_is_created_on_first_use() {
        let data_dir = temp_dir("fresh");
        let pool = open_in(&data_dir).await;
        assert!(!directory(&data_dir).exists());

        keep_daily_copy(&pool, &data_dir, "2026-09-29").await.unwrap();
        assert!(directory(&data_dir).is_dir());
    }
}
