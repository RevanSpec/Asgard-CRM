//! Icônes de l'interface.
//!
//! Générées automatiquement depuis `src/components/Icons.jsx` au moment du
//! portage, puis devenues la source : le fichier React a disparu avec la
//! bascule. On les modifie désormais ici, directement.
//!
//! Pourquoi une génération plutôt qu'une copie : une première version avait été
//! recopiée à la main. Trois icônes sur sept y
//! étaient fausses — il manquait des tracés à Clients, Devis et Factures, et les
//! deux dernières avaient été interverties. Rien ne l'aurait signalé sinon une
//! comparaison visuelle attentive : la génération supprime cette classe
//! d'erreur plutôt que de compter sur la vigilance.

use leptos::prelude::*;
use leptos::svg;

/// Enveloppe commune : mêmes attributs que dans `Icons.jsx`, dont
/// `stroke="currentColor"` qui fait suivre à l'icône la couleur de son parent.
fn frame(children: AnyView) -> AnyView {
    svg::svg()
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("width", "20")
        .attr("height", "20")
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "none")
        .attr("stroke", "currentColor")
        .attr("stroke-width", "2")
        .attr("stroke-linecap", "round")
        .attr("stroke-linejoin", "round")
        .child(children)
        .into_any()
}

pub fn dashboard() -> AnyView {
    frame(
        (
            svg::rect().attr("x", "3").attr("y", "3").attr("width", "7").attr("height", "9").into_any(),
            svg::rect().attr("x", "14").attr("y", "3").attr("width", "7").attr("height", "5").into_any(),
            svg::rect().attr("x", "14").attr("y", "12").attr("width", "7").attr("height", "9").into_any(),
            svg::rect().attr("x", "3").attr("y", "16").attr("width", "7").attr("height", "5").into_any(),
        )
            .into_any(),
    )
}

pub fn clients() -> AnyView {
    frame(
        (
            svg::path().attr("d", "M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2").into_any(),
            svg::circle().attr("cx", "9").attr("cy", "7").attr("r", "4").into_any(),
            svg::path().attr("d", "M23 21v-2a4 4 0 0 0-3-3.87").into_any(),
            svg::path().attr("d", "M16 3.13a4 4 0 0 1 0 7.75").into_any(),
        )
            .into_any(),
    )
}

pub fn invoices() -> AnyView {
    frame(
        (
            svg::path().attr("d", "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z").into_any(),
            svg::polyline().attr("points", "14 2 14 8 20 8").into_any(),
            svg::line().attr("x1", "16").attr("y1", "13").attr("x2", "8").attr("y2", "13").into_any(),
            svg::line().attr("x1", "16").attr("y1", "17").attr("x2", "8").attr("y2", "17").into_any(),
            svg::polyline().attr("points", "10 9 9 9 8 9").into_any(),
        )
            .into_any(),
    )
}

pub fn settings() -> AnyView {
    frame(
        (
            svg::circle().attr("cx", "12").attr("cy", "12").attr("r", "3").into_any(),
            svg::path().attr("d", "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z").into_any(),
        )
            .into_any(),
    )
}

pub fn add() -> AnyView {
    frame(
        (
            svg::line().attr("x1", "12").attr("y1", "5").attr("x2", "12").attr("y2", "19").into_any(),
            svg::line().attr("x1", "5").attr("y1", "12").attr("x2", "19").attr("y2", "12").into_any(),
        )
            .into_any(),
    )
}

pub fn edit() -> AnyView {
    frame(
        (
            svg::path().attr("d", "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7").into_any(),
            svg::path().attr("d", "M18.5 2.5a2.121 2.121 0 1 1 3 3L12 15l-4 1 1-4z").into_any(),
        )
            .into_any(),
    )
}

pub fn delete() -> AnyView {
    frame(
        (
            svg::polyline().attr("points", "3 6 5 6 21 6").into_any(),
            svg::path().attr("d", "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2").into_any(),
            svg::line().attr("x1", "10").attr("y1", "11").attr("x2", "10").attr("y2", "17").into_any(),
            svg::line().attr("x1", "14").attr("y1", "11").attr("x2", "14").attr("y2", "17").into_any(),
        )
            .into_any(),
    )
}

pub fn download() -> AnyView {
    frame(
        (
            svg::path().attr("d", "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4").into_any(),
            svg::polyline().attr("points", "7 10 12 15 17 10").into_any(),
            svg::line().attr("x1", "12").attr("y1", "15").attr("x2", "12").attr("y2", "3").into_any(),
        )
            .into_any(),
    )
}

pub fn search() -> AnyView {
    frame(
        (
            svg::circle().attr("cx", "11").attr("cy", "11").attr("r", "8").into_any(),
            svg::line().attr("x1", "21").attr("y1", "21").attr("x2", "16.65").attr("y2", "16.65").into_any(),
        )
            .into_any(),
    )
}

pub fn email() -> AnyView {
    frame(
        (
            svg::path().attr("d", "M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z").into_any(),
            svg::polyline().attr("points", "22,6 12,13 2,6").into_any(),
        )
            .into_any(),
    )
}

pub fn estimates() -> AnyView {
    frame(
        (
            svg::path().attr("d", "M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2").into_any(),
            svg::rect().attr("x", "8").attr("y", "2").attr("width", "8").attr("height", "4").attr("rx", "1").attr("ry", "1").into_any(),
            svg::path().attr("d", "M9 14h6").into_any(),
            svg::path().attr("d", "M9 18h6").into_any(),
            svg::path().attr("d", "M9 10h6").into_any(),
        )
            .into_any(),
    )
}

pub fn expenses() -> AnyView {
    frame(
        (
            svg::rect().attr("x", "1").attr("y", "4").attr("width", "22").attr("height", "16").attr("rx", "2").attr("ry", "2").into_any(),
            svg::line().attr("x1", "1").attr("y1", "10").attr("x2", "23").attr("y2", "10").into_any(),
        )
            .into_any(),
    )
}

pub fn accounting() -> AnyView {
    frame(
        (
            svg::line().attr("x1", "18").attr("y1", "20").attr("x2", "18").attr("y2", "10").into_any(),
            svg::line().attr("x1", "12").attr("y1", "20").attr("x2", "12").attr("y2", "4").into_any(),
            svg::line().attr("x1", "6").attr("y1", "20").attr("x2", "6").attr("y2", "14").into_any(),
        )
            .into_any(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les treize icônes de l'original, toutes présentes. La génération ayant
    /// eu lieu une fois, c'est la liste qui fait foi désormais.
    #[test]
    fn every_icon_renders() {
        let icons: [fn() -> AnyView; 13] = [
            dashboard, clients, invoices, settings, add, edit, delete,
            download, search, email, estimates, expenses, accounting,
        ];
        assert_eq!(icons.len(), 13);
    }
}
