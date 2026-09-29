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
        logo: String::new(),
        mediator: String::new(),
        insurance: String::new(),
        vat_on_debits: false,
    }
}

fn coffee_client() -> Party {
    Party {
        company_name: "Asgard Coffee & Co".into(),
        contact_name: "Valkyrie".into(),
        address: "45 rue du Bifrost, 75011 Paris".into(),
        phone: "06 55 55 55 55".into(),
        email: "valk@coffee.asgard".into(),
        siren: String::new(),
        vat_number: String::new(),
        delivery_address: String::new(),
    }
}

/// Mentions légales d'une pièce, à partir du jeu de référence.
///
/// `legal_notice` reçoit désormais la pièce, l'émetteur et le client : les
/// mentions ajoutées par le décret n° 2022-1299 viennent des trois. Ce
/// raccourci garde les appels d'origine lisibles ; les cas nouveaux appellent
/// la fonction directement, avec les parties qu'ils veulent éprouver.
fn notice_for(
    kind: DocumentKind,
    tva_rate: Money,
    iban: &str,
    due: Option<CivilDate>,
) -> String {
    let document = Document { tva_rate, due_date: due, ..coffee_invoice() };
    let issuer = Issuer { iban: iban.into(), ..issuer() };
    legal_notice(kind, &document, &issuer, &coffee_client())
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
        operation: None,
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
    let invoice = notice_for(DocumentKind::Invoice, dec!(20), "FR76 1234", due);
    assert!(invoice.contains("Dispensé d'immatriculation au registre du commerce"));
    assert!(invoice.contains("IBAN : FR76 1234"));
    assert!(!invoice.contains("293 B"));

    let exempt = notice_for(DocumentKind::Invoice, dec!(0), "", due);
    assert!(exempt.starts_with("TVA non applicable, article 293 B du CGI."));
    // Écart assumé avec le JavaScript, et seul écart de cette fonction :
    // l'original imprimait `FR76 0000 0000 0000 0000 0000 000` quand l'IBAN
    // manquait. Le client était alors invité à virer sur un compte qui
    // n'existe pas. La mention disparaît au lieu d'être inventée.
    assert!(
        !exempt.contains("FR76 0000"),
        "aucun IBAN de gabarit ne doit être imprimé : {exempt}"
    );
    assert!(!exempt.contains("IBAN"), "pas de mention d'IBAN sans IBAN : {exempt}");
    assert!(exempt.contains("Mode de règlement : Virement bancaire. Échéance : 18/05/2026."));

    let estimate = notice_for(DocumentKind::Estimate, dec!(20), "FR76 1234", None);
    assert!(estimate.contains("valable pour une durée de 3 mois"));
    assert!(estimate.contains("Bon pour accord"));
}

/// Défaut D11 : aucune pièce ne doit sortir avec une identité que l'émetteur
/// n'a pas saisie. Le garde est ici plutôt que dans l'hôte pour être testable
/// sans base de données, et pour valoir quel que soit l'appelant.
#[test]
fn an_issuer_without_identity_cannot_edit_a_document() {
    assert!(issuer().missing_fields(DocumentKind::Invoice).is_empty());

    let nothing = Issuer::default();
    assert_eq!(
        nothing.missing_fields(DocumentKind::Invoice),
        vec!["la raison sociale", "l'adresse", "le SIRET", "l'IBAN"]
    );

    // Un devis ne demande pas de règlement : l'IBAN n'y est pas exigé, et il
    // n'y est pas imprimé non plus.
    assert_eq!(
        nothing.missing_fields(DocumentKind::Estimate),
        vec!["la raison sociale", "l'adresse", "le SIRET"]
    );
    // Un avoir non plus : il rend de l'argent, il n'en réclame pas.
    assert!(!nothing.missing_fields(DocumentKind::CreditNote).contains(&"l'IBAN"));

    // Des espaces ne remplissent rien.
    let blank = Issuer { siret: "   ".into(), ..issuer() };
    assert_eq!(blank.missing_fields(DocumentKind::Invoice), vec!["le SIRET"]);
}

