//! Paramètres — port de `src/components/SettingsTab.jsx`.
//!
//! Chaque modification est enregistrée aussitôt, comme dans l'original. Les
//! taux URSSAF et l'ACRE entrent dans les calculs : les modifier relance le
//! calcul des agrégats côté hôte.

use leptos::prelude::*;
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::widgets::{SelectField, TextArea, TextField};
use crate::settings::{self, Settings};
use crate::state::{use_app, Pending};
use crate::{actions, backup, ipc};

/// Couleur d'accentuation par défaut des PDF.
const DEFAULT_COLOR: &str = "#E5A93C";

/// Séparation entre sections, répétée en ligne dans l'original.
const SECTION: &str = "border-top: 1px solid var(--border-glass); padding-top: 1.5rem; margin-top: 1.5rem";

/// Un signal par champ, relié à l'enregistrement.
///
/// L'original appelait `saveSettings({ ...businessSettings, champ: valeur })`
/// dans chaque `onChange`. Ici un seul effet observe les champs et enregistre
/// l'ensemble — un champ oublié ne peut plus échapper à la sauvegarde.
#[derive(Clone, Copy)]
struct Fields {
    company_name: RwSignal<String>,
    contact_name: RwSignal<String>,
    email: RwSignal<String>,
    phone: RwSignal<String>,
    address: RwSignal<String>,
    siret: RwSignal<String>,
    iban: RwSignal<String>,
    payment_terms: RwSignal<String>,
    urssaf_bnc: RwSignal<String>,
    urssaf_bic: RwSignal<String>,
    urssaf_vente: RwSignal<String>,
    acre: RwSignal<bool>,
    smtp_host: RwSignal<String>,
    smtp_port: RwSignal<String>,
    smtp_user: RwSignal<String>,
    smtp_secure: RwSignal<String>,
    custom_color: RwSignal<String>,
    logo: RwSignal<String>,
    template_invoice: RwSignal<String>,
    template_estimate: RwSignal<String>,
    template_reminder: RwSignal<String>,
}

impl Fields {
    fn from(s: Settings) -> Self {
        Self {
            company_name: RwSignal::new(s.company_name),
            contact_name: RwSignal::new(s.contact_name),
            email: RwSignal::new(s.email),
            phone: RwSignal::new(s.phone),
            address: RwSignal::new(s.address),
            siret: RwSignal::new(s.siret),
            iban: RwSignal::new(s.iban),
            payment_terms: RwSignal::new(s.payment_terms_days.to_string()),
            urssaf_bnc: RwSignal::new(s.urssaf_service_bnc.to_string()),
            urssaf_bic: RwSignal::new(s.urssaf_service_bic.to_string()),
            urssaf_vente: RwSignal::new(s.urssaf_vente.to_string()),
            acre: RwSignal::new(s.acre_enabled),
            smtp_host: RwSignal::new(s.smtp_host),
            smtp_port: RwSignal::new(s.smtp_port),
            smtp_user: RwSignal::new(s.smtp_user),
            smtp_secure: RwSignal::new(s.smtp_secure),
            custom_color: RwSignal::new(s.custom_color),
            logo: RwSignal::new(s.logo_base64),
            template_invoice: RwSignal::new(s.email_template_invoice),
            template_estimate: RwSignal::new(s.email_template_estimate),
            template_reminder: RwSignal::new(s.email_template_reminder),
        }
    }

    /// Réglages courants. Un taux illisible conserve la valeur enregistrée
    /// plutôt que de passer à zéro, ce qui annulerait les cotisations en silence.
    fn read(&self, previous: &Settings) -> Settings {
        let rate = |field: RwSignal<String>, fallback: f64| {
            crate::validation::parse_amount(&field.get()).unwrap_or(fallback)
        };

        let days = crate::validation::parse_amount(&self.payment_terms.get())
            .filter(|value| (0.0..=365.0).contains(value))
            .map(|value| value as u32)
            .unwrap_or(previous.payment_terms_days);

        Settings {
            company_name: self.company_name.get(),
            contact_name: self.contact_name.get(),
            email: self.email.get(),
            phone: self.phone.get(),
            address: self.address.get(),
            siret: self.siret.get(),
            iban: self.iban.get(),
            payment_terms_days: days,
            urssaf_service_bnc: rate(self.urssaf_bnc, previous.urssaf_service_bnc),
            urssaf_service_bic: rate(self.urssaf_bic, previous.urssaf_service_bic),
            urssaf_vente: rate(self.urssaf_vente, previous.urssaf_vente),
            acre_enabled: self.acre.get(),
            smtp_host: self.smtp_host.get(),
            smtp_port: self.smtp_port.get(),
            smtp_user: self.smtp_user.get(),
            smtp_secure: self.smtp_secure.get(),
            custom_color: self.custom_color.get(),
            logo_base64: self.logo.get(),
            email_template_invoice: self.template_invoice.get(),
            email_template_estimate: self.template_estimate.get(),
            email_template_reminder: self.template_reminder.get(),
        }
    }
}

