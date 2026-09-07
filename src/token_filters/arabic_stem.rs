use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Arabic light stemmer that removes common prefixes and suffixes.
///
/// Based on Lucene's ArabicStemmer algorithm which removes:
/// - Definite article (ال)
/// - Common prefixes (و, ب, ك, ف, لل)
/// - Common suffixes (ة, ه, ي, ية, ات, ون, ين, وا)
#[derive(Clone, Debug, Default)]
pub struct ArabicStemTokenFilter;

impl ArabicStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for ArabicStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 4 {
            return (false, None);
        }

        let stemmed = stem_arabic(&chars);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }

        (false, None)
    }
}

/// Lucene ArabicStemmer — faithful port of
/// `org.apache.lucene.analysis.ar.ArabicStemmer`: strip one prefix, then
/// suffixes (each suffix requires ≥2 chars to remain), iterating the
/// suffix table so multiple suffixes can come off.
fn stem_arabic(chars: &[char]) -> String {
    // ( Alef, Lam ), ( Waw, Alef, Lam ), ( Beh, Alef, Lam ),
    // ( Kaf, Alef, Lam ), ( Feh, Alef, Lam ), ( Lam, Lam ), ( Waw )
    const PREFIXES: &[&[char]] = &[
        &['\u{0627}', '\u{0644}'],
        &['\u{0648}', '\u{0627}', '\u{0644}'],
        &['\u{0628}', '\u{0627}', '\u{0644}'],
        &['\u{0643}', '\u{0627}', '\u{0644}'],
        &['\u{0641}', '\u{0627}', '\u{0644}'],
        &['\u{0644}', '\u{0644}'],
        &['\u{0648}'],
    ];
    // ( Heh, Alef ), ( Alef, Noon ), ( Alef, Teh Marbuta ), ( Waw, Noon ),
    // ( Yeh, Noon ), ( Yeh, Heh ), ( Yeh, Teh Marbuta ), ( Heh ),
    // ( Teh Marbuta ), ( Yeh )
    const SUFFIXES: &[&[char]] = &[
        &['\u{0647}', '\u{0627}'],
        &['\u{0627}', '\u{0646}'],
        &['\u{0627}', '\u{062A}'],
        &['\u{0648}', '\u{0646}'],
        &['\u{064A}', '\u{0646}'],
        &['\u{064A}', '\u{0647}'],
        &['\u{064A}', '\u{0629}'],
        &['\u{0647}'],
        &['\u{0629}'],
        &['\u{064A}'],
    ];

    let mut s: Vec<char> = chars.to_vec();
    let mut len = s.len();

    // Prefixes: strip the first matching one. The single-char wa- prefix
    // requires ≥3 remaining characters (Java: len < 4 fails); the others
    // require ≥2.
    for prefix in PREFIXES {
        let passes = if prefix.len() == 1 {
            len >= 4
        } else {
            len >= prefix.len() + 2
        } && s[..prefix.len()] == prefix[..];
        if passes {
            s.drain(..prefix.len());
            len -= prefix.len();
            break;
        }
    }

    // suffixes: iterate the whole table (multiple can strip); each requires
    // ≥2 chars remaining after the cut.
    for suffix in SUFFIXES {
        if len >= suffix.len() + 2 && &s[len - suffix.len()..len] == *suffix {
            len -= suffix.len();
        }
    }

    s.truncate(len);
    s.iter().collect()
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
    fn test_remove_definite_article() {
        let filter = ArabicStemTokenFilter::new();
        // الكتاب → كتاب (remove ال)
        let mut token = make_token("\u{0627}\u{0644}\u{0643}\u{062A}\u{0627}\u{0628}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{0643}\u{062A}\u{0627}\u{0628}");
    }

    #[test]
    fn test_remove_ta_marbuta() {
        let filter = ArabicStemTokenFilter::new();
        // مدرسة → مدرس (remove ة)
        let mut token = make_token("\u{0645}\u{062F}\u{0631}\u{0633}\u{0629}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{0645}\u{062F}\u{0631}\u{0633}");
    }

    #[test]
    fn test_short_word() {
        let filter = ArabicStemTokenFilter::new();
        let mut token = make_token("\u{0641}\u{064A}"); // في (too short)
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{0641}\u{064A}");
    }
}
