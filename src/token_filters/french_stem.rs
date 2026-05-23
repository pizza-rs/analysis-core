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
        let len = text.len();

        if len < 6 {
            return (false, None);
        }

        let stemmed = stem_french_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }

        (false, None)
    }
}

fn stem_french_light(word: &str) -> String {
    let mut result = String::from(word);

    // Remove trailing 's' or 'x' (plural markers)
    if result.ends_with('s') || result.ends_with('x') {
        result.pop();
    }

    let len = result.len();
    if len < 5 {
        return result;
    }

    // Remove 'ment' (adverb suffix) - lentement → lente
    if result.ends_with("ment") && len > 7 {
        result.truncate(len - 4);
        return result;
    }

    // Remove various endings
    if result.ends_with("eux") {
        result.truncate(result.len() - 1); // eux → eu
        return result;
    }

    if result.ends_with("euse") || result.ends_with("ière") {
        result.truncate(result.len() - 4);
        return result;
    }

    if result.ends_with("ère") || result.ends_with("eur") {
        result.truncate(result.len() - 3);
        return result;
    }

    // Feminine endings
    if result.ends_with("ive") {
        result.truncate(result.len() - 3);
        result.push_str("if");
        return result;
    }

    if result.ends_with("aux") {
        result.truncate(result.len() - 3);
        result.push_str("al");
        return result;
    }

    if result.ends_with("ée") || result.ends_with("ie") {
        result.truncate(result.len() - 2);
        return result;
    }

    if result.ends_with('é') || result.ends_with('è') {
        result.pop();
        return result;
    }

    result
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
