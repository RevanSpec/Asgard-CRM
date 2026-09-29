//! Génération des factures et devis au format PDF.
//!
//! Port de `src/pdfGenerator.js` (jsPDF). La mise en page est reproduite au
//! millimètre : mêmes coordonnées, mêmes tailles, mêmes libellés.
//!
//! **Sur les polices.** Le plan de migration annonçait qu'il faudrait embarquer
//! une fonte en Rust, avec des métriques légèrement différentes de jsPDF, et
//! valider page par page. C'était inexact : Helvetica fait partie des quatorze
//! polices standard du format PDF, et `printpdf` les expose comme jsPDF. Les
//! métriques sont donc **identiques**, sans rien embarquer — le binaire n'en
//! grossit pas et le rendu ne dérive pas.
//!
//! **Sur les montants.** L'ancienne version recevait des `f64` et les formatait
//! avec `.toFixed(2)`. Ici les montants sont des `Decimal` venus du noyau
//! métier : le PDF imprime exactement ce que la base contient.

mod layout;

use asgard_core::model::{CivilDate, ServiceType};
use asgard_core::money::{round_cents, Money};
use layout::{flip, parse_hex, text_width_mm, wrap, Align, Rgb, PAGE_HEIGHT, PAGE_WIDTH};
use printpdf::*;

/// Nature du document à éditer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    Invoice,
    Estimate,
    /// Avoir : annule ou corrige une facture émise, qui ne se modifie pas.
    CreditNote,
}

impl DocumentKind {
    fn is_invoice(self) -> bool {
        self == DocumentKind::Invoice
    }

    /// Une pièce qui constate une créance, ou son annulation : tout sauf un
    /// devis. Le cadre de signature et la largeur du pied de page en dépendent.
    fn is_binding(self) -> bool {
        self != DocumentKind::Estimate
    }

    fn banner(self) -> &'static str {
        match self {
            DocumentKind::Invoice => "FACTURE",
            DocumentKind::Estimate => "DEVIS",
            DocumentKind::CreditNote => "AVOIR",
        }
    }

    fn number_label(self) -> &'static str {
        match self {
            DocumentKind::Invoice => "N° Facture :",
            DocumentKind::Estimate => "N° Devis :",
            DocumentKind::CreditNote => "N° Avoir :",
        }
    }

    fn footer_noun(self) -> &'static str {
        match self {
            DocumentKind::Invoice => "Facture",
            DocumentKind::Estimate => "Devis",
            DocumentKind::CreditNote => "Avoir",
        }
    }

    /// Libellé du total. Un avoir n'est pas « à payer » : il est dû au client.
    fn total_label(self) -> &'static str {
        match self {
            DocumentKind::CreditNote => "TOTAL AVOIR (TTC) :",
            _ => "TOTAL NET À PAYER (TTC) :",
        }
    }
}

/// Ce que le document doit imprimer.
#[derive(Debug, Clone)]
pub struct Document {
    pub number: String,
    pub service_type: ServiceType,
    pub description: String,
    pub amount_ht: Money,
    pub tva_rate: Money,
    pub amount_tva: Money,
    pub amount_total: Money,
    pub date: CivilDate,
    /// Échéance de règlement. Absente sur les factures émises avant qu'elle ne
    /// soit enregistrée, et sur les devis, qui n'en ont pas.
    pub due_date: Option<CivilDate>,
    /// Numéro de la facture qu'un avoir corrige. La référence est obligatoire :
    /// un avoir isolé ne se rattache à rien.
    pub corrects: Option<String>,
}

/// Coordonnées du client destinataire.
#[derive(Debug, Clone, Default)]
pub struct Party {
    pub company_name: String,
    pub contact_name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
}

/// Coordonnées et mentions de l'émetteur.
#[derive(Debug, Clone)]
pub struct Issuer {
    pub company_name: String,
    pub contact_name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub siret: String,
    pub iban: String,
    pub accent_colour: String,
    /// Logo, tel que les réglages le stockent : une URL de données
    /// (`data:image/png;base64,…`). Vide quand l'émetteur n'en a pas.
    pub logo: String,
}

