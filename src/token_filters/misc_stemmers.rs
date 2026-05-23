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

        let stemmed = stem_czech(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_czech(word: &str) -> String {
    let mut result = String::from(word);

    // Remove case suffixes (longest first)
    // 4-char
    if result.len() > 6 {
        let len = result.len();
        let suffix = &result[len - 4..];
        match suffix {
            "atům" | "ětem" | "ích" | "ách" => {
                result.truncate(len - 4);
                return result;
            }
            _ => {}
        }
    }

    // 3-char
    if result.len() > 5 {
        let len = result.len();
        let suffix = &result[len - 3..];
        match suffix {
            "ech" | "ich" | "ích" | "ého" | "ěmi" | "emi" | "ému" | "ách" | "ata" | "ovi"
            | "ové" => {
                result.truncate(len - 3);
                return result;
            }
            _ => {}
        }
    }

    // 2-char
    if result.len() > 4 {
        let len = result.len();
        let suffix = &result[len - 2..];
        match suffix {
            "em" | "es" | "ém" | "ím" | "ům" | "at" | "ám" | "os" | "us" | "ým" | "mi" | "ou"
            | "ej" | "ov" | "ín" | "ík" => {
                result.truncate(len - 2);
                return result;
            }
            _ => {}
        }
    }

    // 1-char
    if result.len() > 3 {
        let last = result.chars().last().unwrap();
        match last {
            'e' | 'i' | 'í' | 'ě' | 'u' | 'ů' | 'y' | 'a' | 'o' | 'á' | 'é' | 'ý' => {
                result.pop();
            }
            _ => {}
        }
    }

    // Palatalization
    let len = result.len();
    if len > 3 {
        if result.ends_with("čt") {
            result.truncate(len - 2);
            result.push_str("ck");
        } else if result.ends_with("št") {
            result.truncate(len - 2);
            result.push_str("sk");
        } else if result.ends_with("čn") || result.ends_with("čk") {
            result.truncate(len - 2);
            result.push_str("čn");
        }
    }

    result
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

/// Dutch stemmer (Kraaij-Pohlmann algorithm).
///
/// Removes common Dutch suffixes.
#[derive(Clone, Debug, Default)]
pub struct DutchStemTokenFilter;

impl DutchStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for DutchStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_dutch(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_dutch(word: &str) -> String {
    let mut result = String::from(word);

    // Remove common Dutch suffixes
    if result.ends_with("heid") && result.len() > 7 {
        result.truncate(result.len() - 4);
        return result;
    }

    if result.ends_with("ing") && result.len() > 6 {
        result.truncate(result.len() - 3);
        return result;
    }

    if result.ends_with("en") && result.len() > 4 {
        result.truncate(result.len() - 2);
        return result;
    }

    if result.ends_with('e') && result.len() > 3 {
        result.pop();
        return result;
    }

    if result.ends_with('s') && result.len() > 3 {
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
    fn test_czech_stem() {
        let filter = CzechStemTokenFilter::new();
        let mut token = make_token("knihami");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "kniha");
    }

    #[test]
    fn test_norwegian_light() {
        let filter = NorwegianLightStemTokenFilter::new();
        let mut token = make_token("bøkene");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "bøk");
    }

    #[test]
    fn test_dutch_stem() {
        let filter = DutchStemTokenFilter::new();
        let mut token = make_token("huizen");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "huiz");
    }
}
