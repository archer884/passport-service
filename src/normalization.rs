pub trait Normalizer {
    fn normal_values(&self, s: &str) -> Vec<i32>;
    fn normal_text(&self, s: &str) -> String;
}

#[derive(Copy, Clone)]
pub struct Alphanumeric;

impl Normalizer for Alphanumeric {
    fn normal_values(&self, s: &str) -> Vec<i32> {
        s.bytes()
            .map(|u| match u {
                u if u == b'<' => 0,
                u if u.is_ascii_alphabetic() => u.to_ascii_uppercase() - b'A' + 10,
                u if u >= b'0' && u <= b'9' => u - b'0',
                u => u,
            })
            .filter_map(|u| if u == b'<' { None } else { Some(u as i32) })
            .collect()
    }

    fn normal_text(&self, s: &str) -> String {
        s.into()
    }
}

#[derive(Copy, Clone)]
pub struct Numeric;

impl Normalizer for Numeric {
    fn normal_values(&self, s: &str) -> Vec<i32> {
        s.bytes()
            .map(|u| match u {
                b'O' | b'o' => b'0',
                b'l' | b'I' | b'i' => b'1',
                b'B' => b'8',
                x => x,
            })
            .map(|u| if u >= b'0' && u <= b'9' { u - b'0' } else { u })
            .map(|u| u as i32)
            .collect()
    }

    fn normal_text(&self, s: &str) -> String {
        s.bytes()
            .map(|u| match u {
                b'O' | b'o' => b'0',
                b'l' | b'I' | b'i' => b'1',
                b'B' => b'8',
                x => x,
            })
            .map(|u| if (0..=9).contains(&u) { u - b'0' } else { u } as char)
            .collect()
    }
}
