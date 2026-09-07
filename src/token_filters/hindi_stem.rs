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
    if len > 6
        && (ends_with_hi(chars, len, "ाइयाँ")
            || ends_with_hi(chars, len, "ाइयां")
            || ends_with_hi(chars, len, "ाइयों")
            || ends_with_hi(chars, len, "ाऊंगा")
            || ends_with_hi(chars, len, "ाऊंगी")
            || ends_with_hi(chars, len, "ाएंगी")
            || ends_with_hi(chars, len, "ाएंगे"))
    {
        return Some(len - 5);
    }
    // 4-char suffixes
    if len > 5
        && (ends_with_hi(chars, len, "एंगी")
            || ends_with_hi(chars, len, "एंगे")
            || ends_with_hi(chars, len, "ताएं")
            || ends_with_hi(chars, len, "ताओं")
            || ends_with_hi(chars, len, "नाएं")
            || ends_with_hi(chars, len, "नाओं")
            || ends_with_hi(chars, len, "ाएगा")
            || ends_with_hi(chars, len, "ाएगी")
            || ends_with_hi(chars, len, "ाओगी")
            || ends_with_hi(chars, len, "ाओगे")
            || ends_with_hi(chars, len, "ातीं")
            || ends_with_hi(chars, len, "ियाँ")
            || ends_with_hi(chars, len, "ियां")
            || ends_with_hi(chars, len, "ियों")
            || ends_with_hi(chars, len, "ूंगा")
            || ends_with_hi(chars, len, "ूंगी")
            || ends_with_hi(chars, len, "ेंगी")
            || ends_with_hi(chars, len, "ेंगे"))
    {
        return Some(len - 4);
    }
    // 3-char suffixes
    if len > 4
        && (ends_with_hi(chars, len, "तीं")
            || ends_with_hi(chars, len, "ाइए")
            || ends_with_hi(chars, len, "ाईं")
            || ends_with_hi(chars, len, "ाएं")
            || ends_with_hi(chars, len, "ाओं")
            || ends_with_hi(chars, len, "ाकर")
            || ends_with_hi(chars, len, "ाता")
            || ends_with_hi(chars, len, "ाती")
            || ends_with_hi(chars, len, "ाते")
            || ends_with_hi(chars, len, "ाना")
            || ends_with_hi(chars, len, "ाने")
            || ends_with_hi(chars, len, "ाया")
            || ends_with_hi(chars, len, "ुआं")
            || ends_with_hi(chars, len, "ुएं")
            || ends_with_hi(chars, len, "ुओं")
            || ends_with_hi(chars, len, "ेगा")
            || ends_with_hi(chars, len, "ेगी")
            || ends_with_hi(chars, len, "ोगी")
            || ends_with_hi(chars, len, "ोगे"))
    {
        return Some(len - 3);
    }
    // 2-char suffixes
    if len > 3
        && (ends_with_hi(chars, len, "कर")
            || ends_with_hi(chars, len, "ता")
            || ends_with_hi(chars, len, "ती")
            || ends_with_hi(chars, len, "ते")
            || ends_with_hi(chars, len, "ना")
            || ends_with_hi(chars, len, "नी")
            || ends_with_hi(chars, len, "ने")
            || ends_with_hi(chars, len, "ाँ")
            || ends_with_hi(chars, len, "ां")
            || ends_with_hi(chars, len, "ाई")
            || ends_with_hi(chars, len, "ाए")
            || ends_with_hi(chars, len, "ाओ")
            || ends_with_hi(chars, len, "िए")
            || ends_with_hi(chars, len, "ीं")
            || ends_with_hi(chars, len, "ें")
            || ends_with_hi(chars, len, "ों"))
    {
        return Some(len - 2);
    }
    // 1-char suffixes
    if len > 2
        && (ends_with_hi(chars, len, "ा")
            || ends_with_hi(chars, len, "ि")
            || ends_with_hi(chars, len, "ी")
            || ends_with_hi(chars, len, "ु")
            || ends_with_hi(chars, len, "ू")
            || ends_with_hi(chars, len, "े")
            || ends_with_hi(chars, len, "ो"))
    {
        return Some(len - 1);
    }
    None
}

fn ends_with_hi(s: &[char], len: usize, suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    len >= suffix.len() && &s[len - suffix.len()..len] == suffix.as_slice()
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
