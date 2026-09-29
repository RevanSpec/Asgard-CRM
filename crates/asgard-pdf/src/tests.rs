//! Comparaison avec les PDF de référence de la phase 0.
//!
//! `fixtures/pdf-reference/` contient les 22 documents produits par jsPDF sur le
//! jeu de référence, archivés précisément pour servir de cible à ce portage.
//!
//! **Ce qui est comparé, et ce qui ne l'est pas.** Une comparaison octet à octet
//! entre deux générateurs n'a aucun sens : structure d'objets, ordre du flux,
//! compression, tout diffère. Ce qui doit être identique, c'est ce que le
//! lecteur voit — le texte imprimé, dans l'ordre. C'est donc lui qu'on extrait
//! des deux côtés.
//!
//! Le positionnement, lui, se lit dans le code : les coordonnées sont reprises
//! telles quelles de `pdfGenerator.js`, et `layout.rs` se charge de retourner
//! l'axe vertical.

use super::*;
use asgard_core::model::CivilDate;
use rust_decimal_macros::dec;

/// Date d'édition figée, celle des PDF archivés en phase 0.
fn generated_at() -> CivilDate {
    CivilDate::new(2026, 9, 15)
}

fn issuer() -> Issuer {
    Issuer {
        company_name: "Asgard Solutions".into(),
        contact_name: "Thor Odinson".into(),
        address: "1 rue du Valhalla, 75008 Paris".into(),
        phone: "06 12 34 56 78".into(),
        email: "thor@asgard-solutions.fr".into(),
        siret: "839 204 123 00019".into(),
        iban: "FR76 3000 2000 0001 2345 6789 012".into(),
        accent_colour: "#E5A93C".into(),
    }
}

fn coffee_client() -> Party {
    Party {
        company_name: "Asgard Coffee & Co".into(),
        contact_name: "Valkyrie".into(),
        address: "45 rue du Bifrost, 75011 Paris".into(),
        phone: "06 55 55 55 55".into(),
        email: "valk@coffee.asgard".into(),
    }
}

/// La facture 3 du jeu de référence : vente de marchandises à 5,5 %.
fn coffee_invoice() -> Document {
    Document {
        number: "FAC-ASGARDCOFF-2026-0003".into(),
        service_type: ServiceType::Vente,
        description: "Grains de café d'Éthiopie, 40 kg".into(),
        amount_ht: dec!(1450.35),
        tva_rate: dec!(5.5),
        amount_tva: dec!(79.77),
        amount_total: dec!(1530.12),
        due_date: Some(CivilDate::new(2026, 5, 18)),
        corrects: None,
        date: CivilDate::new(2026, 4, 18),
    }
}

/// Extrait les fragments de texte d'un PDF, dans l'ordre du flux.
///
/// Les deux générateurs écrivent leurs chaînes avec l'opérateur `Tj`, mais pas
/// sous la même forme : jsPDF produit des littéraux `(texte) Tj`, printpdf des
/// chaînes hexadécimales `<4153...> Tj`. Les deux encodages sont du WinAnsi —
/// l'encodage des polices PDF standard, qui couvre le latin-1 et donc les
/// accents français. La fonction gère les deux et renvoie de l'UTF-8.
/// Le parcours se fait sur les **octets bruts**, et le décodage n'intervient
/// qu'une fois un fragment isolé. Décoder d'abord serait une erreur : le WinAnsi
/// produit de l'UTF-8 multi-octets, et les indices calculés sur la chaîne
/// décodée ne correspondraient plus aux positions du fichier.
fn extract_text(pdf: &[u8]) -> Vec<String> {
    let mut fragments = Vec::new();
    let mut index = 0;

    while index < pdf.len() {
        let (payload, after) = match pdf[index] {
            b'(' => match find_unescaped_close(&pdf[index + 1..]) {
                Some(close) => (
                    unescape(&pdf[index + 1..index + 1 + close]),
                    index + close + 2,
                ),
                None => break,
            },
            b'<' => match pdf[index + 1..].iter().position(|b| *b == b'>') {
                Some(close) => (
                    from_hex(&pdf[index + 1..index + 1 + close]),
                    index + close + 2,
                ),
                None => break,
            },
            _ => {
                index += 1;
                continue;
            }
        };

        let follows_tj = pdf[after..]
            .iter()
            .position(|b| !b.is_ascii_whitespace())
            .is_some_and(|offset| pdf[after + offset..].starts_with(b"Tj"));

        if follows_tj && !payload.is_empty() {
            fragments.push(win_ansi(&payload));
        }
        index = after;
    }

    fragments
}

