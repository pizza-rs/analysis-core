use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// NUMBER & CONVERSION FILTERS — Numeric intelligence, format conversion
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts roman numerals to decimal: "XIV" → "14"
#[derive(Clone, Debug)]
pub struct RomanNumeralTokenFilter;
impl RomanNumeralTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for RomanNumeralTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for RomanNumeralTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(value) = parse_roman_numeral(text) {
            let synonym = Token {
                term: Cow::Owned(format!("{}", value)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![synonym]));
        }
        (false, None)
    }
}

/// Converts ordinals to their numeric form: "1st" → "1", "twenty-third" → "23"
#[derive(Clone, Debug)]
pub struct OrdinalTokenFilter;
impl OrdinalTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for OrdinalTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for OrdinalTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        // Strip ordinal suffixes: 1st, 2nd, 3rd, 4th, etc.
        let stripped = lower
            .strip_suffix("st")
            .or_else(|| lower.strip_suffix("nd"))
            .or_else(|| lower.strip_suffix("rd"))
            .or_else(|| lower.strip_suffix("th"));
        if let Some(num_str) = stripped {
            if num_str.chars().all(|c| c.is_ascii_digit()) && !num_str.is_empty() {
                token.term = Cow::Owned(String::from(num_str));
                return (false, None);
            }
        }
        (false, None)
    }
}

/// Normalizes number formats: removes thousands separators, normalizes decimal.
/// "1,234,567.89" → "1234567.89", "1.234.567,89" (EU) → "1234567.89"
#[derive(Clone, Debug)]
pub struct NumberNormTokenFilter {
    pub decimal_char: char,
    pub thousands_char: char,
}
impl NumberNormTokenFilter {
    pub fn new() -> Self {
        Self {
            decimal_char: '.',
            thousands_char: ',',
        }
    }

    pub fn european() -> Self {
        Self {
            decimal_char: ',',
            thousands_char: '.',
        }
    }
}
impl Default for NumberNormTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for NumberNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_magnitude:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        let text = token.term.as_ref();
        if !text.chars().any(|c| c.is_ascii_digit()) {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        for c in text.chars() {
            if c == self.thousands_char {
                continue; // Skip thousands separator
            } else if c == self.decimal_char {
                result.push('.'); // Normalize decimal to dot
            } else {
                result.push(c);
            }
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

/// Tags numbers with their magnitude: "42" → emits "_magnitude:tens"
#[derive(Clone, Debug)]
pub struct NumberMagnitudeTokenFilter;
impl NumberMagnitudeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NumberMagnitudeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NumberMagnitudeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Ok(n) = text.parse::<f64>() {
            let magnitude = get_magnitude(n.abs());
            let tag = Token {
                term: Cow::Owned(format!("_magnitude:{}", magnitude)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Converts hex numbers to decimal: "0xFF" → "255"
#[derive(Clone, Debug)]
pub struct HexToDecimalTokenFilter;
impl HexToDecimalTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HexToDecimalTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HexToDecimalTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let hex_str = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"));
        if let Some(hex) = hex_str {
            if let Ok(val) = u64::from_str_radix(hex, 16) {
                let synonym = Token {
                    term: Cow::Owned(format!("{}", val)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![synonym]));
            }
        }
        (false, None)
    }
}

/// Converts binary notation to decimal: "0b1010" → "10"
#[derive(Clone, Debug)]
pub struct BinaryToDecimalTokenFilter;
impl BinaryToDecimalTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for BinaryToDecimalTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for BinaryToDecimalTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let bin_str = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B"));
        if let Some(bin) = bin_str {
            if let Ok(val) = u64::from_str_radix(bin, 2) {
                let synonym = Token {
                    term: Cow::Owned(format!("{}", val)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![synonym]));
            }
        }
        (false, None)
    }
}

/// Converts octal to decimal: "0o777" → "511"
#[derive(Clone, Debug)]
pub struct OctalToDecimalTokenFilter;
impl OctalToDecimalTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for OctalToDecimalTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for OctalToDecimalTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let oct_str = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O"));
        if let Some(oct) = oct_str {
            if let Ok(val) = u64::from_str_radix(oct, 8) {
                let synonym = Token {
                    term: Cow::Owned(format!("{}", val)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![synonym]));
            }
        }
        (false, None)
    }
}