impl Issuer {
    /// Champs indispensables qui manquent pour éditer une pièce de cette nature.
    ///
    /// Une pièce ne doit jamais porter une identité que l'émetteur n'a pas
    /// saisie. L'ancienne version en fabriquait une — raison sociale de
    /// démonstration, IBAN de gabarit — et rien ne distinguait à l'écran une
    /// facture valable d'une facture au nom de personne (défaut D11).
    ///
    /// Le SIRET est obligatoire sur toute pièce commerciale. L'IBAN ne l'est
    /// que sur une facture : c'est la seule qui demande un règlement.
    pub fn missing_fields(&self, kind: DocumentKind) -> Vec<&'static str> {
        let mut missing = Vec::new();

        for (label, value) in [
            ("la raison sociale", &self.company_name),
            ("l'adresse", &self.address),
            ("le SIRET", &self.siret),
        ] {
            if value.trim().is_empty() {
                missing.push(label);
            }
        }

        if kind.is_invoice() && self.iban.trim().is_empty() {
            missing.push("l'IBAN");
        }

        missing
    }
}

impl Default for Issuer {
    /// Aucune identité par défaut : seule la couleur d'accentuation en a une,
    /// parce qu'un document doit bien être dessiné avec une couleur.
    fn default() -> Self {
        Self {
            company_name: String::new(),
            contact_name: String::new(),
            address: String::new(),
            phone: String::new(),
            email: String::new(),
            siret: String::new(),
            iban: String::new(),
            accent_colour: DEFAULT_ACCENT.into(),
            logo: String::new(),
        }
    }
}

/// Couleur d'accentuation retenue quand les réglages n'en fixent pas.
pub const DEFAULT_ACCENT: &str = "#E5A93C";

const NAVY: Rgb = Rgb(11, 15, 25);
const GREY: Rgb = Rgb(100, 116, 139);
const WHITE: Rgb = Rgb(255, 255, 255);
const TABLE_BG: Rgb = Rgb(248, 250, 252);
const RULE: Rgb = Rgb(229, 231, 235);

/// État de dessin, pour éviter de repasser les mêmes arguments partout.
struct Canvas<'a> {
    layer: PdfLayerReference,
    regular: &'a IndirectFontRef,
    bold: &'a IndirectFontRef,
}

impl Canvas<'_> {
    /// Écrit un texte, avec alignement calculé — printpdf ne sait pas aligner.
    ///
    /// Beaucoup d'arguments, mais ce sont ceux d'une primitive de mise en page :
    /// les regrouper dans une structure allongerait les dizaines d'appels sans
    /// rien clarifier.
    #[allow(clippy::too_many_arguments)]
    fn text(&self, content: &str, size: f32, x: f32, y: f32, align: Align, bold: bool, colour: Rgb) {
        let font = if bold { self.bold } else { self.regular };
        let start = match align {
            Align::Left => x,
            Align::Right => x - text_width_mm(content, size, bold),
            Align::Center => x - text_width_mm(content, size, bold) / 2.0,
        };

        self.layer.set_fill_color(colour.to_color());
        self.layer.use_text(content, size, Mm(start), flip(y), font);
    }

    /// Écrit un texte en le repliant dans une largeur maximale.
    #[allow(clippy::too_many_arguments)]
    fn wrapped(&self, content: &str, size: f32, x: f32, y: f32, max_width: f32, bold: bool, colour: Rgb) {
        // 5 mm d'interligne, comme l'espacement des blocs de l'original.
        for (index, line) in wrap(content, max_width, size, bold).iter().enumerate() {
            self.text(line, size, x, y + index as f32 * 5.0, Align::Left, bold, colour);
        }
    }

    fn filled_rect(&self, x: f32, y: f32, width: f32, height: f32, colour: Rgb) {
        self.layer.set_fill_color(colour.to_color());
        self.layer.add_rect(Rect::new(
            Mm(x),
            flip(y + height),
            Mm(x + width),
            flip(y),
        ));
    }

    fn outlined_rect(&self, x: f32, y: f32, width: f32, height: f32, colour: Rgb) {
        self.layer.set_outline_color(colour.to_color());
        self.layer.set_outline_thickness(0.5);
        self.layer.add_rect(
            Rect::new(Mm(x), flip(y + height), Mm(x + width), flip(y))
                .with_mode(path::PaintMode::Stroke),
        );
    }

    fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, colour: Rgb) {
        self.layer.set_outline_color(colour.to_color());
        self.layer.set_outline_thickness(0.5);
        self.layer.add_line(Line {
            points: vec![
                (Point::new(Mm(x1), flip(y1)), false),
                (Point::new(Mm(x2), flip(y2)), false),
            ],
            is_closed: false,
        });
    }
}

