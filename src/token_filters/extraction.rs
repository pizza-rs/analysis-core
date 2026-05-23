use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

// ═══════════════════════════════════════════════════════════════════════════════
// ENTITY & DATA EXTRACTION FILTERS — Extract structured data from text
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts email addresses from token text and emits them as synonyms.
#[derive(Clone, Debug)]
pub struct EmailExtractTokenFilter {
    pattern: Regex,
}
impl EmailExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap(),
        }
    }
}
impl Default for EmailExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for EmailExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let matches: Vec<&str> = self.pattern.find_iter(text).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            return (false, None);
        }
        let extras: Vec<Token<'a>> = matches.iter().map(|email| Token {
            term: Cow::Owned(String::from(*email)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        }).collect();
        (false, Some(extras))
    }
}

/// Extracts URLs from token text.
#[derive(Clone, Debug)]
pub struct UrlExtractTokenFilter {
    pattern: Regex,
}
impl UrlExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"https?://[^\s<>\[\]{}|\\^`]+").unwrap(),
        }
    }
}
impl Default for UrlExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for UrlExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let matches: Vec<&str> = self.pattern.find_iter(text).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            return (false, None);
        }
        let extras: Vec<Token<'a>> = matches.iter().map(|url| Token {
            term: Cow::Owned(String::from(*url)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        }).collect();
        (false, Some(extras))
    }
}

/// Extracts IP addresses (v4 and v6) from token text.
#[derive(Clone, Debug)]
pub struct IpExtractTokenFilter {
    ipv4_pattern: Regex,
}
impl IpExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            ipv4_pattern: Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").unwrap(),
        }
    }
}
impl Default for IpExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for IpExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let matches: Vec<&str> = self.ipv4_pattern.find_iter(text).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            return (false, None);
        }
        let extras: Vec<Token<'a>> = matches.iter().filter_map(|ip| {
            // Validate octets
            let parts: Vec<&str> = ip.split('.').collect();
            if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
                Some(Token {
                    term: Cow::Owned(String::from(*ip)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                })
            } else {
                None
            }
        }).collect();
        if extras.is_empty() { return (false, None); }
        (false, Some(extras))
    }
}

/// Extracts hashtags from token text (e.g. #pizza → pizza).
#[derive(Clone, Debug)]
pub struct HashtagExtractTokenFilter {
    pub keep_hash: bool,
}
impl HashtagExtractTokenFilter {
    pub fn new() -> Self { Self { keep_hash: false } }
}
impl Default for HashtagExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for HashtagExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.starts_with('#') && text.len() > 1 {
            let tag = if self.keep_hash { text } else { &text[1..] };
            token.term = Cow::Owned(String::from(tag));
        }
        (false, None)
    }
}

/// Extracts @mentions from token text.
#[derive(Clone, Debug)]
pub struct MentionExtractTokenFilter {
    pub keep_at: bool,
}
impl MentionExtractTokenFilter {
    pub fn new() -> Self { Self { keep_at: false } }
}
impl Default for MentionExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for MentionExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.starts_with('@') && text.len() > 1 {
            let mention = if self.keep_at { text } else { &text[1..] };
            token.term = Cow::Owned(String::from(mention));
        }
        (false, None)
    }
}

/// Extracts phone numbers from token text.
#[derive(Clone, Debug)]
pub struct PhoneExtractTokenFilter {
    pattern: Regex,
}
impl PhoneExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"[\+]?[(]?[0-9]{1,4}[)]?[-\s./0-9]{6,}").unwrap(),
        }
    }
}
impl Default for PhoneExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for PhoneExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let matches: Vec<&str> = self.pattern.find_iter(text).map(|m| m.as_str()).collect();
        if matches.is_empty() {
            return (false, None);
        }
        let extras: Vec<Token<'a>> = matches.iter().map(|ph| Token {
            term: Cow::Owned(String::from(*ph)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        }).collect();
        (false, Some(extras))
    }
}

/// Extracts currency amounts (e.g. $100.50, €50, ¥1000).
#[derive(Clone, Debug)]
pub struct CurrencyExtractTokenFilter {
    pattern: Regex,
}
impl CurrencyExtractTokenFilter {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"[$€£¥₹₽₿]\s*\d[\d,]*\.?\d*|\d[\d,]*\.?\d*\s*[$€£¥₹₽₿]").unwrap(),
        }
    }
}
impl Default for CurrencyExtractTokenFilter { fn default() -> Self { Self::new() } }

impl TokenFilter for CurrencyExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if self.pattern.is_match(text) {
            // Normalize: extract symbol and amount
            let normalized: String = text.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            let symbol: String = text.chars().filter(|c| matches!(*c, '$'|'€'|'£'|'¥'|'₹'|'₽'|'₿')).collect();
            let synonym = Token {
                term: Cow::Owned(format!("_currency:{}:{}", symbol, normalized)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![synonym]));
        }
        (false, None)
    }
}

/// Extracts numbers and emits both original and numeric-normalized form.
#[derive(Clone, Debug)]
pub struct NumberExtractTokenFilter;
impl NumberExtractTokenFilter { pub fn new() -> Self { Self } }
impl Default for NumberExtractTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for NumberExtractTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        // Remove commas for normalization
        let cleaned: String = text.chars().filter(|c| *c != ',').collect();
        if let Ok(_) = cleaned.parse::<f64>() {
            if cleaned != text {
                let synonym = Token {
                    term: Cow::Owned(cleaned),
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

/// Map-based lookup filter: replaces tokens found in a dictionary.
#[derive(Clone, Debug)]
pub struct LookupTokenFilter {
    entries: Vec<(String, String)>,
}

impl LookupTokenFilter {
    pub fn new(entries: &[(&str, &str)]) -> Self {
        Self {
            entries: entries.iter().map(|(k, v)| (String::from(*k), String::from(*v))).collect(),
        }
    }

    pub fn from_pairs(entries: Vec<(String, String)>) -> Self {
        Self { entries }
    }
}

impl Default for LookupTokenFilter {
    fn default() -> Self { Self { entries: Vec::new() } }
}

impl TokenFilter for LookupTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        for (key, value) in &self.entries {
            if term == key.as_str() {
                token.term = Cow::Owned(value.clone());
                return (false, None);
            }
        }
        (false, None)
    }
}

/// Emits the domain part of an email as a synonym.
#[derive(Clone, Debug)]
pub struct EmailDomainTokenFilter;
impl EmailDomainTokenFilter { pub fn new() -> Self { Self } }
impl Default for EmailDomainTokenFilter { fn default() -> Self { Self } }

impl TokenFilter for EmailDomainTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(at_pos) = text.find('@') {
            let domain = &text[at_pos + 1..];
            if !domain.is_empty() {
                let synonym = Token {
                    term: Cow::Owned(String::from(domain)),
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