/// Mentions obligatoires entre professionnels. Elles manquaient depuis
/// l'origine : la version JavaScript ne les imprimait pas non plus, et leur
/// absence est sanctionnable.
#[test]
fn an_invoice_carries_the_mandatory_late_payment_terms() {
    let notice = notice_for(DocumentKind::Invoice,
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
    let estimate = notice_for(DocumentKind::Estimate, dec!(20), "FR76 1234", None);
    assert!(!estimate.contains("40 €"));
    assert!(!estimate.contains("Échéance"));
}

/// Une facture d'avant l'enregistrement des échéances conserve la formule
/// générale : inventer une date sur une pièce déjà émise serait pire.
#[test]
fn an_invoice_without_a_recorded_due_date_keeps_the_former_wording() {
    let notice = notice_for(DocumentKind::Invoice, dec!(20), "FR76 1234", None);

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
    let exempt = notice_for(DocumentKind::CreditNote, dec!(0), "FR76 1234", None);
    assert!(exempt.starts_with("TVA non applicable, article 293 B du CGI."));

    let taxed = notice_for(DocumentKind::CreditNote, dec!(20), "FR76 1234", None);
    assert!(!taxed.contains("293 B"));
}

/// PNG 2×2 aux couleurs de la marque, produit à la main : le plus petit
/// fichier qui permette d'éprouver le décodage et le placement.
const TINY_PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAIAAAD91JpzAAAAEElEQVR4nGN4utIGiBggFAA0ygcp9kDWlAAAAABJRU5ErkJggg==";

#[test]
fn a_data_url_decodes_to_image_bytes() {
    let bytes = decode_data_url(TINY_PNG).expect("URL de données lisible");
    assert!(bytes.starts_with(b"\x89PNG"), "les octets doivent être ceux d'un PNG");
}

/// Seules les URL de données d'image sont acceptées : un chemin de fichier ou
/// une adresse distante n'a pas de sens dans un PDF.
#[test]
fn other_urls_are_refused() {
    for url in [
        "",
        "C:/logos/asgard.png",
        "https://exemple.fr/logo.png",
        "data:text/plain;base64,Qm9uam91cg==",
        "data:image/png,pas-du-base64",
        "data:image/png;base64,???",
    ] {
        assert!(decode_data_url(url).is_none(), "{url:?} ne devrait pas être accepté");
    }
}

/// Placement repris de la version jsPDF : 24 mm de haut, 60 mm de large au
/// plus, rapport conservé, centré verticalement dans le bandeau.
#[test]
fn the_logo_keeps_its_ratio_within_the_banner() {
    // Image carrée : la hauteur commande.
    let (width, height, top) = logo_placement(100, 100);
    assert_eq!((width, height), (24.0, 24.0));
    assert_eq!(top, 8.0, "centré dans les 24 mm du bandeau");

    // Image très large : la largeur commande, et le logo se centre.
    let (width, height, top) = logo_placement(600, 100);
    assert_eq!(width, 60.0);
    assert_eq!(height, 10.0);
    assert_eq!(top, 15.0);

    // Une image dégénérée ne fait pas diviser par zéro.
    assert_eq!(logo_placement(0, 0), (0.0, 0.0, 8.0));
}

/// Le logo prend la place du titre, comme dans l'original.
#[test]
fn a_logo_replaces_the_application_title() {
    let with_logo = Issuer { logo: TINY_PNG.into(), ..issuer() };

    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &with_logo,
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");
    assert!(!text.contains("ASGARD CRM"), "le titre cède la place au logo : {text}");
    assert!(!text.contains("Gestion & Facturation"));
    // Le reste de la pièce ne bouge pas.
    assert!(text.contains("FACTURE"));
    assert!(text.contains("FAC-ASGARDCOFF-2026-0003"));
}

/// Une image illisible ne fait pas échouer l'édition : le titre reparaît,
/// comme le faisait le `catch` du JavaScript.
#[test]
fn an_unreadable_logo_falls_back_to_the_title() {
    let broken = Issuer {
        logo: "data:image/png;base64,Qm9uam91cg==".into(),
        ..issuer()
    };

    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &broken,
        generated_at(),
    )
    .unwrap();

    let text = extract_text(&pdf).join(" | ");
    assert!(text.contains("ASGARD CRM"));
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

// ------------------------------------------- mentions du décret n° 2022-1299

/// La nature de l'opération est obligatoire, et elle se déduit du type
/// d'activité tant qu'elle n'a pas été saisie : une pièce émise avant qu'on ne
/// la demande ne doit pas perdre la mention pour autant.
#[test]
fn the_notice_states_what_the_invoice_covers() {
    // Le jeu de référence facture du café : une livraison de biens.
    let goods = notice_for(DocumentKind::Invoice, dec!(20), "FR76 1234", None);
    assert!(
        goods.contains("Opération portant exclusivement sur des livraisons de biens."),
        "{goods}"
    );

    let services = Document {
        service_type: ServiceType::ServiceBnc,
        ..coffee_invoice()
    };
    let notice = legal_notice(DocumentKind::Invoice, &services, &issuer(), &coffee_client());
    assert!(notice.contains("Opération portant exclusivement sur des prestations de services."));

    // Le cas mixte ne se déduit d'aucun type : il ne vient que de la saisie.
    let mixed = Document { operation: Some(Operation::Both), ..coffee_invoice() };
    let notice = legal_notice(DocumentKind::Invoice, &mixed, &issuer(), &coffee_client());
    assert!(notice
        .contains("Opération portant sur des livraisons de biens et des prestations de services."));

    // Ce qui est enregistré l'emporte sur ce que le type laisserait attendre.
    let stored = Document { operation: Some(Operation::Services), ..coffee_invoice() };
    let notice = legal_notice(DocumentKind::Invoice, &stored, &issuer(), &coffee_client());
    assert!(notice.contains("exclusivement sur des prestations de services."));
}

/// L'adresse de livraison n'est exigée que lorsqu'elle diffère : la répéter à
/// l'identique n'apprendrait rien au lecteur.
#[test]
fn a_delivery_address_is_printed_only_when_it_differs() {
    let elsewhere = Party {
        delivery_address: "7 quai de Nidavellir, 29200 Brest".into(),
        ..coffee_client()
    };
    let notice = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &issuer(), &elsewhere);
    assert!(notice.contains("Livraison à : 7 quai de Nidavellir, 29200 Brest."), "{notice}");

    let same = Party {
        delivery_address: coffee_client().address.clone(),
        ..coffee_client()
    };
    let notice = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &issuer(), &same);
    assert!(!notice.contains("Livraison à"), "{notice}");

    // Rien de saisi, rien d'imprimé.
    let notice = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &issuer(), &coffee_client());
    assert!(!notice.contains("Livraison à"));
}

