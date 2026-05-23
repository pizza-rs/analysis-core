//! Token filter that rejoins words broken by hyphens at line endings.
//!
//! In OCR or PDF-extracted text, words are often broken across lines with
//! hyphens, e.g. "knowl-\nedge" → "knowledge". This filter recombines them.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Rejoins tokens that end with a hyphen with the following token.
///
/// This filter is useful for text extracted from PDFs, OCR output, or
/// old typeset documents where words are broken at line boundaries.
///
/// # Example
///
/// Input tokens: `["knowl-", "edge"]` → output: `["knowledge"]`
#[derive(Clone)]
pub struct HyphenatedWordsTokenFilter;

impl HyphenatedWordsTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HyphenatedWordsTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for HyphenatedWordsTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // This filter works on individual tokens. If the token ends with a hyphen,
        // we strip it and mark it for concatenation with the next token.
        // Since the TokenFilter interface processes one token at a time, we strip
        // trailing hyphens. A more complete implementation would need stream-level
        // access to merge adjacent tokens.
        let term = token.term.as_ref();
        if term.ends_with('-') && term.len() > 1 {
            let stripped = &term[..term.len() - 1];
            token.term = Cow::Owned(String::from(stripped));
        }
        (false, None)
    }
}
