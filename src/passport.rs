use crate::normalization::{Normalizer, Alphanumeric, Numeric};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct PassportData {
    // Row 1:
    issuer: String, // 3-5
    name: String,   // 6-44 (Surname<<Given<Names<Separated)

    // Row 2:
    passport_number: String, // 1-10 (includes check digit)
    nationality: String,     // 11-13
    date_of_birth: String,   // 14-20 (includes check digit)
    sex: String,             // 21
    expiration_date: String, // 22-28 (includes check digit)
    personal_number: String, // 29-43 (includes check digit)
    check_digit: String,     // 44 (check digit over 1-10,14-20,22-43)
}

impl PassportData {
    pub fn try_from_str(s: impl AsRef<str>) -> Option<Self> {
        let s: String = s
            .as_ref()
            .bytes()
            .filter(|u| !u.is_ascii_whitespace())
            .map(|u| u as char)
            .collect();

        if s.len() < 88 {
            return None;
        }

        let offset = s.len() - 88;
        let row_1 = &s[offset..(offset + 44)];

        let offset = s.len() - 44;
        let row_2 = &s[offset..];

        Some(PassportData {
            // Row 1:
            issuer: row_1[2..5].into(),
            name: row_1[5..44].into(),

            // Row 2:
            passport_number: row_2[0..10].into(),
            nationality: row_2[10..13].into(),
            date_of_birth: row_2[13..20].into(),
            sex: row_2[20..21].into(),
            expiration_date: row_2[21..28].into(),
            personal_number: row_2[28..43].into(),
            check_digit: row_2[43..].into(),
        })
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Sex {
    Male,
    Female,
    Unspecified,
}

impl Sex {
    pub fn from_str(s: &str) -> Sex {
        match s {
            "M" => Sex::Male,
            "F" => Sex::Female,
            _ => Sex::Unspecified,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassportInfo {
    issuer: Checked<String>,
    nationality: Checked<String>,
    surname: Checked<String>,
    names: Checked<String>,
    sex: Checked<Sex>,

    // Checked fields
    passport_number: Checked<String>,
    date_of_birth: Checked<String>,
    expiration_date: Checked<String>,
    personal_number: Checked<String>,

    // Cumulative checksum warning
    warning: bool,
}

impl PassportInfo {
    pub fn from_data(data: PassportData) -> Self {
        let PassportData {
            issuer,
            nationality,
            name,
            sex,
            passport_number,
            date_of_birth,
            expiration_date,
            personal_number,
            check_digit,
        } = data;

        let (surname, names) = read_names(&name);
        PassportInfo {
            // FIXME: these fields are NOT checked, but we store them as checked for the UI.
            issuer: Checked::new(issuer, true),
            nationality: Checked::new(nationality, true),
            surname: Checked::new(surname, true),
            names: Checked::new(names, true),
            sex: Checked::new(Sex::from_str(&sex), true),

            // Checked fields
            passport_number: check(&passport_number, Alphanumeric),
            date_of_birth: check(&date_of_birth, Numeric),
            expiration_date: check(&expiration_date, Numeric),
            personal_number: check(&personal_number, Alphanumeric),

            // Cumulative checksum warning
            warning: !check(
                format!(
                    "{}{}{}{}{}",
                    passport_number, date_of_birth, expiration_date, personal_number, check_digit
                ),
                Alphanumeric,
            )
            .is_valid(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Checked<T> {
    data: T,
    is_valid: bool,
}

impl<T> Checked<T> {
    fn new(s: impl Into<T>, is_valid: bool) -> Self {
        Checked {
            data: s.into(),
            is_valid,
        }
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }
}

fn check(s: impl AsRef<str>, normalizer: impl Normalizer) -> Checked<String> {
    use crate::weights::Weights;

    let s = s.as_ref();
    let digits = normalizer.normal_values(s);

    let values = &digits[..(s.len() - 1)];
    let check_digit = digits[s.len() - 1];
    let folded_value = values
        .iter()
        .zip(Weights::new())
        .fold(0, |a, (&value, weight)| (value * weight) + a);

    let is_valid = (folded_value % 10) == check_digit;
    let printable_value = normalizer.normal_text(s);
    Checked::new(printable_value[..(printable_value.len() - 1)].trim_end_matches('<'), is_valid)
}

fn read_names(s: &str) -> (String, String) {
    let mut parts = s.split("<<");
    (
        parts.next().expect("Empty name field").into(),
        parts
            .map(|part| part.replace("<", " "))
            .collect::<String>()
            .trim()
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use crate::normalization::{Alphanumeric, Numeric};

    #[test]
    fn check() {
        assert!(super::check("1204159", Numeric).is_valid());
        assert!(super::check("7408122", Numeric).is_valid());
        assert!(super::check("ZE184226B<<<<<1", Alphanumeric).is_valid());
        assert!(super::check("<<<<<<<<<<<<<<<", Alphanumeric).is_valid());
    }
}
