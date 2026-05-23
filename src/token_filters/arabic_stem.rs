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

fn stem_arabic(chars: &[char]) -> String {
    let mut result: Vec<char> = chars.to_vec();

    // Remove definite article ال (alef-lam) at the beginning
    if result.len() > 4 && result[0] == '\u{0627}' && result[1] == '\u{0644}' {
        result = result[2..].to_vec();
    }
    // Remove وال (waw-alef-lam) at the beginning
    else if result.len() > 5
        && result[0] == '\u{0648}'
        && result[1] == '\u{0627}'
        && result[2] == '\u{0644}'
    {
        result = result[3..].to_vec();
    }
    // Remove بال (ba-alef-lam) at the beginning
    else if result.len() > 5
        && result[0] == '\u{0628}'
        && result[1] == '\u{0627}'
        && result[2] == '\u{0644}'
    {
        result = result[3..].to_vec();
    }
    // Remove كال (kaf-alef-lam) at the beginning
    else if result.len() > 5
        && result[0] == '\u{0643}'
        && result[1] == '\u{0627}'
        && result[2] == '\u{0644}'
    {
        result = result[3..].to_vec();
    }
    // Remove فال (fa-alef-lam)
    else if result.len() > 5
        && result[0] == '\u{0641}'
        && result[1] == '\u{0627}'
        && result[2] == '\u{0644}'
    {
        result = result[3..].to_vec();
    }
    // Remove لل (lam-lam) at the beginning
    else if result.len() > 4 && result[0] == '\u{0644}' && result[1] == '\u{0644}' {
        result = result[2..].to_vec();
    }
    // Remove single-char prefixes: و (waw), ب (ba), ك (kaf), ف (fa)
    else if result.len() > 3 {
        match result[0] {
            '\u{0648}' | '\u{0628}' | '\u{0643}' | '\u{0641}' => {
                result = result[1..].to_vec();
            }
            _ => {}
        }
    }

    // Remove suffixes
    let len = result.len();
    if len > 3 {
        // 2-char suffixes
        if len > 4 {
            let s2: String = result[len - 2..].iter().collect();
            match s2.as_str() {
                "\u{0627}\u{062A}" | // ات (alef-ta)
                "\u{0648}\u{0646}" | // ون (waw-nun)
                "\u{064A}\u{0646}" | // ين (ya-nun)
                "\u{064A}\u{0629}" | // ية (ya-ta_marbuta)
                "\u{064A}\u{0627}" | // يا (ya-alef)
                "\u{0648}\u{0627}"   // وا (waw-alef)
                => {
                    result.truncate(len - 2);
                    return result.iter().collect();
                }
                _ => {}
            }
        }

        // 1-char suffixes
        let last = result[result.len() - 1];
        match last {
            '\u{0629}' | // ة (ta marbuta)
            '\u{0647}' | // ه (ha)
            '\u{064A}'   // ي (ya)
            => {
                result.truncate(result.len() - 1);
            }
            _ => {}
        }
    }

    result.iter().collect()
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