pub fn settings() -> impl IntoView {
    let app = use_app();

    // Reconstruit l'écran quand une sauvegarde remplace les réglages : les
    // signaux repartent des valeurs restaurées.
    move || {
        app.settings_epoch.track();
        screen()
    }
}

fn screen() -> impl IntoView {
    let app = use_app();
    let initial = settings::load();
    let fields = Fields::from(initial.clone());

    // Enregistre à chaque modification. Si un paramètre de calcul change, les
    // agrégats sont recalculés.
    let last = StoredValue::new(initial);
    Effect::new(move |_| {
        let previous = last.get_value();
        let current = fields.read(&previous);
        if current == previous {
            return;
        }

        let affects_totals = current.urssaf_service_bnc != previous.urssaf_service_bnc
            || current.urssaf_service_bic != previous.urssaf_service_bic
            || current.urssaf_vente != previous.urssaf_vente
            || current.acre_enabled != previous.acre_enabled;

        settings::save(&current);
        last.set_value(current);

        if affects_totals {
            leptos::task::spawn_local(app.refresh_metrics());
        }
    });

    // Entrée dans un champ : l'original confirmait l'enregistrement, déjà fait.
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        app.inform("Succès", "Paramètres enregistrés !");
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Paramètres de l'entreprise"</h1>
                    <p>"Configurez vos mentions légales, vos taux de charge URSSAF et vos coordonnées."</p>
                </div>
            </div>

            <div class="card-glass">
                <form on:submit=submit>
                    <Company fields=fields />
                    <Urssaf fields=fields />
                    <Branding fields=fields />
                    <Smtp fields=fields />
                    <Templates fields=fields />
                    <Backup />
                </form>
            </div>
        </div>
    }
}

#[component]
fn Company(fields: Fields) -> impl IntoView {
    view! {
        <div class="settings-section">
            <div class="settings-section-title">"Coordonnées de l'émetteur"</div>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem">
                <TextField label="Nom de l'entreprise" value=fields.company_name />
                <TextField label="Nom du contact" value=fields.contact_name />
                <TextField label="Email professionnel" value=fields.email kind="email" />
                <TextField label="Numéro de téléphone" value=fields.phone />
                <div style="grid-column: span 2">
                    <TextField label="Adresse professionnelle" value=fields.address />
                </div>
                <TextField label="SIRET" value=fields.siret />
                <TextField label="IBAN bancaire (Règlement)" value=fields.iban />
                <FieldWithHint
                    label="Délai de règlement (jours)"
                    value=fields.payment_terms
                    hint="Échéance imprimée sur les factures. 30 jours par défaut."
                />
            </div>
        </div>
    }
}

/// Champ numérique accompagné d'un repère — un taux, un délai.
///
/// Champ texte plutôt que numérique : une virgule décimale y est acceptée,
/// et un `type="number"` sans `step` bloquerait l'envoi du formulaire sur
/// « 12.3 ».
#[component]
fn FieldWithHint(label: &'static str, value: RwSignal<String>, hint: &'static str) -> impl IntoView {
    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <input
                type="text"
                inputmode="decimal"
                class="form-input"
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            />
            <span class="metric-subtext">{hint}</span>
        </div>
    }
}