/// L'option pour les débits ne concerne que qui facture de la TVA.
#[test]
fn the_debits_option_follows_the_vat() {
    let opted = Issuer { vat_on_debits: true, ..issuer() };

    let taxed = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &opted, &coffee_client());
    assert!(taxed.contains("Option pour le paiement de la TVA d'après les débits."), "{taxed}");

    let exempt = Document { tva_rate: dec!(0), ..coffee_invoice() };
    let notice = legal_notice(DocumentKind::Invoice, &exempt, &opted, &coffee_client());
    assert!(!notice.contains("d'après les débits"), "{notice}");

    // Sans option exercée, aucune mention — elle serait fausse.
    let notice = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &issuer(), &coffee_client());
    assert!(!notice.contains("d'après les débits"));
}

/// Le médiateur et l'assurance dépendent de l'activité : ils ne s'impriment que
/// s'ils sont renseignés, mais alors sur toutes les pièces — l'assurance figure
/// sur les devis autant que sur les factures (art. L112-11 du code des
/// assurances).
#[test]
fn activity_mentions_appear_on_every_document_when_filled() {
    let declared = Issuer {
        mediator: "Médiation Nord, mediation-nord.fr".into(),
        insurance: "Assurances du Valhalla, RC pro, France entière".into(),
        ..issuer()
    };

    for kind in [DocumentKind::Invoice, DocumentKind::Estimate, DocumentKind::CreditNote] {
        let notice = legal_notice(kind, &coffee_invoice(), &declared, &coffee_client());
        assert!(
            notice.contains("Médiateur de la consommation : Médiation Nord, mediation-nord.fr"),
            "{kind:?} : {notice}"
        );
        assert!(
            notice.contains(
                "Assurance professionnelle : Assurances du Valhalla, RC pro, France entière"
            ),
            "{kind:?} : {notice}"
        );
    }

    // Non renseignés, ils ne laissent pas d'étiquette vide derrière eux.
    let notice = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &issuer(), &coffee_client());
    assert!(!notice.contains("Médiateur"), "{notice}");
    assert!(!notice.contains("Assurance"), "{notice}");
}

