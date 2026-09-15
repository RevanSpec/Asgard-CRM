//! Numérotation des pièces — correction du défaut D3.
//!
//! L'ancienne implémentation (`db.js`) dérivait la séquence du *nombre* de
//! pièces de l'exercice : `count + 1`. Supprimer une facture faisait réutiliser
//! un numéro déjà émis, et deux créations simultanées produisaient le même.
//! L'article 242 nonies A du CGI impose une séquence chronologique continue et
//! sans rupture.
//!
//! Ici, le prochain numéro est lu **et** incrémenté dans la transaction qui
//! insère la pièce. Deux appels concurrents sont sérialisés par la transaction,
//! et la contrainte `UNIQUE` sur la colonne du numéro sert de dernier rempart.
//!
//! Les golden tests de la phase 0 (`src/domain/numbering.test.js`) décrivaient
//! l'ancien comportement, illégal, pour qu'on puisse prouver qu'il existait. Les
//! tests de ce module décrivent le nouveau. C'est le moment prévu par le plan
//! pour que les premiers soient réécrits plutôt que supprimés.

use sqlx::{Sqlite, Transaction};

/// Longueur du fragment client dans un numéro de pièce.
const CLIENT_FRAGMENT_LENGTH: usize = 10;

/// Nombre de chiffres de la séquence.
const SEQUENCE_PADDING: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Invoice,
    Estimate,
}

impl DocumentKind {
    pub fn prefix(self) -> &'static str {
        match self {
            DocumentKind::Invoice => "FAC",
            DocumentKind::Estimate => "DEV",
        }
    }

    /// Clé utilisée dans `document_sequences`.
    pub fn key(self) -> &'static str {
        match self {
            DocumentKind::Invoice => "invoice",
            DocumentKind::Estimate => "estimate",
        }
    }
}

/// Normalise une raison sociale : majuscules, accents retirés sans perdre la
/// lettre, caractères non alphanumériques supprimés, tronqué à dix caractères.
///
/// Reproduit la fonction JavaScript à l'identique — les numéros déjà émis
/// doivent rester reproductibles, sans quoi la reprise casserait la séquence.
pub fn sanitize_client_name(company_name: &str) -> String {
    company_name
        .chars()
        .flat_map(|c| c.to_uppercase())
        .filter_map(strip_diacritic)
        .filter(|c| c.is_ascii_alphanumeric())
        .take(CLIENT_FRAGMENT_LENGTH)
        .collect()
}

/// Remplace les lettres latines accentuées par leur base ASCII.
///
/// Équivalent du `normalize("NFD").replace(/[̀-ͯ]/g, "")` du
/// JavaScript, limité aux lettres majuscules puisque l'appelant a déjà mis en
/// majuscules. Les caractères non gérés ressortent tels quels et seront écartés
/// par le filtre alphanumérique.
fn strip_diacritic(c: char) -> Option<char> {
    Some(match c {
        'À'..='Å' => 'A',
        'Ç' => 'C',
        'È'..='Ë' => 'E',
        'Ì'..='Ï' => 'I',
        'Ñ' => 'N',
        'Ò'..='Ö' | 'Ø' => 'O',
        'Ù'..='Ü' => 'U',
        'Ý' => 'Y',
        // Diacritiques combinants laissés par une décomposition : à écarter.
        '\u{0300}'..='\u{036f}' => return None,
        other => other,
    })
}

pub fn format_number(kind: DocumentKind, company_name: &str, year: i32, sequence: i64) -> String {
    format!(
        "{}-{}-{}-{:0width$}",
        kind.prefix(),
        sanitize_client_name(company_name),
        year,
        sequence,
        width = SEQUENCE_PADDING
    )
}

/// Réserve le prochain numéro de l'exercice et l'incrémente.
///
/// À appeler dans la transaction qui insère la pièce : c'est ce qui rend
/// l'attribution atomique. `INSERT ... ON CONFLICT DO UPDATE ... RETURNING`
/// fait la lecture et l'écriture en une seule instruction, donc sans fenêtre
/// entre les deux.
pub async fn allocate(
    tx: &mut Transaction<'_, Sqlite>,
    kind: DocumentKind,
    year: i32,
) -> Result<i64, sqlx::Error> {
    let next: i64 = sqlx::query_scalar(
        "INSERT INTO document_sequences (kind, year, next_value)
         VALUES (?1, ?2, 2)
         ON CONFLICT (kind, year)
         DO UPDATE SET next_value = next_value + 1
         RETURNING next_value",
    )
    .bind(kind.key())
    .bind(year)
    .fetch_one(&mut **tx)
    .await?;

    // `RETURNING` rend la valeur après écriture : à la création de la ligne il
    // renvoie 2 alors que le numéro attribué est 1, et à chaque mise à jour il
    // renvoie déjà l'incrément suivant.
    Ok(next - 1)
}

/// Positionne la séquence sur un exercice à partir des numéros déjà émis.
///
/// ⚠️ Le point de vigilance de la phase 2. La valeur se déduit du **plus grand
/// numéro existant**, jamais du nombre de lignes reprises : une base dont on a
/// supprimé des pièces a moins de lignes que de numéros émis, et repartir du
/// compte réattribuerait des numéros déjà utilisés — exactement D3.
pub async fn seed_sequence_from_existing(
    tx: &mut Transaction<'_, Sqlite>,
    kind: DocumentKind,
    year: i32,
    highest_issued: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO document_sequences (kind, year, next_value)
         VALUES (?1, ?2, ?3)
         ON CONFLICT (kind, year)
         DO UPDATE SET next_value = MAX(next_value, ?3)",
    )
    .bind(kind.key())
    .bind(year)
    .bind(highest_issued + 1)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Extrait la séquence d'un numéro de pièce, ou `None` s'il ne suit pas le
