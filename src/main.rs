mod weights;

use leptess::tesseract::TessApi;
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
    check: String,           // 44 (check digit over 1-10,14-20,22-43)
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
        let row_1 = dbg!(&s[offset..(offset + 44)]);

        let offset = s.len() - 44;
        let row_2 = dbg!(&s[offset..]);

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
            check: row_2[43..].into(),
        })
    }
}

enum Sex {
    Male,
    Female,
    Unspecified,
}

struct PassportInfo {
    issuer: String,
    nationality: String,
    surname: String,
    name: String,
    sex: Sex,

    // Checked fields
    passport_number: String,
    date_of_birth: String,
    expiration_date: String,
    personal_number: String,
    checksum: String,
}

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

    println!("{:#?}", PassportData::try_from_str(raw_data));
}

fn check(s: impl AsRef<str>, is_numeric: bool) -> Checked<String> {
    use weights::Weights;
    
    let s = s.as_ref();
    let digits: Vec<_> = if is_numeric {
        s.bytes()
            .map(|u| match u {
                b'O' | b'o' => b'0',
                b'l' | b'I' | b'i' => b'1',
                b'B' => b'8',
                x => x,
            } - b'0')
            .map(|u| u as i32)
            .collect()
    } else {
        s.bytes()
            .map(|u| match u {
                u if u.is_ascii_alphabetic() => u.to_ascii_uppercase() - b'A' + 10,
                u if u == b'<' => 0,
                u => u - b'0',
            })
            .filter_map(|u| if u == b'<' {
                None
            } else {
                Some(u as i32)
            })
            .collect()
    };

    let values = dbg!(&digits[..(s.len() - 1)]);
    let check_digit = dbg!(digits[s.len() - 1]);
    let folded_value = values
        .iter()
        .zip(Weights::new())
        .fold(0, |a, (&value, weight)| (value * weight) + a);

    let is_valid = (folded_value % 10) == check_digit;
    Checked::new(&s[..(s.len() - 1)], is_valid)
}

#[cfg(test)]
mod tests {
    #[test]
    fn check() {
        assert!(super::check("1204159", true).is_valid());
        assert!(super::check("7408122", true).is_valid());
        assert!(super::check("ZE184226B<<<<<1", false).is_valid());
        assert!(super::check("<<<<<<<<<<<<<<<", false).is_valid());
    }
}