/// Montant formaté à deux décimales, suivi du symbole euro.
///
/// Toujours deux décimales, y compris sur un montant rond : `5000,00 €` et non
/// `5000 €`. C'est ce que produisait `.toFixed(2)` côté jsPDF, et ce qu'attend
/// un document comptable.
fn euros(amount: Money) -> String {
    format!("{:.2} €", round_cents(amount))
}

/// Taux de TVA tel qu'affiché : `20%`, `5.5%`, sans zéro superflu.
fn rate(value: Money) -> String {
    format!("{}%", value.normalize())
}

fn type_label(service_type: ServiceType) -> &'static str {
    match service_type {
        ServiceType::ServiceBic => "Serv. Comm.",
        ServiceType::Vente => "Vente",
        ServiceType::ServiceBnc => "Service",
    }
}

/// Hauteur maximale du logo dans le bandeau, en millimètres.
const LOGO_MAX_HEIGHT: f32 = 24.0;

/// Largeur maximale du logo, en millimètres.
const LOGO_MAX_WIDTH: f32 = 60.0;

/// Décode une URL de données en octets d'image.
///
/// Les réglages stockent le logo sous la forme que produit `FileReader` :
/// `data:image/png;base64,iVBOR…`. Rien d'autre n'est accepté — un chemin de
/// fichier ou une URL distante n'aurait pas de sens dans un PDF.
pub fn decode_data_url(url: &str) -> Option<Vec<u8>> {
    use base64::Engine;

    let rest = url.strip_prefix("data:")?;
    let (mime, data) = rest.split_once(",")?;
    if !mime.starts_with("image/") || !mime.ends_with(";base64") {
        return None;
    }

    base64::engine::general_purpose::STANDARD.decode(data).ok()
}

/// Taille d'affichage du logo, en millimètres, et sa position verticale.
///
/// Reprend le calcul de la version jsPDF : hauteur de 24 mm, réduite si la
/// largeur dépasse 60 mm, et centrage vertical dans le bandeau.
pub fn logo_placement(pixels_wide: u32, pixels_high: u32) -> (f32, f32, f32) {
    if pixels_wide == 0 || pixels_high == 0 {
        return (0.0, 0.0, 8.0);
    }

    let ratio = pixels_wide as f32 / pixels_high as f32;
    let mut height = LOGO_MAX_HEIGHT;
    let mut width = ratio * height;

    if width > LOGO_MAX_WIDTH {
        width = LOGO_MAX_WIDTH;
        height = width / ratio;
    }

    let top = 8.0 + (LOGO_MAX_HEIGHT - height) / 2.0;
    (width, height, top)
}

/// Conditions de retard, obligatoires sur une facture entre professionnels.
///
/// Le taux est exprimé en multiple de l'intérêt légal plutôt qu'en pourcentage
/// figé : l'intérêt légal change deux fois par an, et un nombre écrit en dur
/// dans le code serait faux six mois plus tard.
const LATE_PAYMENT_TERMS: &str = concat!(
    "En cas de retard de paiement : pénalités au taux de trois fois l'intérêt légal, ",
    "exigibles sans rappel, et indemnité forfaitaire pour frais de recouvrement de 40 € ",
    "(art. L441-10 et D441-5 du code de commerce). Escompte pour paiement anticipé : néant.",
);

