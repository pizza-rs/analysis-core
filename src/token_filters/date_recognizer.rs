use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Recognizes date-like tokens and emits a normalized date synonym.
/// Detects: YYYY-MM-DD, DD/MM/YYYY, MM-DD-YYYY, YYYYMMDD, and more.
#[derive(Clone, Debug)]
pub struct DateRecognizerTokenFilter {
    pub emit_normalized: bool,
}

impl DateRecognizerTokenFilter {
    pub fn new() -> Self {
        Self { emit_normalized: true }
    }
}

impl Default for DateRecognizerTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn try_parse_date(s: &str) -> Option<(u32, u32, u32)> {
    let bytes = s.as_bytes();
    let len = s.len();

    // YYYY-MM-DD or YYYY/MM/DD
    if len == 10 && (bytes[4] == b'-' || bytes[4] == b'/') && bytes[7] == bytes[4] {
        let y: u32 = s[0..4].parse().ok()?;
        let m: u32 = s[5..7].parse().ok()?;
        let d: u32 = s[8..10].parse().ok()?;
        if (1900..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) {
            return Some((y, m, d));
        }
    }
    // DD/MM/YYYY or DD-MM-YYYY
    if len == 10 && (bytes[2] == b'-' || bytes[2] == b'/') && bytes[5] == bytes[2] {
        let d: u32 = s[0..2].parse().ok()?;
        let m: u32 = s[3..5].parse().ok()?;
        let y: u32 = s[6..10].parse().ok()?;
        if (1900..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) {
            return Some((y, m, d));
        }
    }
    // YYYYMMDD
    if len == 8 && s.bytes().all(|b| b.is_ascii_digit()) {
        let y: u32 = s[0..4].parse().ok()?;
        let m: u32 = s[4..6].parse().ok()?;
        let d: u32 = s[6..8].parse().ok()?;
        if (1900..=2100).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d) {
            return Some((y, m, d));
        }
    }
    None
}

impl TokenFilter for DateRecognizerTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if let Some((y, m, d)) = try_parse_date(token.term.as_ref()) {
            if self.emit_normalized {
                let normalized = format!("{:04}-{:02}-{:02}", y, m, d);
                if normalized != token.term.as_ref() {
                    let synonym = Token {
                        term: Cow::Owned(normalized),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    };
                    return (false, Some(alloc::vec![synonym]));
                }
            }
        }
        (false, None)
    }
}
