use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use regex::Regex;

// ═══════════════════════════════════════════════════════════════════════════════
// CODE, LOG & SCIENTIFIC FILTERS — Developer & ops intelligence
// ═══════════════════════════════════════════════════════════════════════════════

/// Splits programming identifiers into words: "getUserById" → ["get","user","by","id"]
#[derive(Clone, Debug)]
pub struct IdentifierSplitTokenFilter {
    pub keep_original: bool,
}
impl IdentifierSplitTokenFilter {
    pub fn new() -> Self {
        Self {
            keep_original: true,
        }
    }
}
impl Default for IdentifierSplitTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for IdentifierSplitTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts = split_identifier(text);
        if parts.len() <= 1 {
            return (false, None);
        }
        let extras: Vec<Token<'a>> = parts
            .iter()
            .map(|p| Token {
                term: Cow::Owned(p.to_lowercase()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            })
            .collect();
        if self.keep_original {
            (false, Some(extras))
        } else {
            token.term = Cow::Owned(extras[0].term.to_string());
            if extras.len() > 1 {
                (false, Some(extras[1..].to_vec()))
            } else {
                (false, None)
            }
        }
    }
}

/// Detects and tags programming keywords.
#[derive(Clone, Debug)]
pub struct ProgrammingKeywordTokenFilter {
    keywords: Vec<&'static str>,
}
impl ProgrammingKeywordTokenFilter {
    pub fn new() -> Self {
        Self {
            keywords: vec![
                "if",
                "else",
                "for",
                "while",
                "return",
                "function",
                "class",
                "import",
                "export",
                "const",
                "let",
                "var",
                "fn",
                "pub",
                "struct",
                "enum",
                "impl",
                "trait",
                "use",
                "mod",
                "match",
                "async",
                "await",
                "try",
                "catch",
                "throw",
                "new",
                "delete",
                "typeof",
                "instanceof",
                "interface",
                "type",
                "void",
                "null",
                "undefined",
                "true",
                "false",
                "self",
                "super",
                "this",
            ],
        }
    }
}
impl Default for ProgrammingKeywordTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for ProgrammingKeywordTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if self.keywords.contains(&text) {
            let tag = Token {
                term: Cow::Owned(String::from("_type:keyword")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Detects log levels: "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
#[derive(Clone, Debug)]
pub struct LogLevelTokenFilter;
impl LogLevelTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for LogLevelTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for LogLevelTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let level = match text.to_uppercase().as_str() {
            "FATAL" | "CRITICAL" | "CRIT" => Some("fatal"),
            "ERROR" | "ERR" => Some("error"),
            "WARN" | "WARNING" => Some("warn"),
            "INFO" => Some("info"),
            "DEBUG" | "DBG" => Some("debug"),
            "TRACE" | "TRC" => Some("trace"),
            _ => None,
        };
        if let Some(lvl) = level {
            let tag = Token {
                term: Cow::Owned(format!("_log_level:{}", lvl)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Extracts key=value pairs from log-style tokens.
#[derive(Clone, Debug)]
pub struct KeyValuePairTokenFilter {
    pub separator: char,
}
impl KeyValuePairTokenFilter {
    pub fn new() -> Self {
        Self { separator: '=' }
    }
}
impl Default for KeyValuePairTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for KeyValuePairTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(pos) = text.find(self.separator) {
            let key = &text[..pos];
            // `separator` is a configurable `char`; use its actual UTF-8
            // length instead of a hard-coded `+1`. Otherwise any
            // multi-byte separator (e.g. `'。'`, `'='` width-variant
            // U+FF1D) would slice into the middle of the next character
            // and panic.
            let value = &text[pos + self.separator.len_utf8()..];
            if !key.is_empty() && !value.is_empty() {
                let key_token = Token {
                    term: Cow::Owned(format!("_key:{}", key)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                let val_token = Token {
                    term: Cow::Owned(format!("_val:{}", value)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                return (false, Some(alloc::vec![key_token, val_token]));
            }
        }
        (false, None)
    }
}

/// Normalizes semantic version numbers: "v1.2.3" → "1.2.3", emits major/minor/patch.
#[derive(Clone, Debug)]
pub struct SemverTokenFilter;
impl SemverTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SemverTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SemverTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let ver = text.strip_prefix('v').unwrap_or(text);
        let parts: Vec<&str> = ver.split('.').collect();
        if parts.len() >= 2
            && parts.len() <= 4
            && parts.iter().all(|p| {
                p.chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c.is_alphanumeric())
            })
        {
            let mut extras = Vec::new();
            if let Some(major) = parts.first() {
                extras.push(Token {
                    term: Cow::Owned(format!("_major:{}", major)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
            if let Some(minor) = parts.get(1) {
                extras.push(Token {
                    term: Cow::Owned(format!("_minor:{}", minor)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
            // Normalize: strip 'v' prefix
            if text.starts_with('v') {
                token.term = Cow::Owned(String::from(ver));
            }
            return (false, Some(extras));
        }
        (false, None)
    }
}

/// Normalizes HTTP status codes to categories.
#[derive(Clone, Debug)]
pub struct HttpStatusTokenFilter;
impl HttpStatusTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HttpStatusTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HttpStatusTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Ok(code) = text.parse::<u16>() {
            let category = match code {
                100..=199 => "informational",
                200..=299 => "success",
                300..=399 => "redirect",
                400..=499 => "client_error",
                500..=599 => "server_error",
                _ => return (false, None),
            };
            let tag = Token {
                term: Cow::Owned(format!("_http:{}", category)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Extracts error codes from structured text (e.g. "E0433", "SQLITE_BUSY").
#[derive(Clone, Debug)]
pub struct ErrorCodeTokenFilter {
    pattern: Regex,
}
impl ErrorCodeTokenFilter {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"^[A-Z][A-Z0-9_]*[0-9]+$|^[A-Z]{1,5}[-_][0-9]+$").unwrap(),
        }
    }
}
impl Default for ErrorCodeTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for ErrorCodeTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if self.pattern.is_match(text) {
            let tag = Token {
                term: Cow::Owned(String::from("_type:error_code")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Strips ANSI color/escape codes from terminal output.
#[derive(Clone, Debug)]
pub struct AnsiStripTokenFilter {
    pattern: Regex,
}
impl AnsiStripTokenFilter {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap(),
        }
    }
}
impl Default for AnsiStripTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for AnsiStripTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let cleaned = self.pattern.replace_all(text, "");
        if cleaned != text {
            token.term = Cow::Owned(cleaned.into_owned());
        }
        (false, None)
    }
}

/// ISBN normalization: strips hyphens and validates.
#[derive(Clone, Debug)]
pub struct IsbnNormTokenFilter;
impl IsbnNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for IsbnNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for IsbnNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let digits: String = text
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x')
            .collect();
        if digits.len() == 10 || digits.len() == 13 {
            token.term = Cow::Owned(digits);
            let tag = Token {
                term: Cow::Owned(String::from("_type:isbn")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// DOI normalization: ensures consistent format.
#[derive(Clone, Debug)]
pub struct DoiNormTokenFilter;
impl DoiNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for DoiNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for DoiNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.starts_with("10.") && text.contains('/') {
            // Already a DOI, normalize case
            token.term = Cow::Owned(text.to_lowercase());
            let tag = Token {
                term: Cow::Owned(String::from("_type:doi")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        // Strip doi: prefix or https://doi.org/ prefix
        if let Some(stripped) = text
            .strip_prefix("doi:")
            .or_else(|| text.strip_prefix("https://doi.org/"))
        {
            token.term = Cow::Owned(stripped.to_lowercase());
            let tag = Token {
                term: Cow::Owned(String::from("_type:doi")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Detects and tags file extensions: "report.pdf" → emits "_ext:pdf"
#[derive(Clone, Debug)]
pub struct FileExtensionTokenFilter;
impl FileExtensionTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for FileExtensionTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for FileExtensionTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(dot_pos) = text.rfind('.') {
            let ext = &text[dot_pos + 1..];
            if !ext.is_empty() && ext.len() <= 10 && ext.chars().all(|c| c.is_alphanumeric()) {
                let tag = Token {
                    term: Cow::Owned(format!("_ext:{}", ext.to_lowercase())),
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

/// Extracts file path components as separate tokens.
#[derive(Clone, Debug)]
pub struct PathComponentTokenFilter;
impl PathComponentTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for PathComponentTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for PathComponentTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.contains('/') || text.contains('\\') {
            let parts: Vec<&str> = text
                .split(&['/', '\\'][..])
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() > 1 {
                let extras: Vec<Token<'a>> = parts
                    .iter()
                    .map(|p| Token {
                        term: Cow::Owned(String::from(*p)),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    })
                    .collect();
                return (false, Some(extras));
            }
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn split_identifier(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();

    for c in s.chars() {
        if c == '_' || c == '-' || c == '.' {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        } else if c.is_uppercase()
            && !current.is_empty()
            && current.chars().last().map_or(false, |l| l.is_lowercase())
        {
            parts.push(current.clone());
            current.clear();
            current.push(c);
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}