/// Décode du WinAnsi, l'encodage des polices PDF standard.
///
/// WinAnsi coïncide avec le latin-1 partout **sauf** dans la plage 0x80–0x9F,
/// que le latin-1 réserve à des caractères de contrôle et que WinAnsi remplit
/// de signes typographiques — dont le **symbole euro, à 0x80**. Décoder
/// naïvement en latin-1 transforme donc chaque « € » en caractère de contrôle,
/// ce qui suffit à faire échouer une comparaison de montants.
fn win_ansi(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| match b {
            0x80 => '€',
            0x82 => '‚',
            0x83 => 'ƒ',
            0x84 => '„',
            0x85 => '…',
            0x86 => '†',
            0x87 => '‡',
            0x88 => 'ˆ',
            0x89 => '‰',
            0x8A => 'Š',
            0x8B => '‹',
            0x8C => 'Œ',
            0x8E => 'Ž',
            0x91 => '\u{2018}',
            0x92 => '\u{2019}',
            0x93 => '\u{201C}',
            0x94 => '\u{201D}',
            0x95 => '•',
            0x96 => '–',
            0x97 => '—',
            0x98 => '˜',
            0x99 => '™',
            0x9A => 'š',
            0x9B => '›',
            0x9C => 'œ',
            0x9E => 'ž',
            0x9F => 'Ÿ',
            other => *other as char,
        })
        .collect()
}

/// Décode une chaîne hexadécimale PDF en octets.
fn from_hex(bytes: &[u8]) -> Vec<u8> {
    let digits: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(u8::is_ascii_hexdigit)
        .collect();

    digits
        .chunks(2)
        .filter(|pair| pair.len() == 2)
        .filter_map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some((high * 16 + low) as u8)
        })
        .collect()
}

/// Trouve la parenthèse fermante d'un littéral PDF, en sautant les échappées.
fn find_unescaped_close(bytes: &[u8]) -> Option<usize> {
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b')' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

/// Retire les échappements d'un littéral PDF.
fn unescape(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 1 < bytes.len() {
            out.push(bytes[index + 1]);
            index += 2;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }

    out
}

fn reference(name: &str) -> Vec<u8> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/pdf-reference/");
    std::fs::read(format!("{path}{name}"))
        .unwrap_or_else(|e| panic!("PDF de référence {name} illisible : {e}"))
}

#[test]
fn produces_a_valid_pdf() {
    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    assert!(pdf.starts_with(b"%PDF-"), "en-tête PDF manquant");
    assert!(
        String::from_utf8_lossy(&pdf).trim_end().ends_with("%%EOF"),
        "marqueur de fin manquant"
    );
}

/// Le test central de la phase : tout ce que jsPDF imprimait, Rust l'imprime.
#[test]
fn prints_everything_the_javascript_version_printed() {
    let ours = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let theirs = reference("FAC-ASGARDCOFF-2026-0003.pdf");

    let ours = extract_text(&ours).join(" | ");
    let expected = extract_text(&theirs);

    assert!(!expected.is_empty(), "la référence doit contenir du texte");

    for fragment in &expected {
        if REPLACED_BY_A_DATED_DUE_DATE.iter().any(|old| fragment.starts_with(old)) {
            continue;
        }
        assert!(
            ours.contains(fragment.as_str()),
            "fragment absent du PDF produit : {fragment:?}\n---\nproduit : {ours}"
        );
    }

    // L'écart est unique, et il ajoute : la phrase annonçait un délai sans
    // jamais nommer de date. Ce qu'elle portait — virement, IBAN — se retrouve
    // dans la nouvelle, avec l'échéance en plus.
    assert!(ours.contains("Mode de règlement : Virement bancaire. Échéance : 18/05/2026."));
    assert!(ours.contains("IBAN : FR76 3000 2000 0001 2345 6789 012"));
}

