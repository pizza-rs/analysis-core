use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Portuguese minimal stemmer (plural reduction only).
/// Implements the RSLP-S algorithm (plural step of RSLP) by Orengo et al.
#[derive(Clone, Debug)]
pub struct PortugueseMinimalStemTokenFilter;

impl PortugueseMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PortugueseMinimalStemTokenFilter {
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

impl TokenFilter for PortugueseMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 4 {
            return (false, None);
        }

        let new_len = if len >= 5 && ends_with_str(&chars[..len], "ões") {
            // ões → ão
            let nl = len - 3;
            chars.truncate(nl);
            chars.push('ã');
            chars.push('o');
            nl + 2
        } else if len >= 4 && ends_with_str(&chars[..len], "ães") {
            // ães → ão
            let nl = len - 3;
            chars.truncate(nl);
            chars.push('ã');
            chars.push('o');
            nl + 2
        } else if len >= 4 && ends_with_str(&chars[..len], "ais") {
            // ais → al
            let nl = len - 2;
            chars.truncate(nl);
            chars.push('l');
            nl + 1
        } else if len >= 4 && ends_with_str(&chars[..len], "éis") {
            // éis → el
            let nl = len - 3;
            chars.truncate(nl);
            chars.push('e');
            chars.push('l');
            nl + 2
        } else if len >= 4 && ends_with_str(&chars[..len], "eis") {
            // eis → el
            let nl = len - 2;
            chars.truncate(nl);
            chars.push('l');
            nl + 1
        } else if len >= 4 && ends_with_str(&chars[..len], "óis") {
            // óis → ol
            let nl = len - 3;
            chars.truncate(nl);
            chars.push('o');
            chars.push('l');
            nl + 2
        } else if len >= 4 && ends_with_str(&chars[..len], "is") {
            // is → il  (if preceded by a vowel context, else just remove s)
            let nl = len - 1;
            chars.truncate(nl);
            nl
        } else if len >= 4 && ends_with_str(&chars[..len], "ns") {
            // ns → m
            let nl = len - 2;
            chars.truncate(nl);
            chars.push('m');
            nl + 1
        } else if ends_with_str(&chars[..len], "s") {
            // generic: remove final 's'
            len - 1
        } else {
            len
        };

        let result: String = chars[..new_len.min(chars.len())].iter().collect();
        if result != text {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
