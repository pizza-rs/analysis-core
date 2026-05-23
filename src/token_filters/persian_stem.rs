use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Persian/Farsi suffix stemmer.
///
/// Removes common Persian suffixes (plural markers, indefinite markers, etc.).
#[derive(Clone, Debug, Default)]
pub struct PersianStemTokenFilter;

impl PersianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for PersianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 4 {
            return (false, None);
        }

        if let Some(new_len) = stem_persian(&chars, len) {
            if new_len < len && new_len >= 2 {
                let stemmed: String = chars[..new_len].iter().collect();
                token.term = Cow::Owned(stemmed);
            }
        }

        (false, None)
    }
}

fn stem_persian(chars: &[char], len: usize) -> Option<usize> {
    // 4-char suffixes
    if len > 5 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "ترین" | "هایی" => return Some(len - 4),
            _ => {}
        }
    }

    // 3-char suffixes
    if len > 4 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ها\u{06CC}" | "تر\u{06CC}" | "ات\u{06CC}" | "ان\u{06CC}" => {
                return Some(len - 3)
            }
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 3 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "ها" | // ها (ha-alef, plural)
            "ان" | // ان (alef-nun, plural)
            "ات" | // ات (alef-ta, plural)
            "تر" | // تر (comparative)
            "ین" | // ین (ya-nun)
            "ی\u{06CC}" | // یی
            "\u{06CC}\u{06CC}" => return Some(len - 2),
            _ => {}
        }
    }

    // 1-char suffixes
    if len > 3 {
        let last = chars[len - 1];
        match last {
            '\u{06CC}' | // ی (ya)
            '\u{0647}' | // ه (ha)  
            '\u{0627}'   // ا (alef)
            => return Some(len - 1),
            _ => {}
        }
    }

    None
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
    fn test_persian_plural_ha() {
        let filter = PersianStemTokenFilter::new();
        // کتابها → کتاب (remove ها)
        let mut token = make_token("کتابها");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "کتاب");
    }

    #[test]
    fn test_short_word() {
        let filter = PersianStemTokenFilter::new();
        let mut token = make_token("من");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "من");
    }
}