/// format. Une base reprise peut contenir des numéros saisis à la main.
pub fn sequence_of(number: &str) -> Option<i64> {
    number.rsplit('-').next()?.parse().ok()
}

/// Extrait l'exercice d'un numéro de pièce.
pub fn year_of(number: &str) -> Option<i32> {
    let parts: Vec<&str> = number.split('-').collect();
    if parts.len() < 2 {
        return None;
    }
    parts[parts.len() - 2].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_like_the_javascript_implementation() {
        assert_eq!(sanitize_client_name("Stark Industries"), "STARKINDUS");
        assert_eq!(sanitize_client_name("Wayne Enterprises"), "WAYNEENTER");
        assert_eq!(sanitize_client_name("Asgard Coffee & Co"), "ASGARDCOFF");
        assert_eq!(sanitize_client_name("Éditions Yggdrasil"), "EDITIONSYG");
        assert_eq!(sanitize_client_name("Crème Brûlée"), "CREMEBRULE");
        assert_eq!(sanitize_client_name("L'Atelier d'Odin"), "LATELIERDO");
        assert_eq!(sanitize_client_name("Studio 54"), "STUDIO54");
        assert_eq!(sanitize_client_name("*** ???"), "");
    }

    #[test]
    fn formats_numbers_like_the_javascript_implementation() {
        assert_eq!(
            format_number(DocumentKind::Invoice, "Stark Industries", 2026, 1),
            "FAC-STARKINDUS-2026-0001"
        );
        assert_eq!(
            format_number(DocumentKind::Estimate, "Asgard Coffee & Co", 2026, 4),
            "DEV-ASGARDCOFF-2026-0004"
        );
        assert_eq!(
            format_number(DocumentKind::Invoice, "Stark Industries", 2026, 12_345),
            "FAC-STARKINDUS-2026-12345"
        );
    }

    #[test]
    fn parses_sequence_and_year_back_out() {
        assert_eq!(sequence_of("FAC-STARKINDUS-2026-0007"), Some(7));
        assert_eq!(year_of("FAC-STARKINDUS-2026-0007"), Some(2026));
        assert_eq!(sequence_of("numero maison"), None);
        assert_eq!(year_of("FAC"), None);
    }

    async fn fresh_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn allocates_a_continuous_sequence() {
        let pool = fresh_pool().await;
        let mut tx = pool.begin().await.unwrap();

        for expected in 1..=5 {
            assert_eq!(
                allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap(),
                expected
            );
        }

        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn keeps_exercises_and_kinds_independent() {
        let pool = fresh_pool().await;
        let mut tx = pool.begin().await.unwrap();

        assert_eq!(allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap(), 1);
        assert_eq!(allocate(&mut tx, DocumentKind::Invoice, 2025).await.unwrap(), 1);
        assert_eq!(allocate(&mut tx, DocumentKind::Estimate, 2026).await.unwrap(), 1);
        assert_eq!(allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap(), 2);

        tx.commit().await.unwrap();
    }

    /// Le comportement que D3 rendait impossible : après suppression d'une
    /// pièce, le numéro suivant ne revient pas en arrière.
    #[tokio::test]
    async fn never_reissues_a_number_after_a_deletion() {
        let pool = fresh_pool().await;
        let mut tx = pool.begin().await.unwrap();

        let first = allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap();
        let second = allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap();
        // La pièce `second` est supprimée — la séquence n'en sait rien et ne
        // doit surtout pas le savoir.
        let third = allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap();

        assert_eq!((first, second, third), (1, 2, 3));
        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn seeds_from_the_highest_issued_number_not_the_row_count() {
        let pool = fresh_pool().await;
        let mut tx = pool.begin().await.unwrap();

        // Base reprise : deux pièces subsistent mais la plus haute porte le
        // numéro 7. Repartir du nombre de lignes donnerait 3 et réattribuerait
        // des numéros déjà émis.
        seed_sequence_from_existing(&mut tx, DocumentKind::Invoice, 2026, 7)
            .await
            .unwrap();

        assert_eq!(allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap(), 8);
        tx.commit().await.unwrap();
    }

    #[tokio::test]
    async fn seeding_never_moves_a_sequence_backwards() {
        let pool = fresh_pool().await;
        let mut tx = pool.begin().await.unwrap();

        seed_sequence_from_existing(&mut tx, DocumentKind::Invoice, 2026, 10).await.unwrap();
        seed_sequence_from_existing(&mut tx, DocumentKind::Invoice, 2026, 3).await.unwrap();

        assert_eq!(allocate(&mut tx, DocumentKind::Invoice, 2026).await.unwrap(), 11);
        tx.commit().await.unwrap();
    }

    /// La contrainte d'unicité est le dernier rempart : même si une séquence
    /// était corrompue, la base refuserait le doublon.
    #[tokio::test]
    async fn the_database_refuses_a_duplicate_number() {
        let pool = fresh_pool().await;

        for _ in 0..2 {
            let result = sqlx::query(
                "INSERT INTO invoices (company_name, invoice_number, service_type,
                    amount_ht_cents, amount_tva_cents, amount_total_cents, date)
                 VALUES ('Stark', 'FAC-STARKINDUS-2026-0001', 'service_bnc', 100, 20, 120, '2026-01-01')",
            )
            .execute(&pool)
            .await;

            if result.is_err() {
                return; // le second insert a bien été refusé
            }
        }

        panic!("la base a accepté deux fois le même numéro de facture");
    }
}
