//! Validation des formulaires — port de `src/domain/validation.js`.
//!
//! Elle reste côté interface : elle doit répondre à la frappe, sans aller-retour
//! vers l'hôte, et ses messages sont du texte d'interface.
//!
//! Les expressions régulières du JavaScript sont réécrites à la main plutôt que
//! confiées au crate `regex`, qui ajouterait plusieurs centaines de kilo-octets
//! au WebAssembly pour deux motifs simples.
//!
//! **Un défaut corrigé au passage.** `parseFloat('1899,99')` valait `1899` : une
//! virgule décimale — ce qu'un utilisateur français tape naturellement —
//! tronquait les centimes en silence. Le test de la phase 0 qui le documentait
//! est réécrit ci-dessous plutôt que supprimé, comme la phase 0 l'exigeait.

/// Nombre de chiffres d'un numéro de téléphone français.
pub const PHONE_DIGITS: usize = 10;

/// Types de voie reconnus. Liste reprise de l'original.
///
/// ⚠️ Restrictive : « 12 bis rue de Paris », « 5 cours Mirabeau » ou un lieu-dit
/// sont refusés alors qu'ils sont valides. C'est un choix de produit — quelles
/// voies accepter — et il n'est pas tranché ici : le comportement est reproduit
/// tel quel, et un test le documente.
const STREET_TYPES: [&str; 13] = [
    "rue", "boulevard", "bd", "avenue", "av", "place", "impasse", "route", "chemin", "allée",
    "voie", "square", "quai",
];

/// Adresse e-mail : `local@domaine.tld`, TLD d'au moins deux lettres.
///
/// Équivalent de `/^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/`.
pub fn is_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.contains('@') {
        return false;
    }

    let local_ok = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "._%+-".contains(c));

    let Some((host, tld)) = domain.rsplit_once('.') else {
        return false;
    };

    let host_ok = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-".contains(c));

    let tld_ok = tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic());

    local_ok && host_ok && tld_ok
}

/// Adresse française : 1 à 4 chiffres, un type de voie reconnu, un libellé.
pub fn is_address(value: &str) -> bool {
    let mut words = value.split_whitespace();

    let number_ok = words
        .next()
        .is_some_and(|n| (1..=4).contains(&n.len()) && n.chars().all(|c| c.is_ascii_digit()));

    let street_ok = words
        .next()
        .is_some_and(|s| STREET_TYPES.contains(&s.to_lowercase().as_str()));

    let label_ok = words.next().is_some();

    number_ok && street_ok && label_ok
}

/// Formate une saisie de téléphone en « xx xx xx xx xx ».
pub fn format_phone(raw: &str) -> String {
    let digits: Vec<char> = raw
        .chars()
        .filter(char::is_ascii_digit)
        .take(PHONE_DIGITS)
        .collect();

    digits
        .chunks(2)
        .map(|pair| pair.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lit un montant saisi, **virgule ou point**.
///
/// Correction du défaut documenté en phase 0 : `parseFloat('1899,99')` renvoyait
/// `1899` et tronquait les centimes sans rien dire. Une saisie illisible renvoie
/// désormais `None`, et le formulaire la refuse avec un message.
pub fn parse_amount(raw: &str) -> Option<f64> {
    let normalised = raw.trim().replace(',', ".");
    normalised.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Montant strictement positif, ou `None`.
fn positive_amount(raw: &str) -> Option<f64> {
    parse_amount(raw).filter(|v| *v > 0.0)
}

pub struct ClientForm<'a> {
    pub company_name: &'a str,
    pub contact_name: &'a str,
    pub email: &'a str,
    pub phone: &'a str,
    pub address: &'a str,
}

/// Champ d'un formulaire auquel une erreur se rattache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    CompanyName,
    ContactName,
    Email,
    Phone,
    Address,
    Client,
    Description,
    Amount,
    Merchant,
}

/// Erreurs d'un formulaire, une au plus par champ, dans l'ordre des champs.
///
/// Équivalent de l'objet `{ champ: message }` que renvoyait la version
/// JavaScript : les formulaires affichent chaque message sous son champ.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldErrors(Vec<(Field, &'static str)>);

impl FieldErrors {
    fn check(&mut self, field: Field, failed: bool, message: &'static str) {
        if failed && self.get(field).is_none() {
            self.0.push((field, message));
        }
    }

    pub fn get(&self, field: Field) -> Option<&'static str> {
        self.0.iter().find(|(f, _)| *f == field).map(|(_, m)| *m)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Messages dans l'ordre des champs, pour les tests.
    #[cfg(test)]
    pub fn messages(&self) -> Vec<&'static str> {
        self.0.iter().map(|(_, m)| *m).collect()
    }
}

