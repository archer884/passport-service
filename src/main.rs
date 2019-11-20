mod weights;

use leptess::tesseract::TessApi;
use serde::Serialize;
use std::path::Path;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    path: String,
}

struct Reader {
    api: TessApi,
}

impl Reader {
    fn new() -> Self {
        Self {
            api: TessApi::new(Some("./resource"), "mrz").unwrap(),
        }
    }

    fn read(&mut self, path: impl AsRef<Path>) -> Option<String> {
        use leptess::leptonica;
        let pixels = leptonica::pix_read(path.as_ref())?;
        self.api.set_image(&pixels);
        self.api.get_utf8_text().ok()
    }
}

#[derive(Clone, Debug)]
struct PassportData {
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
    fn try_from_str(s: impl AsRef<str>) -> Option<Self> {
        let s: String = s
            .as_ref()
            .bytes()
            .filter(|u| !u.is_ascii_whitespace())
            .map(|u| u as char)
            .collect();

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
enum Sex {
    Male,
    Female,
    Unspecified,
}

impl Sex {
    fn from_str(s: &str) -> Sex {
        match s {
            "M" => Sex::Male,
            "F" => Sex::Female,
            _ => Sex::Unspecified,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all="camelCase")]
struct PassportInfo {
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
    fn from_data(data: PassportData) -> Self {
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
            passport_number: check(&passport_number, alphanumeric),
            date_of_birth: check(&date_of_birth, numeric),
            expiration_date: check(&expiration_date, numeric),
            personal_number: check(&personal_number, alphanumeric),

            // Cumulative checksum warning
            warning: !check(
                format!(
                    "{}{}{}{}{}",
                    passport_number, date_of_birth, expiration_date, personal_number, check_digit
                ),
                alphanumeric,
            )
            .is_valid(),
        }
    }
}

fn read_names(s: &str) -> (String, String) {
    // FIXME: this will asplode on an empty string.
    let mut parts = s.split("<<");
    (
        parts.next().expect("Empty name field").into(),
        parts.map(|part| part.replace("<", " ")).collect::<String>().trim().into(),
    )
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all="camelCase")]
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

fn main() {
    let Opt { path } = Opt::from_args();
    let mut reader = Reader::new();
    let raw_data = reader.read(path).unwrap();

    let value = PassportInfo::from_data(PassportData::try_from_str(raw_data).expect("wtf?"));
    let serialized = serde_json::to_string_pretty(&value).unwrap();

    println!("{}", serialized);
}

fn numeric(s: &str) -> Vec<i32> {
    s.bytes()
        .map(|u| match u {
            b'O' | b'o' => b'0',
            b'l' | b'I' | b'i' => b'1',
            b'B' => b'8',
            x => x,
        } - b'0')
        .map(|u| u as i32)
        .collect()
}

fn alphanumeric(s: &str) -> Vec<i32> {
    s.bytes()
        .map(|u| match u {
            u if u.is_ascii_alphabetic() => u.to_ascii_uppercase() - b'A' + 10,
            u if u == b'<' => 0,
            u => u - b'0',
        })
        .filter_map(|u| if u == b'<' { None } else { Some(u as i32) })
        .collect()
}

fn check(s: impl AsRef<str>, filter: impl Fn(&str) -> Vec<i32>) -> Checked<String> {
    use weights::Weights;

    let s = s.as_ref();
    let digits = filter(s);

    let values = &digits[..(s.len() - 1)];
    let check_digit = digits[s.len() - 1];
    let folded_value = values
        .iter()
        .zip(Weights::new())
        .fold(0, |a, (&value, weight)| (value * weight) + a);

    let is_valid = (folded_value % 10) == check_digit;
    Checked::new(s[..(s.len() - 1)].trim_end_matches('<'), is_valid)
}

#[cfg(test)]
mod tests {
    #[test]
    fn check() {
        assert!(super::check("1204159", super::numeric).is_valid());
        assert!(super::check("7408122", super::numeric).is_valid());
        assert!(super::check("ZE184226B<<<<<1", super::alphanumeric).is_valid());
        assert!(super::check("<<<<<<<<<<<<<<<", super::alphanumeric).is_valid());
    }
}
