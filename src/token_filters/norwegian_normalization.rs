use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Norwegian normalization filter.
/// Normalizes interchangeable Scandinavian characters specifically for Norwegian:
/// æÆäÄ→æÆ, öÖøØ→øØ, aa→å (Norwegian-specific foldings).
#[derive(Clone, Debug)]
pub struct NorwegianNormalizationTokenFilter;

impl NorwegianNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NorwegianNormalizationTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for NorwegianNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let mut changed = false;
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;
        while i < len {
            let c = chars[i];
            match c {
                'ä' | 'Ä' => {
                    result.push(if c.is_uppercase() { 'Æ' } else { 'æ' });
                    changed = true;
                }
                'ö' | 'Ö' => {
                    result.push(if c.is_uppercase() { 'Ø' } else { 'ø' });
                    changed = true;
                }
                'a' | 'A' => {
                    // Check for 'aa' → 'å'
                    if i + 1 < len && chars[i + 1].to_ascii_lowercase() == 'a' {
                        result.push(if c.is_uppercase() { 'Å' } else { 'å' });
                        i += 1; // skip the second 'a'
                        changed = true;
                    } else {
                        result.push(c);
                    }
                }
                'æ' if i + 1 < len && chars[i + 1] == 'e' => {
                    // 'æe' is not a valid combo, keep æ, skip e
                    result.push(c);
                    // actually just keep as-is
                    i += 1;
                    result.push(chars[i]);
                }
                _ => {
                    result.push(c);
                }
            }
            i += 1;
        }
        if changed {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