#[component]
fn Urssaf(fields: Fields) -> impl IntoView {
    view! {
        <div class="settings-section" style="border-bottom: none; padding-bottom: 0">
            <div class="settings-section-title">"Cotisations sociales (URSSAF)"</div>
            <div class="form-group" style="margin-bottom: 1.5rem; display: flex; align-items: center; gap: 0.75rem">
                <input
                    type="checkbox"
                    id="acreCheckbox"
                    style="transform: scale(1.25); accent-color: var(--color-gold); cursor: pointer"
                    prop:checked=move || fields.acre.get()
                    on:change=move |ev| fields.acre.set(event_target_checked(&ev))
                />
                <label for="acreCheckbox" style="font-weight: 600; font-size: 0.9rem; cursor: pointer">
                    "Bénéficiaire de l'ACRE (Taux de cotisations réduits de 50%)"
                </label>
            </div>
            <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 1.25rem">
                <FieldWithHint
                    label="Taux Service BNC (%)"
                    value=fields.urssaf_bnc
                    hint="Par défaut 21.1% (Libérale BNC)"
                />
                <FieldWithHint
                    label="Taux Service BIC (%)"
                    value=fields.urssaf_bic
                    hint="Par défaut 21.1% (Artisanal/Comm. BIC)"
                />
                <FieldWithHint
                    label="Taux Vente BIC (%)"
                    value=fields.urssaf_vente
                    hint="Par défaut 12.3% (Vente marchandises)"
                />
            </div>
        </div>
    }
}

