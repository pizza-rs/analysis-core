use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Lightweight Bengali suffix stemmer.
///
/// Removes common Bengali suffixes (case markers, plural markers, verb endings).
#[derive(Clone, Debug, Default)]
pub struct BengaliStemTokenFilter;

impl BengaliStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BengaliStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 4 {
            return (false, None);
        }

        if let Some(new_len) = stem_bengali(&chars, len) {
            if new_len < len && new_len >= 2 {
                let stemmed: String = chars[..new_len].iter().collect();
                token.term = Cow::Owned(stemmed);
            }
        }

        (false, None)
    }
}

fn stem_bengali(chars: &[char], len: usize) -> Option<usize> {
    // 4-char suffixes
    if len > 5 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "গুলি" | "গুলো" | "দের" | "েদের" => return Some(len - 4),
            _ => {}
        }
    }

    // 3-char suffixes
    if len > 4 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "গুল" | "েরা" | "তে" | "দের" | "কে\u{09B0}" => {
                return Some(len - 3)
            }
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 3 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "ের" | "তে" | "কে" | "রা" | "তা" | "য়ে" | "না" | "সে" => {
                return Some(len - 2)
            }
            _ => {}
        }
    }

    // 1-char suffixes (vowel signs)
    if len > 2 {
        let last = chars[len - 1];
        match last {
            '\u{09BE}' | // া
            '\u{09BF}' | // ি
            '\u{09C0}' | // ী
            '\u{09C1}' | // ু
            '\u{09C2}' | // ূ
            '\u{09C7}' | // ে
            '\u{09C8}' | // ৈ
            '\u{09CB}' | // ো
            '\u{09CC}' | // ৌ
            '\u{09CD}'   // ্ (virama)
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
    fn test_bengali_stem_suffix() {
        let filter = BengaliStemTokenFilter::new();
        // বাড়ির → remove ের suffix
        let mut token = make_token("ছেলেদের");
        filter.filter(&mut token);
        // Should remove a suffix
        assert!(token.term.as_ref().len() < "ছেলেদের".len());
    }

    #[test]
    fn test_short_word() {
        let filter = BengaliStemTokenFilter::new();
        let mut token = make_token("কি");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "কি");
    }
}
