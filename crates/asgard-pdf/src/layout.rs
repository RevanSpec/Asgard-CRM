//! Primitives de mise en page.
//!
//! jsPDF exprime tout en millimètres depuis le **coin supérieur gauche**, alors
//! que printpdf compte depuis le **coin inférieur gauche**, convention du format
//! PDF. Porter la mise en page revient donc surtout à retourner l'axe vertical :
//! ces fonctions le font une fois pour toutes, afin que le code de mise en page
//! garde les mêmes coordonnées que l'original et reste comparable ligne à ligne.

use printpdf::*;

/// Hauteur d'une page A4, en millimètres.
pub const PAGE_HEIGHT: f32 = 297.0;
pub const PAGE_WIDTH: f32 = 210.0;

/// Convertit une ordonnée « jsPDF » (depuis le haut) en ordonnée PDF (depuis le bas).
pub fn flip(y: f32) -> Mm {
    Mm(PAGE_HEIGHT - y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    /// Utilisé pour le pied de page, centré comme dans l'original.
    Center,
}

/// Une couleur RVB, exprimée en octets comme dans l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn to_color(self) -> Color {
        Color::Rgb(printpdf::Rgb::new(
            self.0 as f32 / 255.0,
            self.1 as f32 / 255.0,
            self.2 as f32 / 255.0,
            None,
        ))
    }
}

/// Lit une couleur hexadécimale `#RRGGBB` ou `#RGB`.
///
/// Port de `hexToRgb` (`src/pdfGenerator.js`), repli compris : une valeur
/// illisible retombe sur l'or de l'interface plutôt que de faire échouer
/// l'édition d'une facture.
pub fn parse_hex(hex: &str) -> Rgb {
    const FALLBACK: Rgb = Rgb(229, 169, 60);

    let digits: Vec<char> = hex.trim_start_matches('#').chars().collect();

    let expand = |c: char| -> Option<u8> { c.to_digit(16).map(|d| (d * 17) as u8) };
    let pair = |a: char, b: char| -> Option<u8> {
        Some((a.to_digit(16)? * 16 + b.to_digit(16)?) as u8)
    };

    match digits.len() {
        3 => match (expand(digits[0]), expand(digits[1]), expand(digits[2])) {
            (Some(r), Some(g), Some(b)) => Rgb(r, g, b),
            _ => FALLBACK,
        },
        6 => match (
            pair(digits[0], digits[1]),
            pair(digits[2], digits[3]),
            pair(digits[4], digits[5]),
        ) {
            (Some(r), Some(g), Some(b)) => Rgb(r, g, b),
            _ => FALLBACK,
        },
        _ => FALLBACK,
    }
}

/// Largeur d'un texte, en millimètres, pour une police standard.
///
/// L'alignement à droite de jsPDF mesure le texte puis décale le point
/// d'insertion. printpdf n'aligne pas : il faut faire le calcul soi-même, donc
/// disposer des métriques de la police.
///
/// Helvetica est l'une des quatorze polices standard du format PDF : ses
/// métriques sont normalisées et identiques à celles qu'utilise jsPDF. C'est ce
/// qui rend le portage fidèle **sans embarquer de fonte** — le plan de migration
/// prévoyait d'en embarquer une et d'accepter des écarts de métriques ; ce
/// n'était pas nécessaire.
pub fn text_width_mm(text: &str, font_size: f32, bold: bool) -> f32 {
    let widths = if bold {
        &HELVETICA_BOLD_WIDTHS
    } else {
        &HELVETICA_WIDTHS
    };

    // Les largeurs sont exprimées en millièmes de cadratin ; un point vaut
    // 1/72 de pouce, soit 25,4/72 mm.
    let units: u32 = text
        .chars()
        .map(|c| {
            let index = latin1_index(c);
            widths[index] as u32
        })
        .sum();

    (units as f32 / 1000.0) * font_size * 25.4 / 72.0
}

/// Indice d'un caractère dans la table de largeurs.
///
/// Les polices standard utilisent WinAnsiEncoding, qui couvre le latin-1 — donc
/// tous les accents français. Un caractère hors table est traité comme un `?`,
/// de largeur voisine.
fn latin1_index(c: char) -> usize {
    let code = c as u32;
    if (32..=255).contains(&code) {
        (code - 32) as usize
    } else {
        ('?' as u32 - 32) as usize
    }
}

/// Largeurs Helvetica, du caractère 32 au 255, en millièmes de cadratin.
/// Valeurs de la spécification PDF (fichiers AFM des polices standard).
static HELVETICA_WIDTHS: [u16; 224] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // 32-47
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, // 48-63
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, // 64-79
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556, // 80-95
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, // 96-111
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584, 350, // 112-127
    556, 350, 222, 556, 333, 1000, 556, 556, 333, 1000, 667, 333, 1000, 350, 611, 350, // 128-143
    350, 222, 222, 333, 333, 350, 556, 1000, 333, 1000, 500, 333, 944, 350, 500, 667, // 144-159
    278, 333, 556, 556, 556, 556, 260, 556, 333, 737, 370, 556, 584, 333, 737, 333, // 160-175
    400, 584, 333, 333, 333, 556, 537, 278, 333, 333, 365, 556, 834, 834, 834, 611, // 176-191
    667, 667, 667, 667, 667, 667, 1000, 722, 667, 667, 667, 667, 278, 278, 278, 278, // 192-207
    722, 722, 778, 778, 778, 778, 778, 584, 778, 722, 722, 722, 722, 667, 667, 611, // 208-223
    556, 556, 556, 556, 556, 556, 889, 500, 556, 556, 556, 556, 278, 278, 278, 278, // 224-239
    556, 556, 556, 556, 556, 556, 556, 584, 611, 556, 556, 556, 556, 500, 556, 500, // 240-255
];

