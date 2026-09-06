use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Generates shingles (token n-grams) of a fixed size from the token stream.
/// Unlike the full ShingleFilter, this creates shingles of exactly `shingle_size` tokens.
#[derive(Clone, Debug)]
pub struct FixedShingleTokenFilter {
    pub shingle_size: usize,
    pub separator: char,
    pub output_unigrams: bool,
}

impl FixedShingleTokenFilter {
    pub fn new(shingle_size: usize) -> Self {
        Self {
            shingle_size,
            separator: ' ',
            output_unigrams: false,
        }
    }
}

impl Default for FixedShingleTokenFilter {
    fn default() -> Self {
        Self::new(2)
    }
}

impl TokenFilter for FixedShingleTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Fixed shingle generation requires multi-token context;
        // in single-token mode this serves as a config marker.
        // Pipeline-level code should use the helper function.
        let _ = self.shingle_size;
        (false, None)
    }
}

/// Helper: builds shingles from a slice of tokens.
pub fn build_fixed_shingles<'a>(
    tokens: &[Token<'a>],
    size: usize,
    separator: char,
) -> Vec<Token<'a>> {
    if tokens.len() < size {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(tokens.len() - size + 1);
    for window in tokens.windows(size) {
        let mut text = String::new();
        for (i, t) in window.iter().enumerate() {
            if i > 0 {
                text.push(separator);
            }
            text.push_str(&t.term);
        }
        let start = window.first().map(|t| t.start_offset).unwrap_or(0);
        let end = window.last().map(|t| t.end_offset).unwrap_or(0);
        result.push(Token {
            term: Cow::Owned(text),
            start_offset: start,
            end_offset: end,
            position: window[0].position,
        });
    }
    result
}
