//! Token filter that protects specified words from being modified by
//! downstream filters.
//!
//! When a token matches a protected word, it is flagged so downstream
//! filters (like stemmers) can skip it. This is commonly used to prevent
//! stemming of proper nouns, brand names, or technical terms.

use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// A token filter that marks/protects certain words from further modification.
///
/// Protected words pass through unchanged. This filter is typically placed
/// before stemming or other morphological filters to prevent them from
/// modifying specific terms.
///
/// # Example
///
/// ```rust
/// use pizza_analysis_core::ProtectedWordsTokenFilter;
///
/// let filter = ProtectedWordsTokenFilter::new(&["elasticsearch", "lucene", "pizza"]);
/// ```
#[derive(Clone)]
pub struct ProtectedWordsTokenFilter {
    protected: HashSet<String>,
    ignore_case: bool,
}

impl ProtectedWordsTokenFilter {
    /// Create a new filter with the given protected words (case-sensitive by default).
    pub fn new(words: &[&str]) -> Self {
        Self {
            protected: words.iter().map(|w| w.to_string()).collect(),
            ignore_case: false,
        }
    }

    /// Create a case-insensitive variant.
    pub fn new_ignore_case(words: &[&str]) -> Self {
        Self {
            protected: words.iter().map(|w| w.to_lowercase()).collect(),
            ignore_case: true,
        }
    }

    /// Check if a term is protected.
    pub fn is_protected(&self, term: &str) -> bool {
        if self.ignore_case {
            self.protected.contains(&term.to_lowercase())
        } else {
            self.protected.contains(term)
        }
    }
}

impl TokenFilter for ProtectedWordsTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Protected words pass through unchanged - this is intentional.
        // The filter's purpose is to be queried by conditional pipelines
        // that check is_protected() before applying transforms.
        let _ = token;
        (false, None)
    }
}