/// Mentions légales du bas de page.
///
/// Le texte est réglementaire et dépend du type de document et du taux de TVA.
/// Le sortir du code de dessin le rend testable — c'est la partie qui expose à
/// un contrôle, pas la position des traits.
pub fn legal_notice(
    kind: DocumentKind,
    tva_rate: Money,
    iban: &str,
    due_date: Option<CivilDate>,
) -> String {
    let mut notice = String::new();

    if kind.is_invoice() {
        if tva_rate.is_zero() {
            notice.push_str("TVA non applicable, article 293 B du CGI.\n");
        }
        notice.push_str(
            "Dispensé d'immatriculation au registre du commerce et des sociétés (RCS) et au répertoire des métiers (RM).\n",
        );
        // Aucun IBAN de gabarit. L'ancienne version en imprimait un —
        // `FR76 0000 …` — qui envoyait le client virer sur un compte
        // inexistant. `Issuer::missing_fields` refuse désormais d'éditer une
        // facture sans IBAN ; si la mention manque malgré tout, elle disparaît
        // au lieu d'être inventée (défaut D11).
        let account = if iban.trim().is_empty() {
            String::new()
        } else {
            format!(" IBAN : {iban}")
        };
        // L'échéance figure ici aussi : le pied de page est l'endroit où se
        // lisent les conditions de règlement, et une facture sans échéance
        // enregistrée conserve la formule générale d'avant.
        match due_date {
            Some(due) => notice.push_str(&format!(
                "Mode de règlement : Virement bancaire. Échéance : {}.{account}\n",
                due.format_fr()
            )),
            None => notice
                .push_str(&format!("Mode de règlement : Virement bancaire sous 30 jours.{account}\n")),
        }
        // Mentions obligatoires entre professionnels : articles L441-10 et
        // D441-5 du code de commerce. Leur absence est sanctionnable, et elles
        // manquaient depuis l'origine.
        notice.push_str(LATE_PAYMENT_TERMS);
    } else if kind == DocumentKind::CreditNote {
        if tva_rate.is_zero() {
            notice.push_str("TVA non applicable, article 293 B du CGI.\n");
        }
        notice.push_str(
            "Dispensé d'immatriculation au registre du commerce et des sociétés (RCS) et au répertoire des métiers (RM).\n",
        );
        // Un avoir ne fait pas naître de créance : ni échéance, ni pénalités.
        // Il annonce ce qu'il corrige, et comment le montant revient au client.
        notice.push_str(
            "Le présent avoir annule ou corrige la facture mentionnée ci-dessus, à concurrence du montant indiqué.\n",
        );
        notice.push_str(
            "Il est imputable sur une prochaine facture ou remboursé par virement, au choix du client.",
        );
    } else {
        notice.push_str("Devis valable pour une durée de 3 mois à compter de la date d'émission.\n");
        notice.push_str(
            "Le commencement des prestations interviendra après acceptation écrite du présent devis.\n",
        );
        notice.push_str(
            "Mention manuscrite obligatoire du client : \"Bon pour accord\", suivie de la date et de sa signature.",
        );
    }

    notice
}

