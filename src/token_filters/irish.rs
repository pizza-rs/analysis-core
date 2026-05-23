use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Irish-specific elision filter.
///
/// Removes Irish elision prefixes (d', n-, t-, b', m', h-) that appear
/// before vowels in Irish/Gaelic.
#[derive(Clone, Debug, Default)]
pub struct IrishElisionTokenFilter;

impl IrishElisionTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for IrishElisionTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 3 {
            return (false, None);
        }

        // Check for Irish elision prefixes
        let strip_len = get_irish_elision_length(&chars);
        if strip_len > 0 && strip_len < len {
            let new_text: String = chars[strip_len..].iter().collect();
            token.term = Cow::Owned(new_text);
        }

        (false, None)
    }
}

fn get_irish_elision_length(chars: &[char]) -> usize {
    if chars.len() < 2 {
        return 0;
    }

    // d' prefix (d'fhocal)
    if chars[0] == 'd' && chars[1] == '\'' {
        return 2;
    }
    if chars[0] == 'D' && chars[1] == '\'' {
        return 2;
    }

    // m' prefix (m'athair)
    if chars[0] == 'm' && chars[1] == '\'' {
        return 2;
    }
    if chars[0] == 'M' && chars[1] == '\'' {
        return 2;
    }

    // b' prefix (b'fhéidir)
    if chars[0] == 'b' && chars[1] == '\'' {
        return 2;
    }
    if chars[0] == 'B' && chars[1] == '\'' {
        return 2;
    }

    // n- prefix (n-uisce)
    if chars[0] == 'n' && chars[1] == '-' {
        return 2;
    }

    // t- prefix (t-ainm)
    if chars[0] == 't' && chars[1] == '-' {
        return 2;
    }

    // h- prefix (h-Éire → h prefix before vowel)
    if chars[0] == 'h' && chars[1] == '-' {
        return 2;
    }

    0
}

/// Irish-specific lowercase filter.
///
/// Handles Irish eclipsis mutations (nDúnadh → dúnadh, bPáirc → páirc, etc.)
/// by lowercasing while preserving the mutation prefix appropriately.
#[derive(Clone, Debug, Default)]
pub struct IrishLowercaseTokenFilter;

impl IrishLowercaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for IrishLowercaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();

        // Check for eclipsis patterns and handle specially
        if let Some(lowered) = irish_lowercase(text) {
            token.term = Cow::Owned(lowered);
        } else {
            let lower = text.to_lowercase();
            if lower != text {
                token.term = Cow::Owned(lower);
            }
        }

        (false, None)
    }
}

/// Irish eclipsis lowercasing.
/// In Irish, eclipsis prefixes are: mb, gc, nd, bhf, ng, bp, dt, n-
/// When lowercasing, we need to handle these properly.
fn irish_lowercase(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return None;
    }

    // Eclipsis patterns: the prefix consonant(s) + the original initial letter
    // nD → nd, mB → mb, gC → gc, nG → ng, bP → bp, dT → dt, bhF → bhf
    let first = chars[0];
    let second = chars[1];

    // Check if this is an eclipsis pattern (lowercase prefix + uppercase base)
    let is_eclipsis = match (first, second) {
        ('n', c) if c.is_uppercase() => true,
        ('m', 'B') => true,
        ('g', 'C') => true,
        ('b', c) if c == 'P' || (c == 'h' && chars.len() > 2 && chars[2] == 'F') => {
            true
        }
        ('d', 'T') => true,
        _ => false,
    };

    if is_eclipsis {
        // Just lowercase the whole thing
        Some(text.to_lowercase())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_irish_elision_d() {
        let filter = IrishElisionTokenFilter::new();
        let mut token = make_token("d'fhocal");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "fhocal");
    }

    #[test]
    fn test_irish_elision_n() {
        let filter = IrishElisionTokenFilter::new();
        let mut token = make_token("n-uisce");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "uisce");
    }

    #[test]
    fn test_irish_lowercase_eclipsis() {
        let filter = IrishLowercaseTokenFilter::new();
        let mut token = make_token("nDúnadh");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "ndúnadh");
    }

    #[test]
    fn test_irish_lowercase_normal() {
        let filter = IrishLowercaseTokenFilter::new();
        let mut token = make_token("Éire");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "éire");
    }
}
