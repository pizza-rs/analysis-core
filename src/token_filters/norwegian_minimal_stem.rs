use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Norwegian minimal stemmer. Stems known plural forms for Norwegian nouns (Bokmål + Nynorsk).
/// Based on Jacques Savoy's algorithm.
#[derive(Clone, Debug)]
pub struct NorwegianMinimalStemTokenFilter {
    pub use_bokmaal: bool,
    pub use_nynorsk: bool,
}

impl NorwegianMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self {
            use_bokmaal: true,
            use_nynorsk: true,
        }
    }

    pub fn bokmaal_only() -> Self {
        Self {
            use_bokmaal: true,
            use_nynorsk: false,
        }
    }

    pub fn nynorsk_only() -> Self {
        Self {
            use_bokmaal: false,
            use_nynorsk: true,
        }
    }
}

impl Default for NorwegianMinimalStemTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn ends_with_str(chars: &[char], suffix: &str) -> bool {
    let suffix_chars: Vec<char> = suffix.chars().collect();
    if chars.len() < suffix_chars.len() {
        return false;
    }
    let start = chars.len() - suffix_chars.len();
    chars[start..] == suffix_chars[..]
}

impl TokenFilter for NorwegianMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut chars: Vec<char> = text.chars().collect();
        let mut len = chars.len();

        // Remove genitiv -s
        if len > 4 && chars[len - 1] == 's' {
            len -= 1;
        }

        let new_len = if len > 5
            && (ends_with_str(&chars[..len], "ene")
                || (self.use_nynorsk && ends_with_str(&chars[..len], "ane")))
        {
            len - 3
        } else if len > 4
            && (ends_with_str(&chars[..len], "er")
                || ends_with_str(&chars[..len], "en")
                || ends_with_str(&chars[..len], "et")
                || (self.use_nynorsk && ends_with_str(&chars[..len], "ar")))
        {
            len - 2
        } else if len > 3 && (chars[len - 1] == 'a' || chars[len - 1] == 'e') {
            len - 1
        } else {
            len
        };

        if new_len < chars.len() {
            let result: String = chars[..new_len].iter().collect();
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
