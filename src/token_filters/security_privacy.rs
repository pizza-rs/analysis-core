use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

// ═══════════════════════════════════════════════════════════════════════════════
// SECURITY & PRIVACY FILTERS — PII masking, threat detection
// ═══════════════════════════════════════════════════════════════════════════════

/// Masks email addresses: "user@example.com" → "u***@e***.com"
#[derive(Clone, Debug)]
pub struct EmailMaskTokenFilter {
    pub mask_char: char,
}
impl EmailMaskTokenFilter {
    pub fn new() -> Self { Self { mask_char: '*' } }
}
impl Default for EmailMaskTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for EmailMaskTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(at_pos) = text.find('@') {
            let local = &text[..at_pos];
            let domain = &text[at_pos + 1..];
            // Mask the local-part by keeping the first *character* (not the
            // first byte) and replacing every remaining character with
            // `mask_char`. Slicing by byte (`&local[..1]`) would panic on
            // emails whose local-part begins with a non-ASCII character
            // (e.g. internationalized email "ñame@host").
            let masked_local = mask_keep_first_char(local, self.mask_char);
            let masked_domain = if let Some(dot_pos) = domain.find('.') {
                let dname = &domain[..dot_pos];
                let ext = &domain[dot_pos..];
                let masked_d = mask_keep_first_char(dname, self.mask_char);
                format!("{}{}", masked_d, ext)
            } else {
                String::from(domain)
            };
            token.term = Cow::Owned(format!("{}@{}", masked_local, masked_domain));
        }
        (false, None)
    }
}

/// Keep the first Unicode scalar of `s` and replace every remaining
/// scalar with `mask_char`. Returns the input unchanged when it has
/// 0 or 1 characters.
fn mask_keep_first_char(s: &str, mask_char: char) -> String {
    let mut chars = s.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return String::new(),
    };
    let rest_count = chars.count();
    if rest_count == 0 {
        return first.to_string();
    }
    let mut out = String::with_capacity(first.len_utf8() + rest_count * mask_char.len_utf8());
    out.push(first);
    for _ in 0..rest_count {
        out.push(mask_char);
    }
    out
}

/// Masks credit card numbers: "4111111111111111" → "4111********1111"
#[derive(Clone, Debug)]
pub struct CreditCardMaskTokenFilter {
    pub mask_char: char,
    pub visible_start: usize,
    pub visible_end: usize,
}
impl CreditCardMaskTokenFilter {
    pub fn new() -> Self { Self { mask_char: '*', visible_start: 4, visible_end: 4 } }
}
impl Default for CreditCardMaskTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for CreditCardMaskTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 13 && digits.len() <= 19 && is_luhn_valid(&digits) {
            let len = digits.len();
            let start = &digits[..self.visible_start.min(len)];
            let end = &digits[len.saturating_sub(self.visible_end)..];
            let middle_len = len.saturating_sub(self.visible_start + self.visible_end);
            let masked = format!("{}{}{}", start,
                core::iter::repeat(self.mask_char).take(middle_len).collect::<String>(),
                end);
            token.term = Cow::Owned(masked);
        }
        (false, None)
    }
}

/// Masks phone numbers, keeping only last 4 digits.
#[derive(Clone, Debug)]
pub struct PhoneMaskTokenFilter {
    pub mask_char: char,
}
impl PhoneMaskTokenFilter {
    pub fn new() -> Self { Self { mask_char: '*' } }
}
impl Default for PhoneMaskTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for PhoneMaskTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let digits: Vec<char> = text.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 7 {
            let visible = 4;
            let masked_count = digits.len() - visible;
            let masked: String = core::iter::repeat(self.mask_char).take(masked_count).collect::<String>()
                + &digits[masked_count..].iter().collect::<String>();
            token.term = Cow::Owned(masked);
        }
        (false, None)
    }
}

/// Masks IP addresses: "192.168.1.100" → "192.168.xxx.xxx"
#[derive(Clone, Debug)]
pub struct IpMaskTokenFilter {
    pub octets_visible: usize,
}
impl IpMaskTokenFilter {
    pub fn new() -> Self { Self { octets_visible: 2 } }
}
impl Default for IpMaskTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for IpMaskTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts: Vec<&str> = text.split('.').collect();
        if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
            let mut result = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                if i < self.octets_visible {
                    result.push(String::from(*part));
                } else {
                    result.push(String::from("xxx"));
                }
            }
            token.term = Cow::Owned(result.join("."));
        }
        (false, None)
    }
}