/// Messages d'erreur, repris mot pour mot de la version JavaScript.
pub fn validate_client(form: &ClientForm) -> FieldErrors {
    let mut errors = FieldErrors::default();

    errors.check(Field::CompanyName, form.company_name.trim().is_empty(), "Nom d'entreprise requis");
    errors.check(Field::ContactName, form.contact_name.trim().is_empty(), "Nom du contact requis");

    errors.check(Field::Email, form.email.trim().is_empty(), "Email requis");
    errors.check(Field::Email, !is_email(form.email), "Format email invalide (ex: client@domaine.fr)");

    let digits = form.phone.chars().filter(|c| !c.is_whitespace()).count();
    errors.check(Field::Phone, form.phone.trim().is_empty(), "Numéro de téléphone requis");
    errors.check(Field::Phone, digits != PHONE_DIGITS, "Le numéro doit faire exactement 10 chiffres");

    errors.check(Field::Address, form.address.trim().is_empty(), "Adresse requise");
    errors.check(
        Field::Address,
        !is_address(form.address),
        "Format invalide. Ex: '12 rue de Paris' (1-4 chiffres + rue/bd/av/place/impasse/route/chemin/allée/voie/square/quai + nom)",
    );

    errors
}

pub fn validate_invoice(client_id: Option<i64>, description: &str, amount_ht: &str) -> FieldErrors {
    let mut errors = FieldErrors::default();
    errors.check(Field::Client, client_id.is_none(), "Client requis");
    errors.check(Field::Description, description.trim().is_empty(), "Description de la prestation requise");
    errors.check(
        Field::Amount,
        positive_amount(amount_ht).is_none(),
        "Veuillez entrer un montant HT valide supérieur à 0",
    );
    errors
}

pub fn validate_estimate(client_id: Option<i64>, description: &str, amount_ht: &str) -> FieldErrors {
    let mut errors = FieldErrors::default();
    errors.check(Field::Client, client_id.is_none(), "Client requis");
    errors.check(Field::Description, description.trim().is_empty(), "Description requise");
    errors.check(Field::Amount, positive_amount(amount_ht).is_none(), "Montant HT valide requis (> 0)");
    errors
}

pub fn validate_expense(merchant: &str, amount: &str) -> FieldErrors {
    let mut errors = FieldErrors::default();
    errors.check(Field::Merchant, merchant.trim().is_empty(), "Fournisseur requis");
    errors.check(Field::Amount, positive_amount(amount).is_none(), "Montant supérieur à 0 requis");
    errors
}

/// Tests repris de `src/domain/validation.test.js`, dont ils héritent les cas.
#[cfg(test)]
mod tests {
    use super::*;

