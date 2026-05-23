use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Capitalizes the first letter of each token (or only the first word).
/// Useful for building nice-looking facet parameters.
#[derive(Clone, Debug)]
pub struct CapitalizationTokenFilter {
    pub only_first_word: bool,
    pub min_word_length: usize,
    pub force_first_letter: bool,
}

impl CapitalizationTokenFilter {
    pub fn new() -> Self {
        Self {
            only_first_word: true,
            min_word_length: 0,
            force_first_letter: true,
        }
    }

    pub fn capitalize_all() -> Self {
        Self {
            only_first_word: false,
            min_word_length: 0,
            force_first_letter: true,
        }
    }
}

impl Default for CapitalizationTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for CapitalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }
        let mut result = String::with_capacity(text.len());
        if self.only_first_word {
            let mut chars = text.chars();
            if let Some(first) = chars.next() {
                for c in first.to_uppercase() {
                    result.push(c);
                }
                for c in chars {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
            }
        } else {
            let mut capitalize_next = true;
            for c in text.chars() {
                if c.is_whitespace() {
                    result.push(c);
                    capitalize_next = true;
                } else if capitalize_next {
                    for uc in c.to_uppercase() {
                        result.push(uc);
                    }
                    capitalize_next = false;
                } else {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
            }
        }
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