/// Detects and masks SSN-like patterns (xxx-xx-xxxx).
#[derive(Clone, Debug)]
pub struct SsnMaskTokenFilter {
    pattern: Regex,
}
impl SsnMaskTokenFilter {
    pub fn new() -> Self {
        Self { pattern: Regex::new(r"\d{3}-\d{2}-\d{4}").unwrap() }
    }
}
impl Default for SsnMaskTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for SsnMaskTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if self.pattern.is_match(text) {
            // Keep last 4 digits only
            let result = self.pattern.replace_all(text, "***-**-$0");
            let masked = if text.len() >= 4 {
                format!("***-**-{}", &text[text.len()-4..])
            } else {
                String::from("***-**-****")
            };
            token.term = Cow::Owned(masked);
        }
        (false, None)
    }
}

/// Completely redacts (removes) tokens matching sensitive patterns.
#[derive(Clone, Debug)]
pub struct RedactTokenFilter {
    patterns: Vec<Regex>,
}
impl RedactTokenFilter {
    pub fn new(patterns: &[&str]) -> Self {
        Self {
            patterns: patterns.iter().filter_map(|p| Regex::new(p).ok()).collect(),
        }
    }
    pub fn emails_and_phones() -> Self {
        Self::new(&[
            r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}",
            r"\d{3}[-.]?\d{3}[-.]?\d{4}",
        ])
    }
}
impl Default for RedactTokenFilter { fn default() -> Self { Self { patterns: Vec::new() } } }

impl TokenFilter for RedactTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        for pattern in &self.patterns {
            if pattern.is_match(text) {
                return (true, None); // Delete the token entirely
            }
        }
        (false, None)
    }
}

/// Detects SQL injection patterns in tokens and removes them.
#[derive(Clone, Debug)]
pub struct SqlInjectionDetectTokenFilter {
    patterns: Vec<Regex>,
}
impl SqlInjectionDetectTokenFilter {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                Regex::new(r"(?i)(union\s+select|drop\s+table|insert\s+into|delete\s+from)").unwrap(),
                Regex::new(r"(?i)(or\s+1\s*=\s*1|and\s+1\s*=\s*1|'\s*or\s*')").unwrap(),
                Regex::new(r"(?i)(exec\s*\(|execute\s|xp_cmdshell)").unwrap(),
                Regex::new(r"(--|;)\s*(drop|alter|create|truncate)").unwrap(),
            ],
        }
    }
}
impl Default for SqlInjectionDetectTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for SqlInjectionDetectTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        for pattern in &self.patterns {
            if pattern.is_match(text) {
                let tag = Token {
                    term: Cow::Owned(String::from("_threat:sql_injection")),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (true, Some(alloc::vec![tag]));
            }
        }
        (false, None)
    }
}

/// Detects XSS patterns in tokens.
#[derive(Clone, Debug)]
pub struct XssDetectTokenFilter {
    patterns: Vec<Regex>,
}
impl XssDetectTokenFilter {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                Regex::new(r"(?i)<script[\s>]").unwrap(),
                Regex::new(r"(?i)javascript\s*:").unwrap(),
                Regex::new(r"(?i)on(load|error|click|mouse)\s*=").unwrap(),
                Regex::new(r"(?i)<iframe[\s>]").unwrap(),
                Regex::new(r"(?i)eval\s*\(").unwrap(),
            ],
        }
    }
}
impl Default for XssDetectTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for XssDetectTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        for pattern in &self.patterns {
            if pattern.is_match(text) {
                let tag = Token {
                    term: Cow::Owned(String::from("_threat:xss")),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (true, Some(alloc::vec![tag]));
            }
        }
        (false, None)
    }
}

/// Detects path traversal attempts.
#[derive(Clone, Debug)]
pub struct PathTraversalDetectTokenFilter;
impl PathTraversalDetectTokenFilter { pub fn new() -> Self { Self } }
impl Default for PathTraversalDetectTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for PathTraversalDetectTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.contains("../") || text.contains("..\\") || text.contains("%2e%2e") {
            let tag = Token {
                term: Cow::Owned(String::from("_threat:path_traversal")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (true, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn is_luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for c in digits.chars().rev() {
        if let Some(d) = c.to_digit(10) {
            let mut val = d;
            if double {
                val *= 2;
                if val > 9 { val -= 9; }
            }
            sum += val;
            double = !double;
        } else {
            return false;
        }
    }
    sum % 10 == 0
}