    fn valid_client() -> ClientForm<'static> {
        ClientForm {
            company_name: "Stark Industries",
            contact_name: "Pepper Potts",
            email: "pepper@stark.com",
            phone: "06 11 22 33 44",
            address: "108 route de Malibu",
        }
    }

    #[test]
    fn accepts_common_email_addresses() {
        for email in ["a@b.fr", "pepper@stark.com", "contact+devis@sous-domaine.example.co"] {
            assert!(is_email(email), "{email}");
        }
    }

    #[test]
    fn rejects_malformed_email_addresses() {
        for email in ["", "pepper", "pepper@", "@stark.com", "pepper@stark", "pepper @stark.com", "a@b@c.fr"] {
            assert!(!is_email(email), "{email:?}");
        }
    }

    #[test]
    fn accepts_recognised_street_types() {
        for address in [
            "12 rue de Paris", "1 boulevard Haussmann", "45 av des Champs",
            "3 place des Neuf Mondes", "108 route de Malibu", "7 impasse du Chat", "2 quai de Seine",
        ] {
            assert!(is_address(address), "{address}");
        }
    }

    #[test]
    fn address_check_ignores_case() {
        assert!(is_address("12 RUE DE PARIS"));
    }

    /// Défaut connu, reproduit tel quel : ces adresses sont valides et pourtant
    /// refusées. Choix de produit à trancher, pas à corriger en douce.
    #[test]
    fn rejects_valid_french_addresses_known_defect() {
        for address in [
            "12 bis rue de Paris", "Lieu-dit Le Moulin", "5 cours Mirabeau",
            "8 passage Verdeau", "3 villa des Ternes", "Le Bourg",
        ] {
            assert!(!is_address(address), "{address}");
        }
    }

    #[test]
    fn rejects_an_address_without_number_or_label() {
        assert!(!is_address("rue de Paris"));
        assert!(!is_address("12 rue"));
        assert!(!is_address("12345 rue de Paris"), "au plus quatre chiffres");
    }

    #[test]
    fn formats_phone_numbers_in_pairs() {
        assert_eq!(format_phone("0611223344"), "06 11 22 33 44");
        assert_eq!(format_phone("+33 (0)6.11.22.33.44"), "33 06 11 22 33");
        assert_eq!(format_phone("06112233445566"), "06 11 22 33 44");
        assert_eq!(format_phone(""), "");
        assert_eq!(format_phone("0"), "0");
        assert_eq!(format_phone("061"), "06 1");
    }

    /// Les adresses du jeu de référence de la phase 0 doivent passer.
    #[test]
    fn accepts_the_reference_dataset_addresses() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/reference-dataset.json")).unwrap();
        for client in data["clients"].as_array().unwrap() {
            let address = client["address"].as_str().unwrap();
            assert!(is_address(address), "{address}");
        }
    }

    #[test]
    fn phone_formatting_drops_everything_but_digits() {
        assert_eq!(format_phone("06.11-22 33a44"), "06 11 22 33 44");
        assert_eq!(format_phone("+33 6"), "33 6");
    }

    #[test]
    fn a_malformed_email_is_reported_under_its_field() {
        let form = ClientForm { email: "pepper@stark", ..valid_client() };
        let errors = validate_client(&form);
        assert_eq!(errors.get(Field::Email), Some("Format email invalide (ex: client@domaine.fr)"));
        assert_eq!(errors.messages().len(), 1);
    }

    #[test]
    fn a_malformed_address_is_reported_under_its_field() {
        let form = ClientForm { address: "Le Bourg", ..valid_client() };
        let message = validate_client(&form).get(Field::Address).unwrap();
        assert!(message.starts_with("Format invalide. Ex: '12 rue de Paris'"), "{message}");
    }

    /// Un champ vide ne porte que « requis », pas aussi « format invalide » —
    /// l'original testait l'un puis, sinon, l'autre.
    #[test]
    fn an_empty_field_reports_only_that_it_is_required() {
        let errors = validate_client(&ClientForm { email: "", address: "", ..valid_client() });
        assert_eq!(errors.get(Field::Email), Some("Email requis"));
        assert_eq!(errors.get(Field::Address), Some("Adresse requise"));
    }

    #[test]
    fn a_complete_client_is_valid() {
        assert!(validate_client(&valid_client()).is_empty());
    }

    #[test]
    fn every_client_field_is_required() {
        let errors = validate_client(&ClientForm {
            company_name: "",
            contact_name: "",
            email: "",
            phone: "",
            address: "",
        });

        assert_eq!(
            errors.messages(),
            [
                "Nom d'entreprise requis",
                "Nom du contact requis",
                "Email requis",
                "Numéro de téléphone requis",
                "Adresse requise",
            ]
        );
    }

    #[test]
    fn whitespace_counts_as_empty() {
        let form = ClientForm { company_name: "   ", ..valid_client() };
        assert!(validate_client(&form).messages().contains(&"Nom d'entreprise requis"));
    }

    #[test]
    fn phone_needs_exactly_ten_digits() {
        let short = ClientForm { phone: "06 11 22 33", ..valid_client() };
        assert!(validate_client(&short).messages().contains(&"Le numéro doit faire exactement 10 chiffres"));

        let compact = ClientForm { phone: "0611223344", ..valid_client() };
        assert!(validate_client(&compact).is_empty());
    }

    #[test]
    fn an_invoice_needs_client_description_and_amount() {
        assert_eq!(
            validate_invoice(None, "", "").messages(),
            [
                "Client requis",
                "Description de la prestation requise",
                "Veuillez entrer un montant HT valide supérieur à 0",
            ]
        );
        assert!(validate_invoice(Some(1), "Audit", "5000").is_empty());
    }

    #[test]
    fn rejects_zero_negative_and_unreadable_amounts() {
        for amount in ["0", "-100", "abc", ""] {
            assert!(!validate_invoice(Some(1), "Audit", amount).is_empty(), "{amount:?}");
        }
    }

    /// **Défaut de la phase 0, corrigé.** Le JavaScript lisait `1899,99` comme
    /// `1899` et perdait les centimes sans prévenir. Le test qui documentait ce
    /// comportement est réécrit ici, pas supprimé.
    #[test]
    fn a_decimal_comma_is_read_correctly() {
        assert_eq!(parse_amount("1899,99"), Some(1899.99));
        assert_eq!(parse_amount("1899.99"), Some(1899.99));
        assert_eq!(parse_amount(" 12,5 "), Some(12.5));
        assert!(validate_invoice(Some(1), "Audit", "1899,99").is_empty());
    }

    #[test]
    fn a_complete_estimate_is_valid() {
        assert!(validate_estimate(Some(1), "Refonte", "24000").is_empty());
    }

    #[test]
    fn estimates_carry_their_own_shorter_messages() {
        assert_eq!(
            validate_estimate(None, "", "").messages(),
            ["Client requis", "Description requise", "Montant HT valide requis (> 0)"]
        );
    }

    #[test]
    fn an_expense_needs_merchant_and_amount_only() {
        assert_eq!(
            validate_expense("", "").messages(),
            ["Fournisseur requis", "Montant supérieur à 0 requis"]
        );
        assert!(validate_expense("OVH", "119").is_empty());
    }
}