/// Normalizes numeric ranges: "10-20" → emits "10" and "20" as synonyms
#[derive(Clone, Debug)]
pub struct NumericRangeTokenFilter;
impl NumericRangeTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for NumericRangeTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for NumericRangeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_bytes:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        let text = token.term.as_ref();
        // Patterns: "10-20", "10..20", "10~20"
        for sep in &["-", "..", "~"] {
            if let Some(pos) = text.find(sep) {
                let left = &text[..pos];
                let right = &text[pos + sep.len()..];
                if left.chars().all(|c| c.is_ascii_digit() || c == '.')
                    && right.chars().all(|c| c.is_ascii_digit() || c == '.')
                    && !left.is_empty()
                    && !right.is_empty()
                {
                    let t1 = Token {
                        term: Cow::Owned(String::from(left)),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    };
                    let t2 = Token {
                        term: Cow::Owned(String::from(right)),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    };
                    return (false, Some(alloc::vec![t1, t2]));
                }
            }
        }
        (false, None)
    }
}

/// Converts file size strings: "1.5GB" → emits "_bytes:1610612736"
#[derive(Clone, Debug)]
pub struct FileSizeNormTokenFilter;
impl FileSizeNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for FileSizeNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for FileSizeNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_seconds:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        if let Some(bytes) = parse_file_size(&lower) {
            let tag = Token {
                term: Cow::Owned(format!("_bytes:{}", bytes)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Converts duration strings: "2h30m" → emits "_seconds:9000"
#[derive(Clone, Debug)]
pub struct DurationNormTokenFilter;
impl DurationNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for DurationNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for DurationNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with("_pct:") {
            return (false, None); // idempotent: never re-tag an emitted tag
        }
        let text = token.term.as_ref();
        if let Some(seconds) = parse_duration(text) {
            let tag = Token {
                term: Cow::Owned(format!("_seconds:{}", seconds)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Normalizes percentage notation: "85%" → "0.85", emits "_pct:85"
#[derive(Clone, Debug)]
pub struct PercentNormTokenFilter;
impl PercentNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PercentNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for PercentNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.to_string();
        if let Some(num_str) = text.strip_suffix('%') {
            if let Ok(val) = num_str.parse::<f64>() {
                let normalized = val / 100.0;
                token.term = Cow::Owned(format!("{:.4}", normalized));
                let tag = Token {
                    term: Cow::Owned(format!("_pct:{}", num_str)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![tag]));
            }
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn parse_roman_numeral(s: &str) -> Option<u32> {
    let upper = s.to_uppercase();
    let chars: Vec<char> = upper.chars().collect();
    if chars.is_empty()
        || !chars
            .iter()
            .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
    {
        return None;
    }
    let mut total = 0u32;
    let mut prev = 0u32;
    for &c in chars.iter().rev() {
        let val = match c {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return None,
        };
        if val < prev {
            total -= val;
        } else {
            total += val;
        }
        prev = val;
    }
    if total > 0 {
        Some(total)
    } else {
        None
    }
}

fn get_magnitude(n: f64) -> &'static str {
    if n < 1.0 {
        "fraction"
    } else if n < 10.0 {
        "ones"
    } else if n < 100.0 {
        "tens"
    } else if n < 1_000.0 {
        "hundreds"
    } else if n < 1_000_000.0 {
        "thousands"
    } else if n < 1_000_000_000.0 {
        "millions"
    } else if n < 1_000_000_000_000.0 {
        "billions"
    } else {
        "trillions"
    }
}

fn parse_file_size(s: &str) -> Option<u64> {
    let suffixes: &[(&str, u64)] = &[
        ("tb", 1_099_511_627_776),
        ("gb", 1_073_741_824),
        ("mb", 1_048_576),
        ("kb", 1_024),
        ("b", 1),
        ("tib", 1_099_511_627_776),
        ("gib", 1_073_741_824),
        ("mib", 1_048_576),
        ("kib", 1_024),
    ];
    for &(suffix, multiplier) in suffixes {
        if s.ends_with(suffix) {
            let num_str = &s[..s.len() - suffix.len()];
            if let Ok(val) = num_str.parse::<f64>() {
                return Some((val * multiplier as f64) as u64);
            }
        }
    }
    None
}

fn parse_duration(s: &str) -> Option<u64> {
    let lower = s.to_lowercase();
    let mut total_seconds = 0u64;
    let mut current_num = String::new();
    let mut found_any = false;

    for c in lower.chars() {
        if c.is_ascii_digit() || c == '.' {
            current_num.push(c);
        } else {
            if current_num.is_empty() {
                continue;
            }
            let val: f64 = current_num.parse().ok()?;
            current_num.clear();
            let multiplier = match c {
                'd' => 86400,
                'h' => 3600,
                'm' => 60,
                's' => 1,
                _ => continue,
            };
            total_seconds += (val * multiplier as f64) as u64;
            found_any = true;
        }
    }
    if found_any {
        Some(total_seconds)
    } else {
        None
    }
}