/// Un avoir corrige une facture : il en porte les mentions, faute de quoi la
/// pièce rectificative en dirait moins que celle qu'elle rectifie.
#[test]
fn a_credit_note_carries_the_same_operation_mentions() {
    let notice = notice_for(DocumentKind::CreditNote, dec!(20), "FR76 1234", None);

    assert!(notice.contains("Opération portant exclusivement sur des livraisons de biens."));
    // Sans pour autant réclamer un règlement : ce n'est pas son objet.
    assert!(!notice.contains("Échéance"));
    assert!(!notice.contains("40 €"));
}

/// Un devis annonce ce qu'il engage, pas les mentions d'une facture : la nature
/// de l'opération et l'adresse de livraison n'y figurent pas.
#[test]
fn an_estimate_keeps_its_own_mentions() {
    let notice = notice_for(DocumentKind::Estimate, dec!(20), "FR76 1234", None);

    assert!(notice.contains("valable pour une durée de 3 mois"));
    assert!(!notice.contains("Opération portant"), "{notice}");
}

/// Le SIREN du client s'imprime sous ses coordonnées, et le numéro de TVA
/// remonte quand le SIREN manque.
#[test]
fn the_client_block_carries_its_identifiers() {
    let identified = Party {
        siren: "552 100 554".into(),
        vat_number: "FR 12 552100554".into(),
        ..coffee_client()
    };

    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &identified,
        &issuer(),
        generated_at(),
    )
    .unwrap();
    let text = extract_text(&pdf).join(" | ");

    assert!(text.contains("SIREN : 552 100 554"), "{text}");
    assert!(text.contains("N° TVA : FR 12 552100554"));

    // Un client sans identifiants n'affiche pas d'étiquette vide.
    let pdf = render(
        DocumentKind::Invoice,
        &coffee_invoice(),
        &coffee_client(),
        &issuer(),
        generated_at(),
    )
    .unwrap();
    let text = extract_text(&pdf).join(" | ");
    assert!(!text.contains("SIREN"), "{text}");
    assert!(!text.contains("N° TVA"));
}

/// Les mentions n'ont cessé de grandir : l'échéance, les pénalités, puis le
/// décret. Elles s'impriment entre 195 mm et le trait du pied de page, à
/// 260 mm — soit seize lignes. Ce test dit ce qu'il en reste, et échouera avant
/// que la page ne déborde.
#[test]
fn the_legal_notice_fits_above_the_footer() {
    let loaded = Issuer {
        mediator: "Médiation de la consommation Nord-Ouest, www.mediation-nord-ouest.fr".into(),
        insurance: "Assurances du Valhalla, RC professionnelle n° 12 345 678, France entière".into(),
        vat_on_debits: true,
        ..issuer()
    };
    let client = Party {
        siren: "552 100 554".into(),
        vat_number: "FR 12 552100554".into(),
        delivery_address: "7 quai de Nidavellir, bâtiment C, 29200 Brest".into(),
        ..coffee_client()
    };

    let notice = legal_notice(DocumentKind::Invoice, &coffee_invoice(), &loaded, &client);
    let lines: usize = notice
        .split('\n')
        .map(|paragraph| wrap(paragraph, PAGE_WIDTH - 40.0, 8.0, false).len())
        .sum();

    // Première ligne à 195 mm, une ligne tous les 4 mm.
    let bottom = 195.0 + 4.0 * lines as f64;
    assert!(
        bottom <= 260.0,
        "{lines} lignes de mentions descendent jusqu'à {bottom} mm, sous le pied de page"
    );
}
