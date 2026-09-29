//! Tableau de bord — port de `src/components/DashboardTab.jsx`.
//!
//! Tous les chiffres viennent de `asgard-core` depuis la phase 3 : cette vue ne
//! calcule rien, elle met en forme — sauf les coordonnées du graphique, reprises
//! telles quelles de l'original.
//!
//! Une différence assumée : sous les charges URSSAF, le tableau de bord indique
//! aussi **ce qui reste à venir** si tout le facturé est encaissé. La phase 3 a
//! nommé la divergence que le JavaScript entretenait sans le dire — l'encaissé,
//! seul déclarable, et le facturé, simple projection.

use asgard_core::reporting::{Breakdown, Dashboard, ExpenseBreakdown, MonthlySeries};
use asgard_core::thresholds::{Alert, AlertLevel};
use asgard_core::{to_f64, Money};
use asgard_ipc::{Client, Invoice};
use leptos::prelude::*;

use super::documents::invoice_status_badge;
use super::icons;
use super::invoices::{email_invoice, open_invoice_form, CreateForm};
use super::widgets::{GoldButton, IconButton};
use crate::state::{use_app, PaymentForm};
use crate::templates::Kind;
use crate::{actions, format, settings};

pub fn dashboard() -> impl IntoView {
    let app = use_app();
    let creating = RwSignal::new(false);

    view! {
        <div>
            <div class="page-header">
                <div class="page-title-container">
                    <h1>"Tableau de bord"</h1>
                    <p>"Aperçu financier en temps réel de votre auto-entreprise."</p>
                </div>
                <div class="flex-gap-2">
                    <button class="btn btn-primary" on:click=move |_| open_invoice_form(app, creating)>
                        {icons::add()}
                        " Nouvelle Facture"
                    </button>
                </div>
            </div>

            {move || app.dashboard.get().map(|data| view! { <Contents data=data /> })}

            <Show when=move || creating.get()>
                <CreateForm open=creating />
            </Show>
        </div>
    }
}

fn euros(amount: Money) -> String {
    format::euros(to_f64(amount))
}

#[component]
fn Contents(data: Dashboard) -> impl IntoView {
    let acre = settings::load().acre_enabled;
    let projected_gap = data.urssaf_charges_projected - data.urssaf_charges;

    view! {
        <Alerts alerts=data.alerts.clone() />

        <div class="dashboard-grid">
            <div class="card-glass">
                <div class="metric-label">"CA HT Encaissé"</div>
                <div class="metric-value metric-highlight">{euros(data.revenue.ht)}</div>
                <div class="metric-subtext">
                    {format!(
                        "Facturé : {} HT (TTC : {})",
                        euros(data.revenue.ht_facture),
                        euros(data.revenue.ttc),
                    )}
                </div>
            </div>
            <div class="card-glass" style="border-color: rgba(239, 68, 68, 0.2)">
                <div class="metric-label">"Dépenses Totales"</div>
                <div class="metric-value" style="color: #EF4444">{euros(data.total_expenses)}</div>
                <div class="metric-subtext">"Achats et frais professionnels enregistrés"</div>
            </div>
            <div class="card-glass" style="border-color: rgba(244, 63, 94, 0.2)">
                <div class="metric-label">"Charges URSSAF"</div>
                <div class="metric-value" style="color: #FF6B8B">{euros(data.urssaf_charges)}</div>
                <div class="metric-subtext">
                    {format!("Sur encaissé • {}", if acre { "Taux ACRE (-50%)" } else { "Taux plein" })}
                </div>
                // La projection n'est pas déclarable : le dire évite de laisser
                // croire que ce montant est dû aujourd'hui.
                {(!projected_gap.is_zero()).then(|| view! {
                    <div class="metric-subtext">
                        {format!("{} à venir si tout le facturé est encaissé", euros(projected_gap))}
                    </div>
                })}
            </div>
            <div class="card-glass" style="border-color: rgba(16, 185, 129, 0.3)">
                <div class="metric-label">"Bénéfice Net Réel"</div>
                <div class="metric-value" style="color: #10B981">{euros(data.net_profit)}</div>
                <div class="metric-subtext">"Trésorerie réelle après charges et dépenses"</div>
            </div>
        </div>

        <div class="dashboard-details-grid" style="margin-bottom: 2rem">
            <Chart series=data.monthly.clone() />
            <Shares breakdown=data.breakdown categories=data.expenses_by_category.clone() />
        </div>

        <div class="dashboard-details-grid">
            <LatestInvoices />
            <LatestClients />
        </div>
    }
}

