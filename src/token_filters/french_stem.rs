use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// French light stemmer.
///
/// Implements a lightweight suffix-stripping algorithm for French, removing
/// common plural, feminine, and adverbial suffixes.
#[derive(Clone, Debug, Default)]
pub struct FrenchLightStemTokenFilter;

impl FrenchLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for FrenchLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let stemmed = stem_french_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }

        (false, None)
    }
}

/// Lucene FrenchLightStemmer (UniNE algorithm, Jacques Savoy) — faithful
/// port of `org.apache.lucene.analysis.fr.FrenchLightStemmer`. Operates on a
/// char vector with the same length-truncation semantics as the Java
/// original (which mutates a UTF-16 array and returns the new length).
fn stem_french_light(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let mut len = s.len();

    if len > 5 && s[len - 1] == 'x' {
        if s[len - 3] == 'a' && s[len - 2] == 'u' && s[len - 4] != 'e' {
            s[len - 2] = 'l';
        }
        len -= 1;
    }

    if len > 3 && s[len - 1] == 'x' {
        len -= 1;
    }

    if len > 3 && s[len - 1] == 's' {
        len -= 1;
    }

    if len > 9 && ends_with(&s, len, "issement") {
        len -= 6;
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 8 && ends_with(&s, len, "issant") {
        len -= 4;
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 6 && ends_with(&s, len, "ement") {
        len -= 4;
        if len > 3 && ends_with(&s, len, "ive") {
            len -= 1;
            s[len - 1] = 'f';
        }
        return norm(s, len);
    }

    if len > 11 && ends_with(&s, len, "ficatrice") {
        len -= 5;
        s[len - 2] = 'e';
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 10 && ends_with(&s, len, "ficateur") {
        len -= 4;
        s[len - 2] = 'e';
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 9 && ends_with(&s, len, "catrice") {
        len -= 3;
        s[len - 4] = 'q';
        s[len - 3] = 'u';
        s[len - 2] = 'e';
        return norm(s, len);
    }

    if len > 8 && ends_with(&s, len, "cateur") {
        len -= 2;
        s[len - 4] = 'q';
        s[len - 3] = 'u';
        s[len - 2] = 'e';
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 8 && ends_with(&s, len, "atrice") {
        len -= 4;
        s[len - 2] = 'e';
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 7 && ends_with(&s, len, "ateur") {
        len -= 3;
        s[len - 2] = 'e';
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 6 && ends_with(&s, len, "trice") {
        len -= 1;
        s[len - 3] = 'e';
        s[len - 2] = 'u';
        s[len - 1] = 'r';
    }

    if len > 5 && ends_with(&s, len, "ième") {
        return norm(s, len - 4);
    }

    if len > 7 && ends_with(&s, len, "teuse") {
        len -= 2;
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 6 && ends_with(&s, len, "teur") {
        len -= 1;
        s[len - 1] = 'r';
        return norm(s, len);
    }

    if len > 5 && ends_with(&s, len, "euse") {
        return norm(s, len - 2);
    }

    if len > 8 && ends_with(&s, len, "ère") {
        len -= 1;
        s[len - 2] = 'e';
        return norm(s, len);
    }

    if len > 7 && ends_with(&s, len, "ive") {
        len -= 1;
        s[len - 1] = 'f';
        return norm(s, len);
    }

    if len > 4 && (ends_with(&s, len, "folle") || ends_with(&s, len, "molle")) {
        len -= 2;
        s[len - 1] = 'u';
        return norm(s, len);
    }

    if len > 9 && ends_with(&s, len, "nnelle") {
        return norm(s, len - 5);
    }

    if len > 9 && ends_with(&s, len, "nnel") {
        return norm(s, len - 3);
    }

    if len > 4 && ends_with(&s, len, "ète") {
        len -= 1;
        s[len - 2] = 'e';
    }

    if len > 8 && ends_with(&s, len, "ique") {
        len -= 4;
    }

    if len > 8 && ends_with(&s, len, "esse") {
        return norm(s, len - 3);
    }

    if len > 7 && ends_with(&s, len, "inage") {
        return norm(s, len - 3);
    }

    if len > 9 && ends_with(&s, len, "isation") {
        len -= 7;
        if len > 5 && ends_with(&s, len, "ual") {
            s[len - 2] = 'e';
        }
        return norm(s, len);
    }

    if len > 9 && ends_with(&s, len, "isateur") {
        return norm(s, len - 7);
    }

    if len > 8 && ends_with(&s, len, "ation") {
        return norm(s, len - 5);
    }

    if len > 8 && ends_with(&s, len, "ition") {
        return norm(s, len - 5);
    }

    norm(s, len)
}

/// Java `StemmerUtil.endsWith` over the live prefix length.
fn ends_with(s: &[char], len: usize, suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    if len < suffix.len() {
        return false;
    }
    &s[len - suffix.len()..len] == suffix.as_slice()
}

/// Lucene FrenchLightStemmer `norm`: accent folding, double-letter
/// collapse, then -ie/-r/-e stripping.
fn norm(mut s: Vec<char>, mut len: usize) -> String {
    if len > 4 {
        for c in s[..len].iter_mut() {
            match c {
                'à' | 'á' | 'â' => *c = 'a',
                'ô' => *c = 'o',
                'è' | 'é' | 'ê' => *c = 'e',
                'ù' | 'û' => *c = 'u',
                'î' => *c = 'i',
                'ç' => *c = 'c',
                _ => {}
            }
        }

        let mut ch = s[0];
        let mut i = 1;
        while i < len {
            if s[i] == ch && ch.is_alphabetic() {
                s.remove(i);
                len -= 1;
            } else {
                ch = s[i];
                i += 1;
            }
        }
    }

    if len > 4 && ends_with(&s, len, "ie") {
        len -= 2;
    }

    if len > 4 {
        if s[len - 1] == 'r' {
            len -= 1;
        }
        if s[len - 1] == 'e' {
            len -= 1;
        }
        if s[len - 1] == s[len - 2] && s[len - 1].is_alphabetic() {
            len -= 1;
        }
    }

    s[..len].iter().collect()
}

/// French minimal stemmer.
///
/// Only removes plural/feminine markers without deeper stemming.
#[derive(Clone, Debug, Default)]
pub struct FrenchMinimalStemTokenFilter;

impl FrenchMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for FrenchMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let len = text.len();

        if len < 4 {
            return (false, None);
        }

        let stemmed = stem_french_minimal(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }

        (false, None)
    }
}

fn stem_french_minimal(word: &str) -> String {
    let mut result = String::from(word);

    // Remove plural markers
    if result.ends_with("aux") {
        result.truncate(result.len() - 3);
        result.push_str("al");
        return result;
    }

    if result.ends_with('s') || result.ends_with('x') {
        result.pop();
    }

    result
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
    fn test_french_light_plural() {
        let filter = FrenchLightStemTokenFilter::new();
        let mut token = make_token("chateaux");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "chateau");
    }

    #[test]
    fn test_french_minimal_aux() {
        let filter = FrenchMinimalStemTokenFilter::new();
        let mut token = make_token("chevaux");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "cheval");
    }
}
