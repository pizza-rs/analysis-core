use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Czech light stemmer based on the Dolamic/Savoy algorithm.
///
/// Removes common Czech suffixes (case, number, derivational).
#[derive(Clone, Debug, Default)]
pub struct CzechStemTokenFilter;

impl CzechStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for CzechStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        // Lucene's CzechAnalyzer lowercases before stemming; the stemmer
        // itself requires lowercase input (diacritics preserved).
        let stemmed = stem_czech(&text.to_lowercase());
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

/// Lucene CzechStemmer — faithful port of
/// `org.apache.lucene.analysis.cz.CzechStemmer` (aggressive stemmer).
/// Input is expected lowercase with diacritics preserved.
fn stem_czech(word: &str) -> String {
    let mut s: Vec<char> = word.chars().collect();
    let n = s.len();
    let len = stem_czech_chars(&mut s, n);
    s[..len].iter().collect()
}

fn stem_czech_chars(s: &mut [char], len: usize) -> usize {
    let len = remove_case(s, len);
    let len = remove_possessives(s, len);
    if len > 0 {
        normalize(s, len)
    } else {
        len
    }
}

fn ends_with(s: &[char], len: usize, suffix: &str) -> bool {
    let suffix: Vec<char> = suffix.chars().collect();
    len >= suffix.len() && &s[len - suffix.len()..len] == suffix.as_slice()
}

fn remove_case(s: &[char], len: usize) -> usize {
    if len > 7 && ends_with(s, len, "atech") {
        return len - 5;
    }

    if len > 6
        && (ends_with(s, len, "ětem") || ends_with(s, len, "etem") || ends_with(s, len, "atům"))
    {
        return len - 4;
    }

    if len > 5
        && (ends_with(s, len, "ech")
            || ends_with(s, len, "ich")
            || ends_with(s, len, "ích")
            || ends_with(s, len, "ého")
            || ends_with(s, len, "ěmi")
            || ends_with(s, len, "emi")
            || ends_with(s, len, "ému")
            || ends_with(s, len, "ěte")
            || ends_with(s, len, "ete")
            || ends_with(s, len, "ěti")
            || ends_with(s, len, "eti")
            || ends_with(s, len, "ího")
            || ends_with(s, len, "iho")
            || ends_with(s, len, "ími")
            || ends_with(s, len, "ímu")
            || ends_with(s, len, "imu")
            || ends_with(s, len, "ách")
            || ends_with(s, len, "ata")
            || ends_with(s, len, "aty")
            || ends_with(s, len, "ých")
            || ends_with(s, len, "ama")
            || ends_with(s, len, "ami")
            || ends_with(s, len, "ové")
            || ends_with(s, len, "ovi")
            || ends_with(s, len, "ými"))
    {
        return len - 3;
    }

    if len > 4
        && (ends_with(s, len, "em")
            || ends_with(s, len, "es")
            || ends_with(s, len, "ém")
            || ends_with(s, len, "ím")
            || ends_with(s, len, "ům")
            || ends_with(s, len, "at")
            || ends_with(s, len, "ám")
            || ends_with(s, len, "os")
            || ends_with(s, len, "us")
            || ends_with(s, len, "ým")
            || ends_with(s, len, "mi")
            || ends_with(s, len, "ou"))
    {
        return len - 2;
    }

    if len > 3 {
        match s[len - 1] {
            'a' | 'e' | 'i' | 'o' | 'u' | 'ů' | 'y' | 'á' | 'é' | 'í' | 'ý' | 'ě' => {
                return len - 1;
            }
            _ => {}
        }
    }

    len
}

fn remove_possessives(s: &[char], len: usize) -> usize {
    if len > 5 && (ends_with(s, len, "ov") || ends_with(s, len, "in") || ends_with(s, len, "ův")) {
        return len - 2;
    }
    len
}

fn normalize(s: &mut [char], len: usize) -> usize {
    if ends_with(s, len, "čt") {
        // čt -> ck
        s[len - 2] = 'c';
        s[len - 1] = 'k';
        return len;
    }

    if ends_with(s, len, "št") {
        // št -> sk
        s[len - 2] = 's';
        s[len - 1] = 'k';
        return len;
    }

    match s[len - 1] {
        // [cč] -> k
        'c' | 'č' => {
            s[len - 1] = 'k';
            return len;
        }
        // [zž] -> h
        'z' | 'ž' => {
            s[len - 1] = 'h';
            return len;
        }
        _ => {}
    }

    if len > 1 && s[len - 2] == 'e' {
        // e* > *
        s[len - 2] = s[len - 1];
        return len - 1;
    }

    if len > 2 && s[len - 2] == 'ů' {
        // *ů* -> *o*
        s[len - 2] = 'o';
        return len;
    }

    len
}

/// Norwegian light stemmer.
///
/// Handles both Bokmål and Nynorsk variants.
#[derive(Clone, Debug, Default)]
pub struct NorwegianLightStemTokenFilter;

impl NorwegianLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for NorwegianLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_norwegian_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_norwegian_light(word: &str) -> String {
    let mut result = String::from(word);

    // 3-char suffixes
    if result.len() > 5 {
        let len = result.len();
        let suffix = &result[len - 3..];
        match suffix {
            "ene" | "ede" | "ane" | "ede" | "het" | "ert" => {
                result.truncate(len - 3);
                return result;
            }
            _ => {}
        }
    }

    // 2-char suffixes
    if result.len() > 4 {
        let len = result.len();
        let suffix = &result[len - 2..];
        match suffix {
            "er" | "en" | "et" | "ar" | "as" | "es" | "eg" => {
                result.truncate(len - 2);
                return result;
            }
            _ => {}
        }
    }

    // 1-char suffixes
    if result.len() > 3 {
        let last = result.chars().last().unwrap();
        match last {
            'a' | 'e' | 's' => {
                result.pop();
            }
            _ => {}
        }
    }

    result
}

/// Dutch stemming lives in `dutch_stem.rs` (full Snowball port); re-exported
/// here for the analyzer registry.
pub use super::dutch_stem::DutchStemTokenFilter;

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
    fn test_czech_stem() {
        let filter = CzechStemTokenFilter::new();
        let mut token = make_token("knihami");
        filter.filter(&mut token);
        // Lucene CzechStemmer: -mi stripped, then final -a
        assert_eq!(token.term.as_ref(), "knih");
    }

    #[test]
    fn test_norwegian_light() {
        let filter = NorwegianLightStemTokenFilter::new();
        let mut token = make_token("bøkene");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "bøk");
    }
}