/// Seule phrase du PDF de référence que le port ne reproduit plus mot pour mot.
const REPLACED_BY_A_DATED_DUE_DATE: [&str; 1] =
    ["Mode de règlement : Virement bancaire sous 30 jours."];

#[test]
fn prints_the_estimate_signature_block() {
    let mut document = coffee_invoice();
    document.number = "DEV-ASGARDCOFF-2026-0004".into();

    let pdf = render(
        DocumentKind::Estimate,
        &document,
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");

    assert!(text.contains("DEVIS"));
    assert!(text.contains("N° Devis :"));
    assert!(text.contains("Cadre Signature Client (Bon pour accord) :"));
    assert!(text.contains("Bon pour accord"));
    assert!(!text.contains("Mode de règlement"), "un devis n'annonce pas de règlement");
}

#[test]
fn an_exempt_invoice_carries_the_cgi_notice() {
    let mut document = coffee_invoice();
    document.tva_rate = dec!(0);
    document.amount_tva = dec!(0);
    document.amount_total = document.amount_ht;

    let pdf = render(
        DocumentKind::Invoice,
        &document,
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");
    assert!(text.contains("TVA non applicable, article 293 B du CGI."));
}

#[test]
fn a_taxed_invoice_omits_the_cgi_notice() {
    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");
    assert!(!text.contains("293 B"));
}

#[test]
fn amounts_always_carry_two_decimals() {
    let mut document = coffee_invoice();
    document.amount_ht = dec!(5000);
    document.tva_rate = dec!(20);
    document.amount_tva = dec!(1000);
    document.amount_total = dec!(6000);

    let pdf = render(
        DocumentKind::Invoice,
        &document,
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");

    // Un montant rond s'imprime tout de même avec ses centimes.
    assert!(text.contains("5000.00 €"), "montants imprimés : {text}");
    assert!(text.contains("6000.00 €"));
}

#[test]
fn legal_notices_match_the_javascript_wording() {
    let due = Some(CivilDate::new(2026, 5, 18));
    let invoice = legal_notice(DocumentKind::Invoice, dec!(20), "FR76 1234", due);
    assert!(invoice.contains("Dispensé d'immatriculation au registre du commerce"));
    assert!(invoice.contains("IBAN : FR76 1234"));
    assert!(!invoice.contains("293 B"));

    let exempt = legal_notice(DocumentKind::Invoice, dec!(0), "", due);
    assert!(exempt.starts_with("TVA non applicable, article 293 B du CGI."));
    assert!(
        exempt.contains("FR76 0000 0000 0000 0000 0000 000"),
        "un IBAN absent retombe sur le gabarit, comme en JavaScript"
    );

    let estimate = legal_notice(DocumentKind::Estimate, dec!(20), "FR76 1234", None);
    assert!(estimate.contains("valable pour une durée de 3 mois"));
    assert!(estimate.contains("Bon pour accord"));
}

/// Mentions obligatoires entre professionnels. Elles manquaient depuis
/// l'origine : la version JavaScript ne les imprimait pas non plus, et leur
/// absence est sanctionnable.
#[test]
fn an_invoice_carries_the_mandatory_late_payment_terms() {
    let notice = legal_notice(
        DocumentKind::Invoice,
        dec!(20),
        "FR76 1234",
        Some(CivilDate::new(2026, 5, 18)),
    );

    assert!(notice.contains("Échéance : 18/05/2026"));
    assert!(notice.contains("trois fois l'intérêt légal"));
    assert!(notice.contains("indemnité forfaitaire pour frais de recouvrement de 40 €"));
    assert!(notice.contains("Escompte pour paiement anticipé : néant"));

    // Un devis n'annonce ni échéance ni pénalités : il ne fait pas naître de
    // créance.
    let estimate = legal_notice(DocumentKind::Estimate, dec!(20), "FR76 1234", None);
    assert!(!estimate.contains("40 €"));
    assert!(!estimate.contains("Échéance"));
}

/// Une facture d'avant l'enregistrement des échéances conserve la formule
/// générale : inventer une date sur une pièce déjà émise serait pire.
#[test]
fn an_invoice_without_a_recorded_due_date_keeps_the_former_wording() {
    let notice = legal_notice(DocumentKind::Invoice, dec!(20), "FR76 1234", None);

    assert!(notice.contains("Virement bancaire sous 30 jours"));
    assert!(!notice.contains("Échéance :"));
    // Les pénalités, elles, ne dépendent pas de l'échéance.
    assert!(notice.contains("40 €"));
}

#[test]
fn the_due_date_is_printed_next_to_the_issue_date() {
    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");
    assert!(text.contains("Échéance : 18/05/2026"), "texte produit : {text}");
}

/// Un avoir est une pièce à part : son cartouche, son total et ses mentions
/// diffèrent d'une facture, et il nomme la facture qu'il corrige.
#[test]
fn a_credit_note_names_the_invoice_it_corrects() {
    let mut document = coffee_invoice();
    document.number = "AVO-ASGARDCOFF-2026-0001".into();
    document.corrects = Some("FAC-ASGARDCOFF-2026-0003".into());
    document.due_date = None;

    let pdf = render(
        DocumentKind::CreditNote,
        &document,
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");

    assert!(text.contains("AVOIR"));
    assert!(text.contains("N° Avoir :"));
    assert!(text.contains("AVO-ASGARDCOFF-2026-0001"));
    assert!(text.contains("Facture corrigée : FAC-ASGARDCOFF-2026-0003"));
    // Un avoir est dû au client : il n'est pas « à payer ».
    assert!(text.contains("TOTAL AVOIR (TTC) :"));
    assert!(!text.contains("TOTAL NET À PAYER"));
    // Ni créance, ni signature : pas de pénalités, pas de cadre.
    assert!(!text.contains("40 €"));
    assert!(!text.contains("Échéance"));
    assert!(!text.contains("Bon pour accord"));
    assert!(text.contains("annule ou corrige la facture"));
}

#[test]
fn a_credit_note_carries_the_cgi_notice_when_exempt() {
    let exempt = legal_notice(DocumentKind::CreditNote, dec!(0), "FR76 1234", None);
    assert!(exempt.starts_with("TVA non applicable, article 293 B du CGI."));

    let taxed = legal_notice(DocumentKind::CreditNote, dec!(20), "FR76 1234", None);
    assert!(!taxed.contains("293 B"));
}

#[test]
fn activity_labels_match_the_javascript_table() {
    assert_eq!(type_label(ServiceType::ServiceBnc), "Service");
    assert_eq!(type_label(ServiceType::ServiceBic), "Serv. Comm.");
    assert_eq!(type_label(ServiceType::Vente), "Vente");
}

#[test]
fn tva_rates_print_without_trailing_zeros() {
    assert_eq!(rate(dec!(20)), "20%");
    assert_eq!(rate(dec!(20.00)), "20%");
    assert_eq!(rate(dec!(5.5)), "5.5%");
    assert_eq!(rate(dec!(0)), "0%");
}

#[test]
fn an_empty_description_falls_back_like_the_original() {
    let mut document = coffee_invoice();
    document.description = "   ".into();

    let pdf = render(
        DocumentKind::Invoice,
        &document,
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();

    assert!(extract_text(&pdf).join(" | ").contains("Prestation de service"));
}

/// L'édition doit être reproductible : deux appels identiques produisent le même
/// document. C'est la précaution prise en phase 0 côté JavaScript, où la date du
/// jour et un identifiant aléatoire rendaient la référence instable.
#[test]
fn rendering_is_reproducible() {
    let make = || {
        render(
            DocumentKind::Invoice,
            &coffee_invoice(),
            &coffee_client(),
            &issuer(),
            generated_at(),
        )
        .unwrap()
    };

    assert_eq!(extract_text(&make()), extract_text(&make()));
}

#[test]
fn the_footer_carries_the_injected_date() {
    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer(),
        CivilDate::new(2027, 1, 31),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");
    assert!(text.contains("31/01/2027"), "pied de page : {text}");
    assert!(text.contains("Facture généré automatiquement via Asgard CRM"));
}

/// Un émetteur sans SIRET ne doit pas imprimer une ligne vide.
#[test]
fn an_empty_siret_prints_no_line() {
    let mut issuer = issuer();
    issuer.siret = String::new();

    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer,
        generated_at(),
    )
    .unwrap();

    assert!(!extract_text(&pdf).join(" | ").contains("SIRET :"));
}