/// Édite le document et renvoie le PDF encodé.
///
/// `generated_at` est la date imprimée en pied de page. Elle est injectée plutôt
/// que lue de l'horloge, pour que les PDF de référence restent reproductibles —
/// la même précaution que celle prise en phase 0 côté JavaScript.
pub fn render(
    kind: DocumentKind,
    document: &Document,
    client: &Party,
    issuer: &Issuer,
    generated_at: CivilDate,
) -> Result<Vec<u8>, printpdf::Error> {
    let (doc, page, layer) = PdfDocument::new(
        &document.number,
        Mm(PAGE_WIDTH),
        Mm(PAGE_HEIGHT),
        "Document",
    );

    let regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let canvas = Canvas {
        layer: doc.get_page(page).get_layer(layer),
        regular: &regular,
        bold: &bold,
    };

    let accent = parse_hex(&issuer.accent_colour);

    // 1. Bandeau de tête
    canvas.filled_rect(0.0, 0.0, PAGE_WIDTH, 40.0, NAVY);

    // Le logo prend la place du titre. Une image illisible ne fait pas échouer
    // l'édition : le titre reparaît, comme le faisait le `catch` du JavaScript.
    let logo = decode_data_url(&issuer.logo)
        .and_then(|bytes| ::image::load_from_memory(&bytes).ok());

    if let Some(picture) = logo {
        use ::image::GenericImageView;
        let (pixels_wide, pixels_high) = picture.dimensions();
        let (width, height, top) = logo_placement(pixels_wide, pixels_high);

        // printpdf place l'image par son coin inférieur gauche, à 300 ppp par
        // défaut : l'échelle ramène ses pixels aux millimètres voulus.
        let scale = |target_mm: f32, pixels: u32| {
            let natural_pt = pixels as f32 * 72.0 / 300.0;
            (target_mm * 72.0 / 25.4) / natural_pt
        };

        printpdf::Image::from_dynamic_image(&picture).add_to_layer(
            doc.get_page(page).get_layer(layer),
            printpdf::ImageTransform {
                translate_x: Some(Mm(20.0)),
                translate_y: Some(layout::flip(top + height)),
                scale_x: Some(scale(width, pixels_wide)),
                scale_y: Some(scale(height, pixels_high)),
                ..Default::default()
            },
        );
    } else {
        canvas.text("ASGARD CRM", 22.0, 20.0, 25.0, Align::Left, true, accent);
        canvas.text(
            "Gestion & Facturation Auto-Entreprise",
            9.0,
            20.0,
            32.0,
            Align::Left,
            false,
            WHITE,
        );
    }

    canvas.text(kind.banner(), 20.0, PAGE_WIDTH - 20.0, 27.0, Align::Right, true, WHITE);

    // 2. Blocs émetteur et destinataire
    let y = 55.0;

    canvas.text("ÉMETTEUR :", 9.0, 20.0, y, Align::Left, true, NAVY);
    canvas.text(&issuer.company_name, 9.0, 20.0, y + 6.0, Align::Left, false, NAVY);
    canvas.text(&issuer.contact_name, 9.0, 20.0, y + 11.0, Align::Left, false, NAVY);
    canvas.wrapped(&issuer.address, 9.0, 20.0, y + 16.0, 75.0, false, NAVY);
    canvas.text(&format!("Tél : {}", issuer.phone), 9.0, 20.0, y + 27.0, Align::Left, false, NAVY);
    canvas.text(&format!("Email : {}", issuer.email), 9.0, 20.0, y + 32.0, Align::Left, false, NAVY);
    if !issuer.siret.is_empty() {
        canvas.text(&format!("SIRET : {}", issuer.siret), 9.0, 20.0, y + 37.0, Align::Left, false, NAVY);
    }

    canvas.text("FACTURÉ À :", 9.0, 110.0, y, Align::Left, true, NAVY);
    canvas.text(&client.company_name, 9.0, 110.0, y + 6.0, Align::Left, false, NAVY);
    canvas.text(&client.contact_name, 9.0, 110.0, y + 11.0, Align::Left, false, NAVY);
    canvas.wrapped(&client.address, 9.0, 110.0, y + 16.0, 80.0, false, NAVY);
    canvas.text(&format!("Tél : {}", client.phone), 9.0, 110.0, y + 27.0, Align::Left, false, NAVY);
    canvas.text(&format!("Email : {}", client.email), 9.0, 110.0, y + 32.0, Align::Left, false, NAVY);

    canvas.text(kind.number_label(), 9.0, PAGE_WIDTH - 20.0, y, Align::Right, true, NAVY);
    canvas.text(&document.number, 9.0, PAGE_WIDTH - 20.0, y + 5.0, Align::Right, false, NAVY);
    canvas.text(
        &format!("Date : {}", document.date.format_fr()),
        9.0,
        PAGE_WIDTH - 20.0,
        y + 12.0,
        Align::Right,
        false,
        NAVY,
    );
    if let Some(invoice) = document.corrects.as_deref() {
        canvas.text(
            &format!("Facture corrigée : {invoice}"),
            9.0,
            PAGE_WIDTH - 20.0,
            y + 18.0,
            Align::Right,
            true,
            NAVY,
        );
    }
    // L'échéance est une mention obligatoire (art. L441-9 du code de commerce),
    // et le client la cherche ici, à côté de la date d'émission.
    if let Some(due) = document.due_date.filter(|_| kind.is_invoice()) {
        canvas.text(
            &format!("Échéance : {}", due.format_fr()),
            9.0,
            PAGE_WIDTH - 20.0,
            y + 18.0,
            Align::Right,
            true,
            NAVY,
        );
    }

    // 3. Tableau de la prestation
    let y = 110.0;
    canvas.filled_rect(20.0, y, PAGE_WIDTH - 40.0, 10.0, TABLE_BG);
    canvas.line(20.0, y, PAGE_WIDTH - 20.0, y, RULE);
    canvas.line(20.0, y + 10.0, PAGE_WIDTH - 20.0, y + 10.0, RULE);

    canvas.text("Description de la prestation", 9.0, 22.0, y + 6.5, Align::Left, true, NAVY);
    canvas.text("Type", 9.0, 95.0, y + 6.5, Align::Left, true, NAVY);
    canvas.text("TVA", 9.0, 125.0, y + 6.5, Align::Right, true, NAVY);
    canvas.text("Montant HT", 9.0, 155.0, y + 6.5, Align::Right, true, NAVY);
    canvas.text("Total TTC", 9.0, PAGE_WIDTH - 22.0, y + 6.5, Align::Right, true, NAVY);

    let y = 120.0;
    let description = if document.description.trim().is_empty() {
        "Prestation de service"
    } else {
        &document.description
    };

    canvas.wrapped(description, 9.0, 22.0, y + 7.0, 65.0, false, NAVY);
    canvas.text(type_label(document.service_type), 9.0, 95.0, y + 7.0, Align::Left, false, NAVY);
    canvas.text(&rate(document.tva_rate), 9.0, 125.0, y + 7.0, Align::Right, false, NAVY);
    canvas.text(&euros(document.amount_ht), 9.0, 155.0, y + 7.0, Align::Right, false, NAVY);
    canvas.text(&euros(document.amount_total), 9.0, PAGE_WIDTH - 22.0, y + 7.0, Align::Right, false, NAVY);
    canvas.line(20.0, y + 15.0, PAGE_WIDTH - 20.0, y + 15.0, RULE);

    // 4. Bloc des totaux
    let y = 145.0;
    canvas.text("Total Hors Taxes (HT) :", 9.0, 135.0, y, Align::Right, false, NAVY);
    canvas.text(&euros(document.amount_ht), 9.0, PAGE_WIDTH - 22.0, y, Align::Right, false, NAVY);
    canvas.text(
        &format!("TVA ({}) :", rate(document.tva_rate)),
        9.0,
        135.0,
        y + 6.0,
        Align::Right,
        false,
        NAVY,
    );
    canvas.text(&euros(document.amount_tva), 9.0, PAGE_WIDTH - 22.0, y + 6.0, Align::Right, false, NAVY);

    canvas.filled_rect(100.0, y + 12.0, PAGE_WIDTH - 120.0, 10.0, NAVY);
    canvas.text(kind.total_label(), 10.0, 135.0, y + 18.5, Align::Right, true, WHITE);
    canvas.text(&euros(document.amount_total), 10.0, PAGE_WIDTH - 22.0, y + 18.5, Align::Right, true, accent);

    // 5. Mentions légales
    let y = 190.0;
    canvas.text("MENTIONS LÉGALES & CONDITIONS", 8.0, 20.0, y, Align::Left, true, NAVY);

    let notice = legal_notice(kind, document.tva_rate, &issuer.iban, document.due_date);
    // Le devis réserve la moitié droite au cadre de signature.
    let max_width = if kind.is_binding() { PAGE_WIDTH - 40.0 } else { 95.0 };

    let mut line_y = y + 5.0;
    for paragraph in notice.split('\n') {
        for line in wrap(paragraph, max_width, 8.0, false) {
            canvas.text(&line, 8.0, 20.0, line_y, Align::Left, false, NAVY);
            line_y += 4.0;
        }
    }

    // Cadre de signature, pour les devis seulement.
    if !kind.is_binding() {
        canvas.text(
            "Cadre Signature Client (Bon pour accord) :",
            7.0,
            122.0,
            y + 23.0,
            Align::Left,
            false,
            GREY,
        );
        canvas.outlined_rect(120.0, y + 25.0, 70.0, 22.0, Rgb(200, 200, 200));
    }

    // 6. Pied de page
    canvas.line(20.0, 260.0, PAGE_WIDTH - 20.0, 260.0, GREY);
    let footer = format!(
        "{} généré automatiquement via Asgard CRM le {}",
        kind.footer_noun(),
        generated_at.format_fr()
    );
    canvas.text(&footer, 7.0, PAGE_WIDTH / 2.0, 267.0, Align::Center, false, GREY);

    doc.save_to_bytes()
}

#[cfg(test)]
mod tests;
