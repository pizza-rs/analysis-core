use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Lightweight Hindi suffix stemmer.
///
/// Removes common Hindi suffixes (postpositions, case markers, plural markers).
/// Based on the algorithm from Lucene's HindiStemmer.
#[derive(Clone, Debug, Default)]
pub struct HindiStemTokenFilter;

impl HindiStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for HindiStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 3 {
            return (false, None);
        }

        // Try suffixes of decreasing length
        if let Some(new_len) = stem_hindi(&chars, len) {
            if new_len < len {
                let stemmed: String = chars[..new_len].iter().collect();
                token.term = Cow::Owned(stemmed);
            }
        }

        (false, None)
    }
}

fn stem_hindi(chars: &[char], len: usize) -> Option<usize> {
    // 5-char suffixes
    if len > 6 {
        let suffix: String = chars[len - 5..].iter().collect();
        match suffix.as_str() {
            "ियाँ" | "## ियों" => return Some(len - 5),
            _ => {}
        }
    }

    // 4-char suffixes
    if len > 5 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "िया\u{0901}" | " ## ियो" => return Some(len - 4),
            _ => {}
        }
    }

    // 3-char suffixes
    if len > 4 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ियाँ" | "ाओं" | "ुओं" | "ियों" | "ियाँ" | "ाएँ" | "ाएं" | "ों" => {
                return Some(len - 3)
            }
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 3 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "ों" | "ें" | "ों" | "ो\u{0902}" | "े\u{0902}" | "ा\u{0902}" | "ी\u{0902}" | "िं" | "ाँ"
            | "ि\u{0902}" | "ा\u{0901}" | "ी\u{0901}" | "ों" | "ें" => {
                return Some(len - 2)
            }
            _ => {}
        }
    }

    // 1-char suffixes (matras/vowel signs)
    if len > 2 {
        let last = chars[len - 1];
        match last {
            '\u{093E}' | // ा
            '\u{093F}' | // ि
            '\u{0940}' | // ी
            '\u{0941}' | // ु
            '\u{0942}' | // ू
            '\u{0947}' | // े
            '\u{0948}' | // ै
            '\u{094B}' | // ो
            '\u{094C}' | // ौ
            '\u{094D}'   // ्  (halant/virama)
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
    fn test_hindi_stem_matra() {
        let filter = HindiStemTokenFilter::new();
        // लड़की → remove ी matra
        let mut token = make_token("लड़की");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "लड़क");
    }

    #[test]
    fn test_short_word_no_stem() {
        let filter = HindiStemTokenFilter::new();
        let mut token = make_token("का");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "का"); // too short
    }
}