#[component]
fn Branding(fields: Fields) -> impl IntoView {
    let on_logo = move |ev: leptos::ev::Event| {
        let Some(file) = first_file(&ev) else { return };
        read_as_data_url(file, move |url| fields.logo.set(url));
        clear_file_input(&ev);
    };

    // L'original affichait la couleur par défaut tant qu'aucune n'était choisie.
    let color = move || {
        let chosen = fields.custom_color.get();
        if chosen.is_empty() { DEFAULT_COLOR.to_string() } else { chosen }
    };

    let small_button = "padding: 0.5rem 1rem; font-size: 0.8rem";

    view! {
        <div class="settings-section" style=SECTION>
            <div class="settings-section-title">"Identité Visuelle & PDF"</div>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem">
                <div class="form-group">
                    <label class="form-label">"Couleur d'accentuation des PDF"</label>
                    <div style="display: flex; gap: 0.75rem; align-items: center">
                        <input
                            type="color"
                            class="form-input"
                            style="width: 50px; height: 40px; padding: 2px; cursor: pointer; border: 1px solid var(--border-glass)"
                            prop:value=color
                            on:input=move |ev| fields.custom_color.set(event_target_value(&ev))
                        />
                        <input
                            type="text"
                            class="form-input"
                            style="font-family: monospace"
                            prop:value=color
                            on:input=move |ev| fields.custom_color.set(event_target_value(&ev))
                        />
                        <button
                            type="button"
                            class="btn btn-secondary"
                            style=small_button
                            on:click=move |_| fields.custom_color.set(DEFAULT_COLOR.into())
                        >
                            "Réinitialiser"
                        </button>
                    </div>
                    <span class="metric-subtext">
                        "Couleur utilisée pour les titres et totaux sur les devis/factures exportés."
                    </span>
                </div>

                <div class="form-group">
                    <label class="form-label">"Logo de l'entreprise (Format image)"</label>
                    <div style="display: flex; gap: 0.75rem; align-items: center">
                        {move || {
                            let logo = fields.logo.get();
                            if logo.is_empty() {
                                view! {
                                    <div style="border: 1px dashed var(--border-glass); border-radius: 4px; height: 40px; width: 60px; display: flex; align-items: center; justify-content: center; font-size: 12px; color: var(--text-muted)">
                                        "Aucun"
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div style="position: relative; border: 1px solid var(--border-glass); border-radius: 4px; padding: 4px; background: rgba(255,255,255,0.05); display: flex; align-items: center; justify-content: center; height: 40px; width: 60px">
                                        <img src=logo alt="Logo" style="max-height: 100%; max-width: 100%; object-fit: contain" />
                                        <button
                                            type="button"
                                            style="position: absolute; top: -5px; right: -5px; background: #EF4444; color: white; border: none; border-radius: 50%; width: 16px; height: 16px; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 10px; font-weight: bold"
                                            title="Supprimer le logo"
                                            on:click=move |_| fields.logo.set(String::new())
                                        >
                                            "×"
                                        </button>
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                        <input
                            type="file"
                            accept="image/*"
                            style="display: none"
                            id="logoUploadInput"
                            on:change=on_logo
                        />
                        <label
                            for="logoUploadInput"
                            class="btn btn-secondary"
                            style="padding: 0.5rem 1rem; font-size: 0.8rem; cursor: pointer; margin: 0"
                        >
                            "Choisir un logo"
                        </label>
                    </div>
                    <span class="metric-subtext">
                        "Recommandé : PNG transparent, format paysage (hauteur max 60px)."
                    </span>
                    // Régression connue depuis la phase 4 : le générateur Rust
                    // n'imprime pas encore le logo. Mieux vaut le dire que
                    // laisser croire qu'il figure sur les factures.
                    <span class="metric-subtext" style="display: block; margin-top: 0.4rem">
                        "⚠️ Le logo n'apparaît pas encore sur les PDF depuis le passage du générateur en Rust."
                    </span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Smtp(fields: Fields) -> impl IntoView {
    let app = use_app();
    let draft = RwSignal::new(String::new());
    let stored = RwSignal::new(false);
    let testing = RwSignal::new(false);

    // L'interface ne relit jamais le mot de passe : elle sait seulement s'il
    // existe dans le trousseau.
    leptos::task::spawn_local(async move {
        if let Ok(present) = ipc::call::<bool>("has_smtp_password").await {
            stored.set(present);
        }
    });

    let save_password = move |_| {
        let password = draft.get();
        leptos::task::spawn_local(async move {
            #[derive(Serialize)]
            struct Args {
                password: String,
            }
            let clearing = password.is_empty();
            match ipc::invoke::<_, ()>("set_smtp_password", &Args { password }).await {
                Ok(()) => {
                    stored.set(!clearing);
                    draft.set(String::new());
                    app.inform(
                        "Enregistré",
                        if clearing {
                            "Le mot de passe SMTP a été supprimé du trousseau."
                        } else {
                            "Le mot de passe SMTP a été placé dans le trousseau de votre système. \
                             Il ne figure plus dans les réglages ni dans les sauvegardes."
                        },
                    );
                }
                Err(error) => app.report(error),
            }
        });
    };

    let test = move |_| {
        testing.set(true);
        let config = actions::SmtpConfig::from_settings(&settings::load(), Some(draft.get()));
        leptos::task::spawn_local(async move {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Args {
                smtp_config: actions::SmtpConfig,
            }
            let outcome = ipc::invoke::<_, actions::SendOutcome>("test_smtp", &Args { smtp_config: config }).await;
            testing.set(false);
            match outcome {
                Ok(o) if o.success => app.inform(
                    "Connexion réussie",
                    "La configuration SMTP est correcte ! Le serveur a validé les identifiants.",
                ),
                Ok(o) => app.inform(
                    "Échec de la connexion",
                    format!("Erreur de connexion SMTP : {}", o.error.unwrap_or_default()),
                ),
                Err(error) => app.report_as("Impossible de tester la connexion", error),
            }
        });
    };

    let securities = Signal::derive(|| {
        vec![
            ("none".into(), "Aucune (STARTTLS automatique / Proton Mail Bridge)".into()),
            ("ssl".into(), "SSL Strict (Port 465)".into()),
        ]
    });

    view! {
        <div class="settings-section" style=SECTION>
            <div class="flex-between" style="margin-bottom: 1.25rem">
                <div class="settings-section-title" style="margin-bottom: 0">"Configuration de messagerie (SMTP)"</div>
                <button type="button" class="btn btn-secondary" on:click=test disabled=move || testing.get()>
                    {move || if testing.get() { "Vérification en cours..." } else { "Tester la connexion SMTP" }}
                </button>
            </div>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem; margin-bottom: 1rem">
                <TextField
                    label="Hôte SMTP"
                    value=fields.smtp_host
                    placeholder="ex: 127.0.0.1 (Proton Mail Bridge)"
                />
                <TextField label="Port SMTP" value=fields.smtp_port placeholder="ex: 1025" />
                <TextField
                    label="Utilisateur SMTP / Adresse Mail"
                    value=fields.smtp_user
                    placeholder="Votre adresse e-mail"
                />

                <div class="form-group">
                    <label class="form-label">"Mot de passe SMTP"</label>
                    <div style="display: flex; gap: 0.5rem">
                        <input
                            id="smtp-password"
                            type="password"
                            class="form-input"
                            autocomplete="off"
                            placeholder=move || if stored.get() { "Enregistré — saisir pour remplacer" } else { "Mot de passe ou clé générée" }
                            prop:value=move || draft.get()
                            on:input=move |ev| draft.set(event_target_value(&ev))
                        />
                        <button
                            type="button"
                            class="btn btn-secondary"
                            style="white-space: nowrap"
                            disabled=move || draft.get().is_empty() && !stored.get()
                            on:click=save_password
                        >
                            {move || if draft.get().is_empty() && stored.get() { "Effacer" } else { "Enregistrer" }}
                        </button>
                    </div>
                    <span class="metric-subtext" style="display: block; margin-top: 0.4rem; line-height: 1.4">
                        {move || if stored.get() {
                            "🔐 Conservé dans le trousseau de votre système. Il ne figure ni dans les réglages, ni dans les sauvegardes."
                        } else {
                            "🔐 Sera conservé dans le trousseau de votre système, et non dans les réglages."
                        }}
                    </span>
                </div>

                <SelectField label="Sécurité Connexion" value=fields.smtp_secure options=securities />
            </div>
            <span class="metric-subtext" style="display: block; color: var(--color-gold); line-height: 1.5">
                "💡 Pour Proton Mail Bridge, laissez l'Hôte sur " <strong>"127.0.0.1"</strong>
                ", le port sur celui indiqué par votre application Bridge (souvent 1025), et choisissez \"Aucune\" pour la sécurité."
            </span>
        </div>
    }
}

/// Balises reconnues par les modèles, telles que l'original les présentait.
const TAGS: [(&str, &str); 8] = [
    ("{clientName}", "Nom du client"),
    ("{documentNumber}", "N° Facture / Devis"),
    ("{amountTotal}", "Montant TTC (€)"),
    ("{dueDate}", "Date d'échéance (factures)"),
    ("{documentDate}", "Date du document"),
    ("{description}", "Description du document"),
    ("{senderName}", "Votre nom"),
    ("{senderCompany}", "Votre entreprise"),
];

#[component]
fn Templates(fields: Fields) -> impl IntoView {
    view! {
        <div class="settings-section" style=SECTION>
            <div class="settings-section-title">"Modèles d'e-mails"</div>
            <p class="metric-subtext" style="margin-bottom: 1.25rem; line-height: 1.5">
                "Personnalisez les messages d'accompagnement envoyés par e-mail avec vos factures, devis et relances."
            </p>

            <div style="display: grid; grid-template-columns: 1fr; gap: 1.25rem; margin-bottom: 1.25rem">
                <TextArea label="Modèle : Envoi de Facture" value=fields.template_invoice min_height="120px" />
                <TextArea label="Modèle : Envoi de Devis" value=fields.template_estimate min_height="120px" />
                <TextArea
                    label="Modèle : Relance Facture Impayée"
                    value=fields.template_reminder
                    min_height="120px"
                />
            </div>

            <div style="padding: 1rem; background-color: rgba(229,169,60,0.03); border-radius: 8px; border: 1px dashed rgba(229,169,60,0.2); font-size: 0.8rem; line-height: 1.6">
                "💡 " <strong>"Balises utilisables :"</strong>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 0.5rem; margin-top: 0.5rem; font-family: monospace">
                    {TAGS
                        .into_iter()
                        .map(|(tag, meaning)| view! { <div>{format!("{tag} : {meaning}")}</div> })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn Backup() -> impl IntoView {
    let app = use_app();

    let export = move |_| leptos::task::spawn_local(backup::export(app));

    // Le fichier est lu et reconnu d'abord, puis la restauration — qui écrase
    // tout — attend une confirmation, comme le `window.confirm` d'origine.
    let import = move |ev: leptos::ev::Event| {
        let Some(file) = first_file(&ev) else { return };
        clear_file_input(&ev);
        leptos::task::spawn_local(async move {
            let text = match JsFuture::from(file.text()).await {
                Ok(text) => text.as_string().unwrap_or_default(),
                Err(error) => {
                    return app.inform(
                        "Erreur",
                        format!("Impossible de lire le fichier de sauvegarde : {error:?}"),
                    );
                }
            };
            match backup::parse(&text) {
                Ok(content) => app.pending.set(Some(Pending::ImportBackup(content))),
                Err(message) => app.inform("Erreur", message),
            }
        });
    };

    view! {
        <div class="settings-section" style=SECTION>
            <div class="settings-section-title">"Sécurité & Sauvegarde des données"</div>
            <p class="metric-subtext" style="margin-bottom: 1.25rem; line-height: 1.5">
                "Vos données sont stockées localement dans votre base de données locale. Exportez régulièrement des sauvegardes pour éviter toute perte de données en cas de panne de votre ordinateur."
            </p>
            <div style="display: flex; gap: 1rem">
                <button type="button" class="btn btn-primary" on:click=export>
                    "Exporter les données (.json)"
                </button>
                <input
                    type="file"
                    accept=".json"
                    id="dbBackupImportInput"
                    style="display: none"
                    on:change=import
                />
                <label for="dbBackupImportInput" class="btn btn-secondary" style="cursor: pointer; margin: 0">
                    "Restaurer une sauvegarde"
                </label>
            </div>
        </div>
    }
}

/// Premier fichier choisi dans un `<input type="file">`.
fn first_file(ev: &leptos::ev::Event) -> Option<web_sys::File> {
    let input: web_sys::HtmlInputElement = ev.target()?.dyn_into().ok()?;
    input.files()?.get(0)
}

/// Vide le champ : choisir à nouveau le même fichier doit relancer la lecture,
/// comme `e.target.value = null` dans l'original.
fn clear_file_input(ev: &leptos::ev::Event) {
    if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
        input.set_value("");
    }
}

/// Lit un fichier image en URL de données, pour l'aperçu et le stockage.
fn read_as_data_url(file: web_sys::File, done: impl Fn(String) + 'static) {
    let Ok(reader) = web_sys::FileReader::new() else { return };
    let reader_for_load = reader.clone();

    let on_load = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        if let Ok(result) = reader_for_load.result() {
            if let Some(url) = result.as_string() {
                done(url);
            }
        }
    });

    reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
    // La fermeture doit survivre à cette fonction : elle est rappelée plus tard,
    // à la fin de la lecture.
    on_load.forget();
    let _ = reader.read_as_data_url(&file);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un taux illisible ne doit pas remettre les cotisations à zéro en
    /// silence : la valeur précédente est conservée.
    #[test]
    fn an_unreadable_rate_keeps_the_previous_value() {
        let owner = leptos::prelude::Owner::new();
        owner.with(|| {
            let fields = Fields::from(Settings::default());
            fields.urssaf_bnc.set("pas un nombre".into());
            let read = fields.read(&Settings::default());
            assert_eq!(read.urssaf_service_bnc, 21.1);
        });
    }

    /// Une échéance figure sur un document qui ne se corrige que par avoir :
    /// une saisie illisible ne doit pas la déplacer en silence.
    #[test]
    fn an_unreadable_payment_delay_keeps_the_previous_value() {
        let owner = leptos::prelude::Owner::new();
        owner.with(|| {
            let fields = Fields::from(Settings::default());

            fields.payment_terms.set("quarante".into());
            assert_eq!(fields.read(&Settings::default()).payment_terms_days, 30);

            fields.payment_terms.set("-5".into());
            assert_eq!(fields.read(&Settings::default()).payment_terms_days, 30);

            fields.payment_terms.set("45".into());
            assert_eq!(fields.read(&Settings::default()).payment_terms_days, 45);
        });
    }

    #[test]
    fn a_comma_rate_is_accepted() {
        let owner = leptos::prelude::Owner::new();
        owner.with(|| {
            let fields = Fields::from(Settings::default());
            fields.urssaf_vente.set("12,8".into());
            assert_eq!(fields.read(&Settings::default()).urssaf_vente, 12.8);
        });
    }

    /// Les balises annoncées doivent être celles que le moteur de modèles
    /// remplace : une balise documentée mais inconnue resterait telle quelle
    /// dans l'e-mail envoyé.
    #[test]
    fn documented_tags_are_the_ones_templates_resolve() {
        let data = crate::templates::TemplateData {
            client_name: "a".into(),
            document_number: "b".into(),
            description: "c".into(),
            amount_total: "d".into(),
            due_date: "e".into(),
            document_date: "f".into(),
            sender_name: "g".into(),
            sender_company: "h".into(),
        };
        for (tag, _) in TAGS {
            let resolved = crate::templates::resolve(tag, &data);
            assert_ne!(resolved, tag, "{tag} n'est pas remplacée");
        }
    }
}
