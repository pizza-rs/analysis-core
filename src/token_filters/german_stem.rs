use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// German light stemmer.
///
/// Removes common German suffixes (plural, case, derivational).
/// More aggressive than minimal, less than full Snowball.
#[derive(Clone, Debug, Default)]
pub struct GermanLightStemTokenFilter;

impl GermanLightStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GermanLightStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 5 {
            return (false, None);
        }

        let stemmed = stem_german_light(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_german_light(word: &str) -> String {
    let mut result = String::from(word);

    // Step 1: Remove plural suffixes
    if result.ends_with("ern") || result.ends_with("eln") {
        result.truncate(result.len() - 2); // keep the 'e'
    } else if result.ends_with("en") || result.ends_with("er") || result.ends_with("es") {
        result.truncate(result.len() - 2);
    } else if result.ends_with('e') || result.ends_with('s') || result.ends_with('n') {
        result.pop();
    }

    // Step 2: Remove umlaut (ä→a, ö→o, ü→u)
    let chars: Vec<char> = result.chars().collect();
    let mut deumlauted = String::with_capacity(result.len());
    let mut changed = false;
    for c in &chars {
        match c {
            'ä' => {
                deumlauted.push('a');
                changed = true;
            }
            'ö' => {
                deumlauted.push('o');
                changed = true;
            }
            'ü' => {
                deumlauted.push('u');
                changed = true;
            }
            _ => deumlauted.push(*c),
        }
    }
    if changed {
        result = deumlauted;
    }

    result
}

/// German minimal stemmer.
///
/// Only removes the most basic plural markers.
#[derive(Clone, Debug, Default)]
pub struct GermanMinimalStemTokenFilter;

impl GermanMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GermanMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 5 {
            return (false, None);
        }

        let stemmed = stem_german_minimal(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_german_minimal(word: &str) -> String {
    let mut result = String::from(word);

    if result.ends_with("ern") {
        result.truncate(result.len() - 3);
    } else if result.ends_with("en") || result.ends_with("er") || result.ends_with("es") {
        result.truncate(result.len() - 2);
    } else if result.ends_with('e') || result.ends_with('s') {
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
    fn test_german_light_plural() {
        let filter = GermanLightStemTokenFilter::new();
        let mut token = make_token("bücher");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "buch");
    }

    #[test]
    fn test_german_minimal() {
        let filter = GermanMinimalStemTokenFilter::new();
        let mut token = make_token("häuser");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "häus");
    }

    #[test]
    fn test_german_light_en() {
        let filter = GermanLightStemTokenFilter::new();
        let mut token = make_token("katzen");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "katz");
    }
}
