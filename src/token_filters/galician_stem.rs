use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Galician stemmer based on the Lucene GalicianStemmer.
///
/// Removes common Galician suffixes (plural, gender, derivational).
#[derive(Clone, Debug, Default)]
pub struct GalicianStemTokenFilter;

impl GalicianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GalicianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_galician(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

/// Galician minimal stemmer — only handles plurals.
///
/// A lighter variant that only removes plural suffixes.
#[derive(Clone, Debug, Default)]
pub struct GalicianMinimalStemTokenFilter;

impl GalicianMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GalicianMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_galician_plural(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_galician_plural(word: &str) -> String {
    let lower = word.to_lowercase();
    let len = lower.len();

    if len < 4 {
        return lower;
    }

    // -ns -> -n
    if lower.ends_with("ns") && len > 3 {
        return lower[..len - 1].to_string();
    }

    // -ões -> -ón
    if lower.ends_with("ões") && len > 4 {
        let mut result = lower[..len - "ões".len()].to_string();
        result.push_str("ón");
        return result;
    }

    // -ais -> -al
    if lower.ends_with("ais") && len > 4 {
        let mut result = lower[..len - 2].to_string();
        result.push('l');
        return result;
    }

    // -éis -> -el
    if lower.ends_with("éis") && len > 4 {
        let mut result = lower[..len - "éis".len()].to_string();
        result.push_str("el");
        return result;
    }

    // -eis -> -el
    if lower.ends_with("eis") && len > 4 {
        let mut result = lower[..len - 3].to_string();
        result.push_str("el");
        return result;
    }

    // -ís -> -il
    if lower.ends_with("ís") && len > 3 {
        let mut result = lower[..len - "ís".len()].to_string();
        result.push_str("il");
        return result;
    }

    // -is -> -il (unstressed)
    if lower.ends_with("is") && len > 3 {
        let mut result = lower[..len - 2].to_string();
        result.push('l');
        return result;
    }

    // -les -> -l
    if lower.ends_with("les") && len > 4 {
        return lower[..len - 2].to_string();
    }

    // -res -> -r
    if lower.ends_with("res") && len > 4 {
        return lower[..len - 2].to_string();
    }

    // -s (generic)
    if lower.ends_with('s') && len > 3 {
        return lower[..len - 1].to_string();
    }

    lower
}

fn stem_galician(word: &str) -> String {
    let lower = word.to_lowercase();
    let len = lower.len();

    if len < 4 {
        return lower;
    }

    // First reduce plural
    let word = stem_galician_plural(&lower);
    let len = word.len();

    if len < 3 {
        return word;
    }

    // Remove augmentative/diminutive suffixes
    let augmentative = ["iño", "iña", "ote", "ota", "azo", "aza"];
    for suffix in &augmentative {
        if word.ends_with(suffix) && len > suffix.len() + 2 {
            return word[..len - suffix.len()].to_string();
        }
    }

    // Remove adverb
    if word.ends_with("mente") && len > 7 {
        return word[..len - 5].to_string();
    }

    // Remove common derivational suffixes
    let suffixes = [
        "ización", "amente", "idade", "ación", "ición", "mente", "ismo", "ista", "anza", "enza",
        "eiro", "eira", "ería", "ble", "dor", "dora", "oso", "osa",
    ];

    for suffix in &suffixes {
        if word.ends_with(suffix) && len > suffix.len() + 2 {
            return word[..len - suffix.len()].to_string();
        }
    }

    // Remove gender marker
    if word.ends_with('a') && len > 3 {
        let mut result = word[..len - 1].to_string();
        result.push('o');
        return result;
    }

    word
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_galician_plural() {
        let filter = GalicianMinimalStemTokenFilter::new();

        let mut token = Token::new("libros", 0, 6, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "libro");
    }

    #[test]
    fn test_galician_full() {
        let filter = GalicianStemTokenFilter::new();

        let mut token = Token::new("rapidamente", 0, 11, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "rapida");
    }
}
