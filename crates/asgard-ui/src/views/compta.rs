//! Comptabilité — port de `src/components/ComptaTab.jsx`.
//!
//! Trois sous-onglets : livre des recettes, déclaration URSSAF, seuils. Tous
//! les chiffres viennent d'`asgard-core` via l'hôte ; cette vue les met en forme.

use asgard_core::thresholds::Gauge;
use asgard_core::urssaf::DeclarationLine;
use leptos::prelude::*;

use crate::state::{use_app, Period};
use crate::{actions, format, ipc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sub {
    Recettes,
    Urssaf,
    Seuils,
}

const MONTHS: [&str; 12] = [
    "Janvier", "Février", "Mars", "Avril", "Mai", "Juin", "Juillet", "Août", "Septembre",
    "Octobre", "Novembre", "Décembre",
];

const QUARTERS: [&str; 4] = [
    "T1 (Jan - Fév - Mar)",
    "T2 (Avr - Mai - Jun)",
    "T3 (Jul - Aoû - Sep)",
    "T4 (Oct - Nov - Déc)",
];

pub fn compta() -> impl IntoView {
    let sub = RwSignal::new(Sub::Recettes);

    let tab_button = move |target: Sub, label: &'static str| {
        view! {
            <button
                class="btn"
                class:btn-primary=move || sub.get() == target
                class:btn-secondary=move || sub.get() != target
                on:click=move |_| sub.set(target)
            >
                {label}
            </button>
        }
    };

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Comptabilité & Déclarations"</h1>
                    <p>"Suivez vos recettes encaissées, estimez vos cotisations et surveillez vos seuils légaux."</p>
                </div>
            </div>

            <div
                class="flex-gap-2"
                style="border-bottom: 1px solid var(--border-glass); padding-bottom: 0.75rem; margin-bottom: 1.5rem"
            >
                {tab_button(Sub::Recettes, "Livre des Recettes")}
                {tab_button(Sub::Urssaf, "Déclaration URSSAF")}
                {tab_button(Sub::Seuils, "Seuils de Chiffre d'Affaires")}
            </div>

            {move || match sub.get() {
                Sub::Recettes => recettes().into_any(),
                Sub::Urssaf => urssaf().into_any(),
                Sub::Seuils => seuils().into_any(),
            }}
        </div>
    }
}

// ------------------------------------------------------ livre des recettes

