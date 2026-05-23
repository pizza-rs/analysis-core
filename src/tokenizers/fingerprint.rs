use alloc::borrow::Cow;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Produces a single "fingerprint" token from the input text.
///
/// The fingerprint is created by:
/// 1. Lowercasing
/// 2. Tokenizing on whitespace/punctuation
/// 3. Sorting unique tokens alphabetically
/// 4. Joining with a single space
///
/// Useful for deduplication and fuzzy matching where word order doesn't matter.
/// Example: `"The Quick Brown Fox"` → `"brown fox quick the"`
#[derive(Clone, Debug)]
pub struct FingerprintTokenizer {
    /// Maximum number of tokens to include in fingerprint
    max_tokens: usize,
    /// Separator between tokens in the fingerprint
    separator: String,
}

impl FingerprintTokenizer {
    pub fn new() -> Self {
        Self {
            max_tokens: 100,
            separator: String::from(" "),
        }
    }

    pub fn with_max_tokens(mut self, n: usize) -> Self {
        self.max_tokens = n;
        self
    }

    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = String::from(sep);
        self
    }
}

impl Default for FingerprintTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for FingerprintTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        if text.is_empty() {
            return Vec::new();
        }

        // Extract words (lowercased), dedup + sort in one step via BTreeSet.
        // Avoids the previous O(n²) `Vec::contains` scan.
        let lower = text.to_lowercase();
        let mut unique: BTreeSet<String> = BTreeSet::new();
        for word in lower.split(|c: char| !c.is_alphanumeric()) {
            if !word.is_empty() {
                unique.insert(String::from(word));
            }
        }

        let mut words: Vec<String> = unique.into_iter().collect();
        words.truncate(self.max_tokens);

        let fingerprint = words.join(&self.separator);

        if fingerprint.is_empty() {
            return Vec::new();
        }

        vec![Token {
            term: Cow::Owned(fingerprint),
            start_offset: 0,
            end_offset: text.len() as u32,
            position: 0,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_fingerprint() {
        let tok = FingerprintTokenizer::new();
        let tokens = tok.tokenize("The Quick Brown Fox");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "brown fox quick the");
    }

    #[test]
    fn test_duplicate_removal() {
        let tok = FingerprintTokenizer::new();
        let tokens = tok.tokenize("hello hello world world");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "hello world");
    }

    #[test]
    fn test_punctuation_stripped() {
        let tok = FingerprintTokenizer::new();
        let tokens = tok.tokenize("hello, world! foo.");
        assert_eq!(tokens[0].term.as_ref(), "foo hello world");
    }

    #[test]
    fn test_empty() {
        let tok = FingerprintTokenizer::new();
        let tokens = tok.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_custom_separator() {
        let tok = FingerprintTokenizer::new().with_separator("_");
        let tokens = tok.tokenize("b a c");
        assert_eq!(tokens[0].term.as_ref(), "a_b_c");
    }
}
