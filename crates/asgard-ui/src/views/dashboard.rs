//! Tableau de bord — port de `src/components/DashboardTab.jsx`.
//!
//! Tous les chiffres viennent de `asgard-core` depuis la phase 3 : cette vue ne
//! calcule rien, elle met en forme. C'est ce qui rend le portage mécanique —
//! l'essentiel du composant React était déjà devenu de la présentation.
//!
//! Une différence assumée : le tableau de bord affiche désormais **les deux
//! bases de cotisations**. La phase 3 a nommé la divergence que le JavaScript
//! entretenait sans le dire — l'encaissé, seul déclarable, et le facturé, simple
//! projection. Les montrer côte à côte est le prolongement naturel de ce choix.

use asgard_core::reporting::Dashboard;
use asgard_core::thresholds::AlertLevel;
use leptos::prelude::*;

use crate::format;
use crate::state::use_app;

pub fn dashboard() -> impl IntoView {
    let app = use_app();

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Tableau de bord"</h1>
                    <p>"Aperçu financier en temps réel de votre auto-entreprise."</p>
                </div>
            </div>

            {move || {
                app.dashboard
                    .get()
                    .map(|data| view! { <Contents data=data /> })
            }}
        </div>
    }
}

#[component]
fn Contents(data: Dashboard) -> impl IntoView {
    let alerts = data.alerts.clone();
    let projected_gap = data.urssaf_charges_projected - data.urssaf_charges;

    view! {
        <Alerts alerts=alerts />

        <div class="metrics-grid">
            <Metric
                label="CA HT encaissé"
                value=format::euros(asgard_core::to_f64(data.revenue.ht))
                note=format!(
                    "Facturé : {} HT",
                    format::euros(asgard_core::to_f64(data.revenue.ht_facture)),
                )
                tone="gold"
            />
            <Metric
                label="Dépenses totales"
                value=format::euros(asgard_core::to_f64(data.total_expenses))
                note="Achats et frais professionnels enregistrés".to_string()
                tone="red"
            />
            <Metric
                label="Charges URSSAF"
                value=format::euros(asgard_core::to_f64(data.urssaf_charges))
                note=if projected_gap.is_zero() {
                    "Sur encaissé — tout est réglé".to_string()
                } else {
                    // La projection n'est pas déclarable : le dire évite de
                    // laisser croire que ce montant est dû aujourd'hui.
                    format!(
                        "Sur encaissé · {} à venir si tout le facturé rentre",
                        format::euros(asgard_core::to_f64(projected_gap)),
                    )
                }
                tone="pink"
            />
            <Metric
                label="Bénéfice net réel"
                value=format::euros(asgard_core::to_f64(data.net_profit))
                note="Trésorerie réelle après charges et dépenses".to_string()
                tone="green"
            />
        </div>

        <div class="dashboard-grid">
            <Breakdown data=data.breakdown />
            <Categories data=data.expenses_by_category />
        </div>
    }
}

#[component]
fn Metric(
    label: &'static str,
    value: String,
    note: String,
    tone: &'static str,
) -> impl IntoView {
    view! {
        <div class="card-glass metric-card">
            <div class="metric-label">{label}</div>
            <div class=format!("metric-value metric-{tone}")>{value}</div>
            <span class="metric-subtext">{note}</span>
        </div>
    }
}

#[component]
fn Alerts(alerts: Vec<asgard_core::thresholds::Alert>) -> impl IntoView {
    // Les alertes arrivent par valeur, déjà calculées : un rendu conditionnel
    // ordinaire suffit, là où `Show` imposerait de cloner pour satisfaire deux
    // closures.
    (!alerts.is_empty()).then(|| {
        view! {
            <div class="alerts-stack">
                {alerts
                    .into_iter()
                    .map(|alert| {
                        let class = match alert.level {
                            AlertLevel::Danger => "alert alert-danger",
                            AlertLevel::Warning => "alert alert-warning",
                        };
                        view! {
                            <div class=class>
                                <strong>{alert.title}</strong>
                                <p>{alert.message}</p>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        }
    })
}

#[component]
fn Breakdown(data: asgard_core::reporting::Breakdown) -> impl IntoView {
    let rows = [
        ("Libérale (BNC)", data.bnc),
        ("Artisanale/Comm. (BIC)", data.bic),
        ("Vente Marchandises", data.vente),
    ];

    view! {
        <div class="card-glass" style="padding: 1.5rem">
            <h3>"Répartition du CA encaissé"</h3>
            <div class="breakdown-list">
                {rows
                    .into_iter()
                    .map(|(label, share)| {
                        let pct = asgard_core::to_f64(share);
                        view! {
                            <div class="breakdown-row">
                                <div class="flex-between">
                                    <span>{label}</span>
                                    <strong>{format::percent(pct)}</strong>
                                </div>
                                <div class="progress-track">
                                    <div
                                        class="progress-fill"
                                        style=format!("width: {pct:.1}%")
                                    />
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn Categories(data: asgard_core::reporting::ExpenseBreakdown) -> impl IntoView {
    let empty = data.list.is_empty();

    view! {
        <div class="card-glass" style="padding: 1.5rem">
            <h3>"Dépenses par catégorie"</h3>
            {if empty {
                view! {
                    <p style="color: var(--text-secondary)">"Aucune dépense enregistrée."</p>
                }
                    .into_any()
            } else {
                view! {
                    <div class="breakdown-list">
                        {data
                            .list
                            .into_iter()
                            .map(|slice| {
                                let pct = asgard_core::to_f64(slice.pct);
                                view! {
                                    <div class="breakdown-row">
                                        <div class="flex-between">
                                            <span>{slice.label}</span>
                                            <strong>
                                                {format::euros(asgard_core::to_f64(slice.amount))}
                                            </strong>
                                        </div>
                                        <div class="progress-track">
                                            <div
                                                class="progress-fill"
                                                style=format!("width: {pct:.1}%")
                                            />
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}