/// Largeurs Helvetica-Bold, mêmes conventions.
static HELVETICA_BOLD_WIDTHS: [u16; 224] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611,
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556,
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611,
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584, 350,
    556, 350, 278, 556, 500, 1000, 556, 556, 333, 1000, 667, 333, 1000, 350, 611, 350,
    350, 278, 278, 500, 500, 350, 556, 1000, 333, 1000, 556, 333, 944, 350, 500, 667,
    278, 333, 556, 556, 556, 556, 280, 556, 333, 737, 370, 556, 584, 333, 737, 333,
    400, 584, 333, 333, 333, 611, 556, 278, 333, 333, 365, 556, 834, 834, 834, 611,
    722, 722, 722, 722, 722, 722, 1000, 722, 667, 667, 667, 667, 278, 278, 278, 278,
    722, 722, 778, 778, 778, 778, 778, 584, 778, 722, 722, 722, 722, 667, 667, 611,
    556, 556, 556, 556, 556, 556, 889, 556, 556, 556, 556, 556, 278, 278, 278, 278,
    611, 611, 611, 611, 611, 611, 611, 584, 611, 611, 611, 611, 611, 556, 611, 556,
];

/// Découpe un texte pour qu'il tienne dans une largeur donnée.
///
/// Port de l'option `maxWidth` de `doc.text()`, utilisée pour les adresses et
/// les descriptions de prestation.
pub fn wrap(text: &str, max_width_mm: f32, font_size: f32, bold: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        if text_width_mm(&candidate, font_size, bold) <= max_width_mm || current.is_empty() {
            current = candidate;
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flips_the_vertical_axis() {
        // jsPDF place l'origine en haut ; le PDF la place en bas.
        assert_eq!(flip(0.0), Mm(297.0));
        assert_eq!(flip(297.0), Mm(0.0));
        assert_eq!(flip(20.0), Mm(277.0));
    }

    #[test]
    fn parses_hex_colours_like_the_javascript_helper() {
        assert_eq!(parse_hex("#E5A93C"), Rgb(229, 169, 60));
        assert_eq!(parse_hex("E5A93C"), Rgb(229, 169, 60));
        assert_eq!(parse_hex("#fff"), Rgb(255, 255, 255));
        assert_eq!(parse_hex("#000"), Rgb(0, 0, 0));
    }

    #[test]
    fn falls_back_to_gold_on_an_unreadable_colour() {
        // Le comportement du JavaScript : une couleur illisible ne doit pas
        // faire échouer l'édition d'une facture.
        for bad in ["", "#12", "pas une couleur", "#GGGGGG", "#1234567"] {
            assert_eq!(parse_hex(bad), Rgb(229, 169, 60), "entrée : {bad:?}");
        }
    }

    #[test]
    fn measures_text_with_the_standard_helvetica_metrics() {
        // Une espace en Helvetica 9 pt : 278/1000 × 9 × 25,4/72 ≈ 0,882 mm.
        let space = text_width_mm(" ", 9.0, false);
        assert!((space - 0.882).abs() < 0.01, "largeur mesurée : {space}");

        // Le gras est plus large que le romain, à taille égale.
        assert!(text_width_mm("FACTURE", 20.0, true) > text_width_mm("FACTURE", 20.0, false));
    }

    #[test]
    fn measures_accented_characters() {
        // Les accents français doivent être mesurés, pas ignorés : sans cela
        // l'alignement à droite dériverait sur « FACTURÉ À ».
        assert!(text_width_mm("É", 9.0, false) > 0.0);
        assert!(text_width_mm("à", 9.0, false) > 0.0);
        assert_eq!(
            text_width_mm("E", 9.0, false),
            text_width_mm("É", 9.0, false),
            "en Helvetica, É a la largeur de E"
        );
    }

    #[test]
    fn width_grows_with_font_size() {
        let small = text_width_mm("Asgard", 8.0, false);
        let large = text_width_mm("Asgard", 16.0, false);
        assert!((large - small * 2.0).abs() < 0.001);
    }

    #[test]
    fn wraps_text_within_a_maximum_width() {
        let address = "108 route de Malibu, 75008 Paris";
        let lines = wrap(address, 30.0, 9.0, false);

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(
                text_width_mm(line, 9.0, false) <= 30.0,
                "ligne trop large : {line:?}"
            );
        }
        assert_eq!(lines.join(" "), address);
    }

    #[test]
    fn a_single_long_word_is_not_lost() {
        let lines = wrap("Anticonstitutionnellement", 5.0, 9.0, false);
        assert_eq!(lines, vec!["Anticonstitutionnellement"]);
    }

    #[test]
    fn empty_text_yields_one_empty_line() {
        assert_eq!(wrap("", 50.0, 9.0, false), vec![""]);
    }
}
