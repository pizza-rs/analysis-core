//! Token filter that keeps or discards tokens based on character type detection.
//!
//! Equivalent to Elasticsearch's `keep_types` filter. Useful for filtering
//! out numeric tokens, keeping only alphabetic tokens, etc.

use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// The mode of the filter: keep matching types or exclude them.
#[derive(Clone, Debug, PartialEq)]
pub enum KeepTypesMode {
    /// Only keep tokens of the specified types.
    Include,
    /// Discard tokens of the specified types.
    Exclude,
}

/// Token type categories for filtering.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenType {
    /// Alphabetic characters only (Unicode letter category)
    Alpha,
    /// Numeric characters only
    Numeric,
    /// Alphanumeric (mixed letters and digits)
    AlphaNumeric,
    /// CJK ideographs
    Cjk,
    /// Hangul (Korean)
    Hangul,
    /// Katakana (Japanese)
    Katakana,
    /// Hiragana (Japanese)
    Hiragana,
}

impl TokenType {
    fn matches(&self, term: &str) -> bool {
        if term.is_empty() {
            return false;
        }
        match self {
            Self::Alpha => term.chars().all(|c| c.is_alphabetic()),
            Self::Numeric => term.chars().all(|c| c.is_ascii_digit() || c == '.'),
            Self::AlphaNumeric => {
                term.chars().all(|c| c.is_alphanumeric())
                    && term.chars().any(|c| c.is_alphabetic())
                    && term.chars().any(|c| c.is_ascii_digit())
            }
            Self::Cjk => term.chars().all(|c| is_cjk(c)),
            Self::Hangul => term.chars().all(|c| is_hangul(c)),
            Self::Katakana => term.chars().all(|c| is_katakana(c)),
            Self::Hiragana => term.chars().all(|c| is_hiragana(c)),
        }
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF | 0xF900..=0xFAFF)
}

fn is_hangul(c: char) -> bool {
    matches!(c as u32, 0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F)
}

fn is_katakana(c: char) -> bool {
    matches!(c as u32, 0x30A0..=0x30FF | 0x31F0..=0x31FF | 0xFF65..=0xFF9F)
}

fn is_hiragana(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x309F)
}

/// Filters tokens based on their detected character type.
///
/// # Example
///
/// ```rust
/// use pizza_analysis_core::KeepTypesMode;
/// use pizza_analysis_core::KeepTypesTokenFilter;
/// use pizza_analysis_core::TokenType;
///
/// // Only keep alphabetic tokens (discard numbers, mixed, etc.)
/// let filter = KeepTypesTokenFilter::new(vec![TokenType::Alpha], KeepTypesMode::Include);
/// ```
#[derive(Clone)]
pub struct KeepTypesTokenFilter {
    types: Vec<TokenType>,
    mode: KeepTypesMode,
}

impl KeepTypesTokenFilter {
    pub fn new(types: Vec<TokenType>, mode: KeepTypesMode) -> Self {
        Self { types, mode }
    }

    /// Create a filter that keeps only alphabetic tokens.
    pub fn alpha_only() -> Self {
        Self::new(vec![TokenType::Alpha], KeepTypesMode::Include)
    }

    /// Create a filter that excludes numeric tokens.
    pub fn exclude_numeric() -> Self {
        Self::new(vec![TokenType::Numeric], KeepTypesMode::Exclude)
    }
}

impl TokenFilter for KeepTypesTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let matches_any = self.types.iter().any(|t| t.matches(term));

        let should_delete = match self.mode {
            KeepTypesMode::Include => !matches_any,
            KeepTypesMode::Exclude => matches_any,
        };

        (should_delete, None)
    }
}
