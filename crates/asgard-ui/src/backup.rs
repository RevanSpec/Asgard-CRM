//! Sauvegarde et restauration — `handleExportBackup` et `handleImportBackup`
//! de `App.jsx`.

use leptos::prelude::Update;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::actions;
use crate::ipc;
use crate::settings::{self, Settings};
use crate::state::App;

/// Exporte les tables et les réglages dans un fichier choisi par l'utilisateur.
pub async fn export(app: App) {
    const FAILED: &str = "Échec de l'exportation de la sauvegarde";

    let tables = match ipc::call::<Value>("export_backup").await {
        Ok(tables) => tables,
        Err(error) => return app.report_as(FAILED, error),
    };

    // Les réglages accompagnent les tables, sans le mot de passe SMTP : il vit
    // dans le trousseau et n'a aucune raison de voyager.
    let backup = serde_json::json!({
        "db": tables,
        "settings": settings::load(),
        "backupVersion": 2,
    });

    let date = crate::views::modals::today_iso();
    let contents = serde_json::to_string_pretty(&backup).unwrap_or_default();
    match actions::export_text(contents, format!("asgard_crm_backup_{date}.json"), "json").await {
        Ok(Some(path)) => app.inform("Succès", format!("Sauvegarde enregistrée : {path}")),
        Ok(None) => {}
        Err(error) => app.report_as(FAILED, error),
    }
}

/// Préfixe des erreurs de lecture, repris de l'original.
const UNREADABLE: &str = "Erreur lors du traitement du fichier de sauvegarde";

/// Lit un fichier de sauvegarde et vérifie qu'il en est bien une.
///
/// Fait **avant** la confirmation : inutile de demander s'il faut écraser les
/// données avec un fichier qui ne pourra pas être repris.
pub fn parse(text: &str) -> Result<Value, String> {
    let backup: Value = serde_json::from_str(text).map_err(|error| format!("{UNREADABLE} : {error}"))?;

    // `!backup.db && !backup.clients` : une clé nulle compte comme absente.
    let present = |key| backup.get(key).is_some_and(|v| !v.is_null());
    if !present("db") && !present("clients") {
        return Err(format!(
            "{UNREADABLE} : Le fichier importé n'est pas une sauvegarde Asgard CRM valide."
        ));
    }
    Ok(backup)
}

/// Remplace les données par celles de la sauvegarde, une fois confirmé.
pub async fn import(app: App, backup: Value) {
    #[derive(Serialize)]
    struct Args<'a> {
        backup: &'a Value,
    }

    match ipc::invoke::<_, ImportReport>("import_backup", &Args { backup: &backup }).await {
        Ok(report) => {
            // Réglages restaurés, mot de passe écarté : une sauvegarde d'avant
            // la phase 1 le contient en clair.
            if let Some(saved) = backup.get("settings") {
                if let Ok(restored) = serde_json::from_value::<Settings>(saved.clone()) {
                    settings::save(&restored);
                }
            }
            settings::strip_legacy_password();
            app.settings_epoch.update(|n| *n += 1);

            app.reload().await;
            app.inform("Sauvegarde restaurée", describe_import(&report));
        }
        Err(error) => app.report_as(UNREADABLE, error),
    }
}

/// Compte rendu d'une reprise de sauvegarde, tel que le renvoie l'hôte.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportReport {
    clients: usize,
    invoices: usize,
    estimates: usize,
    expenses: usize,
    adjustments: Vec<Adjustment>,
    skipped: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Adjustment {
    document: String,
    field: String,
    before: f64,
    after: f64,
}

/// Met en mots le compte rendu d'une reprise.
///
/// La partie qui compte est `adjustments` : le passage aux centimes change
/// réellement certains montants, et l'utilisateur doit pouvoir l'expliquer
/// plutôt que de le découvrir dans une déclaration.
fn describe_import(report: &ImportReport) -> String {
    let mut parts = vec![format!(
        "{} client(s), {} facture(s), {} devis et {} dépense(s) repris.",
        report.clients, report.invoices, report.estimates, report.expenses
    )];

    if !report.adjustments.is_empty() {
        let shown: Vec<String> = report
            .adjustments
            .iter()
            .take(5)
            .map(|a| format!("• {} — {} : {} → {} €", a.document, a.field, a.before, a.after))
            .collect();
        let rest = report.adjustments.len().saturating_sub(5);

        parts.push(format!(
            "\n{} montant(s) ajusté(s) au centime. Vos données étaient stockées en virgule \
             flottante ; elles le sont désormais en centimes, ce qui supprime les écarts \
             d'arrondi :\n{}{}",
            report.adjustments.len(),
            shown.join("\n"),
            if rest > 0 { format!("\n… et {rest} autre(s).") } else { String::new() }
        ));
    }

    if !report.skipped.is_empty() {
        parts.push(format!(
            "\n{} pièce(s) écartée(s) :\n• {}",
            report.skipped.len(),
            report.skipped.iter().take(5).cloned().collect::<Vec<_>>().join("\n• ")
        ));
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(adjustments: usize, skipped: usize) -> ImportReport {
        ImportReport {
            clients: 3,
            invoices: 7,
            estimates: 2,
            expenses: 3,
            adjustments: (0..adjustments)
                .map(|i| Adjustment {
                    document: format!("FAC-{i}"),
                    field: "amountTva".into(),
                    before: 379.998,
                    after: 380.0,
                })
                .collect(),
            skipped: (0..skipped).map(|i| format!("FAC-{i} — numéro déjà présent")).collect(),
        }
    }

    #[test]
    fn a_clean_import_reports_counts_only() {
        let text = describe_import(&report(0, 0));
        assert!(text.starts_with("3 client(s), 7 facture(s), 2 devis et 3 dépense(s) repris."));
        assert!(!text.contains("ajusté"));
    }

    /// Le cœur du rapport : les montants qui ont bougé sont listés, pièce par
    /// pièce, avec l'avant et l'après.
    #[test]
    fn adjusted_amounts_are_listed_with_before_and_after() {
        let text = describe_import(&report(1, 0));
        assert!(text.contains("1 montant(s) ajusté(s) au centime"));
        assert!(text.contains("FAC-0 — amountTva : 379.998 → 380 €"));
        assert!(!text.contains("  "), "{text}");
    }

    #[test]
    fn long_adjustment_lists_are_truncated() {
        let text = describe_import(&report(8, 0));
        assert!(text.contains("… et 3 autre(s)."));
        assert!(!text.contains("FAC-7"));
    }

    #[test]
    fn skipped_documents_are_reported() {
        let text = describe_import(&report(0, 2));
        assert!(text.contains("2 pièce(s) écartée(s)"));
    }

    #[test]
    fn current_and_legacy_backups_are_recognised() {
        assert!(parse(r#"{"db": {"clients": []}, "backupVersion": 2}"#).is_ok());
        // Format d'avant `db` : les tables à la racine.
        assert!(parse(r#"{"clients": []}"#).is_ok());
    }

    /// Rien n'est demandé à l'utilisateur pour un fichier qui ne pourra pas
    /// être repris : l'erreur tombe avant la confirmation.
    #[test]
    fn other_files_are_refused_before_any_confirmation() {
        for text in ["pas du json", "[]", "42", r#"{"db": null}"#, r#"{"autre": 1}"#] {
            let error = parse(text).unwrap_err();
            assert!(error.starts_with(UNREADABLE), "{text} → {error}");
        }
        assert!(parse(r#"{"autre": 1}"#).unwrap_err().ends_with("sauvegarde Asgard CRM valide."));
    }
}