#[component]
fn Alerts(alerts: Vec<Alert>) -> impl IntoView {
    (!alerts.is_empty()).then(|| {
        view! {
            <div style="display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 1.5rem">
                {alerts
                    .into_iter()
                    .map(|alert| {
                        let danger = alert.level == AlertLevel::Danger;
                        let (border, background, icon, color) = if danger {
                            ("rgba(239, 68, 68, 0.4)", "rgba(239, 68, 68, 0.05)", "🚨", "#EF4444")
                        } else {
                            ("rgba(229, 169, 60, 0.4)", "rgba(229, 169, 60, 0.03)", "⚠️", "var(--color-gold)")
                        };
                        view! {
                            <div
                                class="card-glass"
                                style=format!(
                                    "padding: 1rem 1.25rem; border-color: {border}; background: {background}; \
                                     display: flex; align-items: flex-start; gap: 0.75rem"
                                )
                            >
                                <span style="font-size: 1.25rem">{icon}</span>
                                <div>
                                    <h4 style=format!("margin: 0; font-weight: 700; color: {color}; font-size: 0.95rem")>
                                        {alert.title}
                                    </h4>
                                    <p style="margin: 0.25rem 0 0 0; font-size: 0.85rem; color: var(--text-secondary); line-height: 1.4">
                                        {alert.message}
                                    </p>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        }
    })
}

/// `Math.round` : arrondi au plus proche, les moitiés vers le haut
/// (`Math.round(-2.5) === -2`), contrairement à `f64::round`.
fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

/// Graphique « CA vs Bénéfice net », au format SVG de l'original.
///
/// Les coordonnées sont celles de `DashboardTab.jsx` (`getY`, abscisses de
/// 45 en 45). Le balisage est produit en texte : il ne contient que des
/// nombres et les libellés de mois du noyau, échappés par précaution.
fn chart_svg(series: &MonthlySeries) -> String {
    let (min, max) = (to_f64(series.min_val), to_f64(series.max_val));
    let get_y = |value: f64| {
        let range = max - min;
        if range == 0.0 {
            return 230.0;
        }
        230.0 - (value - min) / range * 200.0
    };
    let x_of = |index: usize| 60.0 + index as f64 * 45.0;

    let values = |list: &[Money]| list.iter().map(|v| to_f64(*v)).collect::<Vec<f64>>();
    let ca = values(&series.ca_values);
    let profit = values(&series.profit_values);

    let path = |points: &[f64]| {
        points
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{} {} {}", if i == 0 { "M" } else { "L" }, x_of(i), get_y(*v)))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let area = |points: &[f64]| {
        if points.is_empty() {
            return String::new();
        }
        let zero = get_y(0.0);
        format!("{} L {} {zero} L {} {zero} Z", path(points), x_of(points.len() - 1), x_of(0))
    };

    let mut svg = String::from(concat!(
        r#"<svg width="100%" height="100%" viewBox="0 0 600 280" preserveAspectRatio="none">"#,
        r#"<defs>"#,
        r#"<linearGradient id="ca-gradient" x1="0" y1="0" x2="0" y2="1">"#,
        r#"<stop offset="0%" stop-color="var(--color-gold)" stop-opacity="0.25"/>"#,
        r#"<stop offset="100%" stop-color="var(--color-gold)" stop-opacity="0.0"/>"#,
        r#"</linearGradient>"#,
        r#"<linearGradient id="profit-gradient" x1="0" y1="0" x2="0" y2="1">"#,
        r##"<stop offset="0%" stop-color="#10B981" stop-opacity="0.25"/>"##,
        r##"<stop offset="100%" stop-color="#10B981" stop-opacity="0.0"/>"##,
        r#"</linearGradient>"#,
        r#"</defs>"#,
    ));

    // Lignes horizontales et graduations.
    for ratio in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let value = min + ratio * (max - min);
        let y = get_y(value);
        svg += &format!(
            r#"<g><line x1="45" y1="{y}" x2="570" y2="{y}" class="chart-grid-line"/><text x="40" y="{}" text-anchor="end" fill="var(--text-muted)" font-size="8" font-family="monospace">{}</text></g>"#,
            y + 3.0,
            js_round(value),
        );
    }

    // Aires, ligne de zéro, courbes.
    svg += &format!(r#"<path d="{}" fill="url(#ca-gradient)"/>"#, area(&ca));
    svg += &format!(r#"<path d="{}" fill="url(#profit-gradient)"/>"#, area(&profit));
    if min < 0.0 {
        let zero = get_y(0.0);
        svg += &format!(
            r##"<line x1="45" y1="{zero}" x2="570" y2="{zero}" stroke="#EF4444" stroke-dasharray="4 4" stroke-width="1.5" opacity="0.6"/>"##
        );
    }
    svg += &format!(r#"<path d="{}" fill="none" stroke="var(--color-gold)" stroke-width="2"/>"#, path(&ca));
    svg += &format!(r##"<path d="{}" fill="none" stroke="#10B981" stroke-width="2"/>"##, path(&profit));

    // Points et valeurs.
    for (i, v) in ca.iter().enumerate() {
        let (x, y) = (x_of(i), get_y(*v));
        svg += &format!(
            r#"<g><circle cx="{x}" cy="{y}" r="3.5" fill="var(--bg-primary)" stroke="var(--color-gold)" stroke-width="1.5"/>"#
        );
        if *v > 0.0 {
            svg += &format!(
                r#"<text x="{x}" y="{}" text-anchor="middle" fill="var(--text-primary)" font-size="8" font-weight="600">{}</text>"#,
                y - 8.0,
                js_round(*v),
            );
        }
        svg += "</g>";
    }
    for (i, v) in profit.iter().enumerate() {
        let (x, y) = (x_of(i), get_y(*v));
        svg += &format!(
            r##"<g><circle cx="{x}" cy="{y}" r="3.5" fill="var(--bg-primary)" stroke="#10B981" stroke-width="1.5"/>"##
        );
        if v.abs() > 0.0 {
            let label_y = if *v >= 0.0 { y - 8.0 } else { y + 12.0 };
            svg += &format!(
                r##"<text x="{x}" y="{label_y}" text-anchor="middle" fill="#10B981" font-size="8" font-weight="600">{}</text>"##,
                js_round(*v),
            );
        }
        svg += "</g>";
    }

    // Mois en abscisse.
    for (i, label) in series.labels.iter().enumerate() {
        svg += &format!(
            r#"<text x="{}" y="255" text-anchor="middle" class="chart-label">{}</text>"#,
            x_of(i),
            escape(label),
        );
    }

    svg += "</svg>";
    svg
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[component]
fn Chart(series: MonthlySeries) -> impl IntoView {
    let year = js_sys::Date::new_0().get_full_year();
    let legend = |color: &'static str, label: &'static str| {
        view! {
            <div style="display: flex; align-items: center; gap: 0.4rem">
                <span style=format!(
                    "display: inline-block; width: 12px; height: 12px; background-color: {color}; border-radius: 2px"
                )></span>
                <span>{label}</span>
            </div>
        }
    };

    view! {
        <div class="card-glass">
            <h3 style="font-family: var(--font-title); margin-bottom: 0.75rem">
                {format!("Suivi CA vs Bénéfice Net ({year})")}
            </h3>
            <div style="display: flex; gap: 1.5rem; justify-content: center; margin-bottom: 1rem; font-size: 0.8rem">
                {legend("var(--color-gold)", "Chiffre d'Affaires HT")}
                {legend("#10B981", "Bénéfice Net Réel")}
            </div>
            <div style="position: relative; width: 100%; height: 320px" inner_html=chart_svg(&series)></div>
        </div>
    }
}

/// Onglets du panneau de droite.
#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Activity,
    Expenses,
}

#[component]
fn Shares(breakdown: Breakdown, categories: ExpenseBreakdown) -> impl IntoView {
    let panel = RwSignal::new(Panel::Activity);

    let tab = move |which: Panel, label: &'static str| {
        let style = move || {
            let active = panel.get() == which;
            format!(
                "background: none; border: none; color: {}; font-weight: 600; font-size: 0.9rem; \
                 cursor: pointer; padding: 0.25rem 0.5rem; border-bottom: {}; outline: none",
                if active { "var(--color-gold)" } else { "var(--text-muted)" },
                if active { "2px solid var(--color-gold)" } else { "none" },
            )
        };
        view! {
            <button type="button" style=style on:click=move |_| panel.set(which)>{label}</button>
        }
    };

    let activity = [
        ("Libérale (BNC)", breakdown.bnc, "var(--color-gold)"),
        ("Artisanale/Comm. (BIC)", breakdown.bic, "var(--color-blue)"),
        ("Vente Marchandises", breakdown.vente, "var(--success)"),
    ];

    view! {
        <div class="card-glass flex-between" style="flex-direction: column; align-items: stretch">
            <div style="display: flex; border-bottom: 1px solid var(--border-glass); margin-bottom: 1.25rem; padding-bottom: 0.5rem; gap: 0.5rem">
                {tab(Panel::Activity, "Activités")}
                {tab(Panel::Expenses, "Dépenses par catégorie")}
            </div>

            {move || match panel.get() {
                Panel::Activity => view! {
                    <div style="display: flex; flex-direction: column; justify-content: center; gap: 1.25rem; flex-grow: 1">
                        {activity
                            .into_iter()
                            .map(|(label, share, color)| {
                                let pct = to_f64(share);
                                view! {
                                    <div>
                                        <div class="flex-between" style="font-size: 0.85rem; margin-bottom: 0.4rem">
                                            <span style="font-weight: 500">{label}</span>
                                            <span style=format!("color: {color}; font-weight: 600")>
                                                {format::percent(pct)}
                                            </span>
                                        </div>
                                        <Bar pct=pct color=color height=8 />
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                }
                    .into_any(),
                Panel::Expenses => view! {
                    <div style="display: flex; flex-direction: column; gap: 0.85rem; flex-grow: 1; overflow-y: auto; max-height: 200px; padding-right: 4px">
                        {if categories.list.is_empty() {
                            view! {
                                <div style="text-align: center; color: var(--text-muted); font-size: 0.85rem; padding: 2rem 0">
                                    "Aucune dépense enregistrée pour le moment."
                                </div>
                            }
                                .into_any()
                        } else {
                            categories
                                .list
                                .iter()
                                .map(|slice| {
                                    let pct = to_f64(slice.pct);
                                    view! {
                                        <div>
                                            <div class="flex-between" style="font-size: 0.85rem; margin-bottom: 0.3rem">
                                                <span style="font-weight: 500">{slice.label.clone()}</span>
                                                <span style="color: var(--color-gold); font-weight: 600">
                                                    {format!("{} ({})", euros(slice.amount), format::percent(pct))}
                                                </span>
                                            </div>
                                            <Bar pct=pct color="var(--color-gold)" height=6 />
                                        </div>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </div>
                }
                    .into_any(),
            }}
        </div>
    }
}

/// Barre de progression des répartitions.
#[component]
fn Bar(pct: f64, color: &'static str, height: u32) -> impl IntoView {
    let radius = height / 2;
    view! {
        <div style=format!(
            "height: {height}px; background-color: var(--bg-tertiary); border-radius: {radius}px; overflow: hidden"
        )>
            <div style=format!(
                "width: {pct}%; height: 100%; background-color: {color}; border-radius: {radius}px"
            )></div>
        </div>
    }
}

/// Les dix premières factures, dans l'ordre de la liste des factures.
fn latest_invoices(invoices: &[Invoice]) -> Vec<Invoice> {
    invoices.iter().take(10).cloned().collect()
}

/// Les cinq clients les plus récemment créés.
fn latest_clients(clients: &[Client]) -> Vec<Client> {
    let mut sorted = clients.to_vec();
    // Dates ISO 8601 : l'ordre des chaînes est l'ordre chronologique.
    sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    sorted.truncate(5);
    sorted
}

#[component]
fn LatestInvoices() -> impl IntoView {
    let app = use_app();
    let rows = move || app.snapshot.with(|s| s.as_ref().map(|s| latest_invoices(&s.invoices)).unwrap_or_default());

    view! {
        <div class="card-glass">
            <h3 style="font-family: var(--font-title); margin-bottom: 1rem">"Dernières factures saisies (Top 10)"</h3>
            <Show
                when=move || !rows().is_empty()
                fallback=|| view! {
                    <div class="empty-state" style="padding: 2rem">
                        <p>"Aucune facture enregistrée pour le moment."</p>
                    </div>
                }
            >
                <div class="table-container" style="margin-top: 0">
                    <table class="table-glass">
                        <thead>
                            <tr>
                                <th>"Numéro"</th>
                                <th>"Client"</th>
                                <th>"Statut"</th>
                                <th>"Montant TTC"</th>
                                <th class="text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            // Lignes redessinées à chaque changement : une `<For>` indexée
                            // sur l'identifiant garderait l'ancien contenu d'une ligne modifiée.
                            {move || {
                                rows()
                                    .into_iter()
                                    .map(|invoice| view! { <InvoiceRow invoice=invoice /> })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn InvoiceRow(invoice: Invoice) -> impl IntoView {
    let app = use_app();
    let id = invoice.id;
    let number = invoice.invoice_number.clone();
    let for_email = invoice.clone();

    view! {
        <tr>
            <td style="font-weight: 600; color: var(--color-gold)">{invoice.invoice_number.clone()}</td>
            <td>{invoice.company_name.clone()}</td>
            // Badge corrigé : l'original testait `'envoye'` ici aussi, et une
            // facture envoyée n'affichait rien.
            <td>{invoice_status_badge(&invoice.status)}</td>
            <td style="font-weight: 600">{format::euros(invoice.amount_total)}</td>
            <td class="text-right">
                <div class="flex-gap-2" style="justify-content: flex-end">
                    {(invoice.status != "payee").then(|| view! {
                        <GoldButton
                            label="Régler"
                            title="Enregistrer le règlement"
                            on_click=Callback::new(move |_| {
                                app.payment.set(Some(PaymentForm { invoice_id: id, number: number.clone() }))
                            })
                        />
                    })}
                    <IconButton
                        icon=icons::email
                        title="Envoyer par e-mail"
                        on_click=Callback::new(move |_| email_invoice(app, &for_email, Kind::Invoice))
                    />
                    <IconButton
                        icon=icons::download
                        title="Exporter en PDF"
                        on_click=Callback::new(move |_| {
                            leptos::task::spawn_local(actions::export_pdf(app, Kind::Invoice, id))
                        })
                    />
                </div>
            </td>
        </tr>
    }
}

#[component]
fn LatestClients() -> impl IntoView {
    let app = use_app();
    let clients = move || app.snapshot.with(|s| s.as_ref().map(|s| latest_clients(&s.clients)).unwrap_or_default());

    view! {
        <div class="card-glass">
            <h3 style="font-family: var(--font-title); margin-bottom: 1rem">"Derniers clients (Top 5)"</h3>
            {move || {
                let list = clients();
                if list.is_empty() {
                    view! {
                        <div class="empty-state" style="padding: 2rem">
                            <p>"Aucun client enregistré."</p>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <div style="display: flex; flex-direction: column; gap: 0.75rem">
                            {list
                                .into_iter()
                                .map(|client| view! {
                                    <div
                                        class="flex-between"
                                        style="padding: 0.75rem 1rem; background-color: rgba(255,255,255,0.02); border-radius: 8px; border: 1px solid var(--border-glass)"
                                    >
                                        <div>
                                            <div style="font-weight: 600; font-size: 0.9rem">{client.company_name}</div>
                                            <div style="font-size: 0.8rem; color: var(--text-secondary)">{client.contact_name}</div>
                                        </div>
                                        <div class="badge badge-blue">"Client"</div>
                                    </div>
                                })
                                .collect_view()}
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn series(ca: [i64; 12], profit: [i64; 12]) -> MonthlySeries {
        let money = |v: &[i64; 12]| v.iter().map(|x| Money::from(*x)).collect::<Vec<_>>();
        let all: Vec<i64> = ca.iter().chain(profit.iter()).copied().collect();
        MonthlySeries {
            labels: asgard_core::reporting::MONTH_LABELS.iter().map(|m| m.to_string()).collect(),
            ca_values: money(&ca),
            profit_values: money(&profit),
            max_val: Money::from(*all.iter().max().unwrap()),
            min_val: Money::from(*all.iter().min().unwrap()),
        }
    }

    #[test]
    fn math_round_rounds_halves_up() {
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(js_round(1999.4), 1999.0);
    }

    /// Mêmes coordonnées que `getY` dans l'original : le maximum en haut
    /// (y = 30), le minimum en bas (y = 230), les points de 45 en 45.
    #[test]
    fn chart_uses_the_original_coordinates() {
        let mut ca = [0; 12];
        ca[1] = 1000;
        let svg = chart_svg(&series(ca, [0; 12]));

        assert!(svg.contains(r#"d="M 60 230 L 105 30 L 150 230"#), "{svg}");
        // Valeur affichée au-dessus du point de février.
        assert!(svg.contains(r#"<text x="105" y="22" text-anchor="middle" fill="var(--text-primary)" font-size="8" font-weight="600">1000</text>"#));
        assert!(svg.contains(r#"class="chart-label">Déc</text>"#));

        // Aucun guillemet égaré après une balise fermante : le balisage est
        // assemblé à la main, une faute de délimiteur y passerait inaperçue.
        assert!(!svg.contains(r#"/>""#), "{svg}");
        assert_eq!(svg.matches("<g>").count(), svg.matches("</g>").count());
    }

    #[test]
    fn a_flat_year_draws_on_the_baseline() {
        let svg = chart_svg(&series([0; 12], [0; 12]));
        assert!(svg.contains(r#"d="M 60 230 L 105 230"#));
        // Aucune valeur nulle n'est étiquetée.
        assert!(!svg.contains(r#"font-weight="600">0</text>"#));
    }

    /// Un mois à perte trace la ligne de zéro, pointillée en rouge.
    #[test]
    fn a_loss_draws_the_zero_line() {
        let mut profit = [0; 12];
        profit[0] = -500;
        let mut ca = [0; 12];
        ca[0] = 500;
        let svg = chart_svg(&series(ca, profit));
        assert!(svg.contains(r#"stroke-dasharray="4 4""#));
        // Étiquette d'une perte placée sous le point.
        assert!(svg.contains(r##"y="242" text-anchor="middle" fill="#10B981""##), "{svg}");
    }

    #[test]
    fn labels_are_escaped() {
        let mut s = series([0; 12], [0; 12]);
        s.labels[0] = "<b>".into();
        assert!(chart_svg(&s).contains("&lt;b&gt;"));
    }

    #[test]
    fn latest_clients_are_the_five_most_recent() {
        let client = |id: i64, created: &str| Client {
            id,
            company_name: format!("C{id}"),
            contact_name: String::new(),
            email: String::new(),
            phone: String::new(),
            address: String::new(),
            created_at: created.into(),
        };
        let clients: Vec<Client> = (1..=7).map(|i| client(i, &format!("2026-0{i}-01T00:00:00Z"))).collect();
        let ids: Vec<i64> = latest_clients(&clients).iter().map(|c| c.id).collect();
        assert_eq!(ids, [7, 6, 5, 4, 3]);
    }

    #[test]
    fn money_is_formatted_like_the_rest_of_the_port() {
        assert_eq!(euros(dec!(1450.5)), "1\u{202F}450,50\u{A0}€");
    }
}
