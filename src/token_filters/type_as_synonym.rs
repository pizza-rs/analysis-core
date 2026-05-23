use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Emits the detected token type as a synonym token at the same position.
/// Token types are inferred from content: numeric, alpha, alphanumeric, punctuation, etc.
#[derive(Clone, Debug)]
pub struct TypeAsSynonymTokenFilter {
    pub prefix: String,
}

impl TypeAsSynonymTokenFilter {
    pub fn new() -> Self {
        Self {
            prefix: String::from("_type:"),
        }
    }

    pub fn with_prefix(prefix: &str) -> Self {
        Self {
            prefix: String::from(prefix),
        }
    }
}

impl Default for TypeAsSynonymTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_type(term: &str) -> &'static str {
    if term.is_empty() {
        return "empty";
    }
    let mut has_alpha = false;
    let mut has_digit = false;
    let mut has_punct = false;
    let mut has_space = false;
    for c in term.chars() {
        if c.is_alphabetic() {
            has_alpha = true;
        } else if c.is_ascii_digit() {
            has_digit = true;
        } else if c.is_whitespace() {
            has_space = true;
        } else {
            has_punct = true;
        }
    }
    match (has_alpha, has_digit, has_punct, has_space) {
        (true, false, false, false) => "alpha",
        (false, true, false, false) => "numeric",
        (true, true, false, false) => "alphanumeric",
        (false, false, true, false) => "punctuation",
        (true, false, false, true) => "phrase",
        _ => "mixed",
    }
}

impl TokenFilter for TypeAsSynonymTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let type_name = detect_type(token.term.as_ref());
        let synonym = Token {
            term: Cow::Owned(format!("{}{}", self.prefix, type_name)),
            start_offset: token.start_offset,
            end_offset: token.end_offset,
            position: token.position,
        };
        (false, Some(alloc::vec![synonym]))
    }
}