fn recettes() -> impl IntoView {
    let app = use_app();

    // Affichage : le plus récent en premier. L'export, lui, est produit par
    // l'hôte dans l'ordre chronologique que la loi impose.
    let rows = move || {
        let mut paid: Vec<_> = app
            .snapshot
            .get()
            .map(|s| s.invoices.into_iter().filter(|i| i.status == "payee").collect())
            .unwrap_or_default();
        paid.sort_by(|a: &asgard_ipc::Invoice, b| {
            let key = |i: &asgard_ipc::Invoice| i.payment_date.clone().unwrap_or_else(|| i.date.clone());
            key(b).cmp(&key(a))
        });
        paid
    };

    let export = move |_| {
        if rows().is_empty() {
            app.inform("Erreur", "Aucune recette encaissée à exporter.");
            return;
        }
        leptos::task::spawn_local(async move {
            match ipc::call::<String>("recettes_csv").await {
                Ok(csv) => {
                    // Année courante dans le nom, comme l'original — et non
                    // l'année de la période affichée dans l'onglet URSSAF.
                    let year = js_sys::Date::new_0().get_full_year();
                    let name = format!("Livre_des_recettes_{year}.csv");
                    match actions::export_text(csv, name, "csv").await {
                        Ok(Some(path)) => {
                            app.inform("Enregistré", format!("Livre des recettes enregistré : {path}"))
                        }
                        Ok(None) => {}
                        Err(error) => app.report(error),
                    }
                }
                Err(error) => app.report(error),
            }
        });
    };

    view! {
        <div>
            <div class="flex-between" style="margin-bottom: 1rem">
                <h3 style="font-family: var(--font-title)">"Registre Chronologique des Recettes Encaissées"</h3>
                <button class="btn btn-secondary" on:click=export>"Exporter en CSV (.excel)"</button>
            </div>

            <div class="table-container" style="margin-top: 0">
                <table class="table-glass">
                    <thead>
                        <tr>
                            <th>"Date Encaissement"</th>
                            <th>"N° Facture"</th>
                            <th>"Client"</th>
                            <th>"Moyen de Règlement"</th>
                            <th>"Montant HT"</th>
                            <th>"Montant TTC"</th>
                        </tr>
                    </thead>
                    <tbody>
                        // Lignes redessinées à chaque changement : une `<For>` indexée
                        // sur l'identifiant garderait l'ancien contenu d'une ligne modifiée.
                        {move || {
                            rows()
                                .into_iter()
                                .map(|inv| view! {
                                    <tr>
                                        <td style="font-weight: 600; color: var(--color-gold)">
                                            {format::date(inv.payment_date.as_deref().unwrap_or(&inv.date))}
                                        </td>
                                        <td>{inv.invoice_number.clone()}</td>
                                        <td>{inv.company_name.clone()}</td>
                                        <td>{format::payment_method(inv.payment_method.as_deref())}</td>
                                        <td>{format::euros(inv.amount_ht)}</td>
                                        <td style="font-weight: 600">{format::euros(inv.amount_total)}</td>
                                    </tr>
                                })
                                .collect_view()
                        }}
                        <Show when=move || rows().is_empty()>
                            <tr>
                                <td colspan="6" style="text-align: center; padding: 2rem; color: var(--text-secondary)">
                                    "Aucune facture n'est encore marquée comme \"Payée\"."
                                </td>
                            </tr>
                        </Show>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ------------------------------------------------------- déclaration URSSAF

fn urssaf() -> impl IntoView {
    let app = use_app();

    // Toute modification de période recalcule la déclaration côté hôte.
    let set_period = move |change: fn(&mut Period, u32), value: u32| {
        app.period.update(|p| change(p, value));
        leptos::task::spawn_local(app.refresh_metrics());
    };

    let monthly = move || app.period.get().monthly;

    view! {
        <div>
            <h3 style="font-family: var(--font-title); margin-bottom: 1rem">
                "Simulateur de Déclaration Mensuelle / Trimestrielle"
            </h3>

            <div class="card-glass" style="margin-bottom: 1.5rem; padding: 1.25rem">
                <div style="display: flex; gap: 1.5rem; align-items: center; flex-wrap: wrap">
                    <div class="form-group" style="margin-bottom: 0; min-width: 150px">
                        <label class="form-label">"Type de Période"</label>
                        <select
                            class="form-input"
                            on:change=move |ev| {
                                set_period(|p, v| p.monthly = v == 1, u32::from(event_target_value(&ev) == "monthly"))
                            }
                        >
                            <option value="monthly" selected=monthly>"Mensuelle"</option>
                            <option value="quarterly" selected=move || !monthly()>"Trimestrielle"</option>
                        </select>
                    </div>

                    <div class="form-group" style="margin-bottom: 0; min-width: 120px">
                        <label class="form-label">"Année"</label>
                        <select
                            class="form-input"
                            on:change=move |ev| {
                                let year = event_target_value(&ev).parse().unwrap_or(2026);
                                set_period(|p, v| p.year = v as i32, year)
                            }
                        >
                            {[2026_u32, 2025].into_iter().map(|y| view! {
                                <option value=y.to_string() selected=move || app.period.get().year == y as i32>
                                    {y.to_string()}
                                </option>
                            }).collect_view()}
                        </select>
                    </div>

                    {move || if monthly() {
                        view! {
                            <div class="form-group" style="margin-bottom: 0; min-width: 150px">
                                <label class="form-label">"Mois"</label>
                                <select
                                    class="form-input"
                                    on:change=move |ev| {
                                        let m = event_target_value(&ev).parse().unwrap_or(1);
                                        set_period(|p, v| p.month = v, m)
                                    }
                                >
                                    {MONTHS.iter().enumerate().map(|(i, name)| {
                                        let value = i as u32 + 1;
                                        view! {
                                            <option value=value.to_string() selected=move || app.period.get().month == value>
                                                {*name}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="form-group" style="margin-bottom: 0; min-width: 150px">
                                <label class="form-label">"Trimestre"</label>
                                <select
                                    class="form-input"
                                    on:change=move |ev| {
                                        let q = event_target_value(&ev).parse().unwrap_or(1);
                                        set_period(|p, v| p.quarter = v, q)
                                    }
                                >
                                    {QUARTERS.iter().enumerate().map(|(i, name)| {
                                        let value = i as u32 + 1;
                                        view! {
                                            <option value=value.to_string() selected=move || app.period.get().quarter == value>
                                                {*name}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>

            {move || app.declaration.get().map(|d| view! {
                <div style="display: grid; grid-template-columns: 1.8fr 1.2fr; gap: 1.5rem">
                    <div class="card-glass" style="padding: 1.5rem">
                        <h4 style="font-family: var(--font-title); color: var(--color-gold); margin-bottom: 1.25rem">
                            "Montants à déclarer à l'URSSAF"
                        </h4>
                        <div style="display: flex; flex-direction: column; gap: 1rem">
                            <Line label="Prestations de Services Libérales (BNC)" line=d.bnc />
                            <Line label="Prestations de Services Artisanales/Comm. (BIC)" line=d.bic />
                            <Line label="Achat / Vente de Marchandises (BIC)" line=d.vente />
                        </div>
                        <div
                            class="flex-between"
                            style="margin-top: 1.5rem; padding: 1rem; background-color: rgba(255,255,255,0.02); border-radius: 8px; border: 1px solid var(--border-glass)"
                        >
                            <span style="font-weight: 700">"Total Charges Période"</span>
                            <span style="font-size: 1.25rem; font-weight: 800; color: #FF6B8B">
                                {format::euros(asgard_core::to_f64(d.total_charges))}
                            </span>
                        </div>
                    </div>

                    <div class="card-glass" style="padding: 1.5rem; display: flex; flex-direction: column; justify-content: space-between">
                        <div>
                            <h4 style="font-family: var(--font-title); margin-bottom: 1rem">"Informations de déclaration"</h4>
                            <p style="font-size: 0.85rem; color: var(--text-secondary); line-height: 1.5; margin-bottom: 1rem">
                                "Pour déclarer vos cotisations, rendez-vous sur votre espace professionnel "
                                <strong>"autoentrepreneur.urssaf.fr"</strong>"."
                            </p>
                            <div style="padding: 1rem; background-color: rgba(229,169,60,0.03); border-radius: 8px; border: 1px dashed rgba(229,169,60,0.2); font-size: 0.85rem; line-height: 1.5">
                                "📌 " <strong>"Rappel :"</strong>
                                " Vous devez déclarer le Chiffre d'Affaires "
                                <strong>"réellement encaissé"</strong>
                                " au cours de la période sélectionnée, et non le montant facturé non payé."
                            </div>
                        </div>
                        <div style="margin-top: 1.5rem">
                            <div style="font-size: 0.8rem; color: var(--text-muted); margin-bottom: 0.25rem">
                                "CA Encaissé sur la période"
                            </div>
                            <div style="font-size: 1.75rem; font-weight: 800; color: var(--color-gold)">
                                {format::euros(asgard_core::to_f64(d.total_ca))}
                            </div>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}

#[component]
fn Line(label: &'static str, line: DeclarationLine) -> impl IntoView {
    view! {
        <div class="flex-between" style="padding-bottom: 0.75rem; border-bottom: 1px solid var(--border-glass)">
            <div>
                <div style="font-weight: 600; font-size: 0.95rem">{label}</div>
                <span style="font-size: 0.75rem; color: var(--text-muted)">
                    {format!("Taux de cotisation appliqué : {}", format::rate(asgard_core::to_f64(line.rate)))}
                </span>
            </div>
            <div style="text-align: right">
                <div style="font-size: 1.15rem; font-weight: 700; color: var(--color-gold)">
                    {format::euros(asgard_core::to_f64(line.ht))}
                </div>
                <span style="font-size: 0.75rem; color: #FF6B8B">
                    {format!("Cotisations : {}", format::euros(asgard_core::to_f64(line.charges)))}
                </span>
            </div>
        </div>
    }
}

// ------------------------------------------------------------------- seuils

fn seuils() -> impl IntoView {
    let app = use_app();

    view! {
        <div>
            <h3 style="font-family: var(--font-title); margin-bottom: 1.25rem">
                "Surveillance des Seuils (Plafonds Annuels)"
            </h3>
            {move || app.dashboard.get().map(|d| {
                let g = d.gauges;
                view! {
                    <div style="display: flex; flex-direction: column; gap: 2rem">
                        <Family
                            title=format!("Activités de Services (Plafonds pour {})", g.year)
                            gauge=g.service
                            tva_label="Seuil Franchise de TVA (Services)"
                            micro_label="Plafond Régime Micro-Entreprise (Services)"
                            micro_note="Plafond légal au-delà duquel vous devez basculer vers un régime réel d'imposition."
                            noun="services"
                        />
                        <Family
                            title=format!("Activités d'Achat / Vente de Marchandises (Plafonds pour {})", g.year)
                            gauge=g.vente
                            tva_label="Seuil Franchise de TVA (Ventes)"
                            micro_label="Plafond Régime Micro-Entreprise (Ventes)"
                            micro_note="Plafond légal d'activité pour l'achat / revente de marchandises."
                            noun="ventes"
                        />
                    </div>
                }
            })}
        </div>
    }
}

#[component]
fn Family(
    title: String,
    gauge: Gauge,
    tva_label: &'static str,
    micro_label: &'static str,
    micro_note: &'static str,
    noun: &'static str,
) -> impl IntoView {
    let exceeded = gauge.exceeded_tva;
    let pct_tva = asgard_core::to_f64(gauge.pct_tva);
    let pct_micro = asgard_core::to_f64(gauge.pct_micro);

    let tva_fill = if exceeded {
        "linear-gradient(90deg, #FF6B8B, #EF4444)"
    } else {
        "linear-gradient(90deg, var(--color-gold), var(--color-gold-hover))"
    };

    let tva_note = if exceeded {
        "⚠️ Vous avez dépassé le seuil de franchise. Vous devez facturer de la TVA.".to_string()
    } else {
        format!(
            "Il vous reste {} de marge avant d'assujettir vos {noun} à la TVA.",
            format::round_euros(gauge.remaining_before_tva)
        )
    };

    view! {
        <div class="card-glass" style="padding: 1.5rem">
            <h4 style="font-family: var(--font-title); color: var(--color-gold); margin-bottom: 1rem">{title}</h4>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem">
                <div>
                    <div class="flex-between" style="margin-bottom: 0.5rem; font-size: 0.9rem">
                        <span>{tva_label}</span>
                        <span style="font-weight: 600">
                            {format!("{} / {}", format::round_euros(gauge.ca), format::round_euros(gauge.tva_tolerance))}
                        </span>
                    </div>
                    <div style="height: 10px; background-color: rgba(255,255,255,0.05); border-radius: 5px; overflow: hidden; margin-bottom: 0.5rem">
                        <div style=format!("height: 100%; width: {pct_tva}%; background: {tva_fill}; border-radius: 5px") />
                    </div>
                    <span style=format!(
                        "font-size: 0.75rem; color: {}",
                        if exceeded { "#FF6B8B" } else { "var(--text-muted)" }
                    )>
                        {tva_note}
                    </span>
                </div>
                <div>
                    <div class="flex-between" style="margin-bottom: 0.5rem; font-size: 0.9rem">
                        <span>{micro_label}</span>
                        <span style="font-weight: 600">
                            {format!("{} / {}", format::round_euros(gauge.ca), format::round_euros(gauge.micro_limit))}
                        </span>
                    </div>
                    <div style="height: 10px; background-color: rgba(255,255,255,0.05); border-radius: 5px; overflow: hidden; margin-bottom: 0.5rem">
                        <div style=format!(
                            "height: 100%; width: {pct_micro}%; background: linear-gradient(90deg, var(--color-blue), #38BDF8); border-radius: 5px"
                        ) />
                    </div>
                    <span style="font-size: 0.75rem; color: var(--text-muted)">{micro_note}</span>
                </div>
            </div>
        </div>
    }
}
