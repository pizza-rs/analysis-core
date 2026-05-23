use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Latvian stemmer based on the Lucene LatvianStemmer.
///
/// Removes Latvian noun/adjective/verb endings to produce stems.
#[derive(Clone, Debug, Default)]
pub struct LatvianStemTokenFilter;

impl LatvianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for LatvianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_latvian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_latvian(word: &str) -> String {
    let lower = word.to_lowercase();
    let len = lower.len();

    if len < 4 {
        return lower;
    }

    // Masculine noun declension 1-6
    // Try 3-char endings first
    if len > 5 {
        let ending: String = lower
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        match ending.as_str() {
            "iem" | "ām" => {
                let byte_len = lower.len() - ending.len();
                return lower[..byte_len].to_string();
            }
            _ => {}
        }
    }

    // 2-char noun/adjective endings
    if len > 4 {
        let endings_to_strip = [
            "as", "es", "is", "os", "us", // nominative singular
            "am", "em", "im", "um", // dative singular
            "ai", "ei", // dative feminine
            "ās", "ēs", "īs", "ūs", // genitive/nominative plural
            "āi", "os", "us", // various cases
            "ām", "ēm", "īm", // dative plural
            "ās", "ēs", "īs", // various cases
            "aj", "ej", "ij", "uj", // locative
        ];

        for ending in &endings_to_strip {
            if lower.ends_with(ending) {
                let byte_len = lower.len() - ending.len();
                if byte_len >= 3 {
                    return lower[..byte_len].to_string();
                }
            }
        }
    }

    // Single char endings
    if len > 3 {
        let last = lower.chars().last().unwrap();
        match last {
            'a' | 'e' | 'i' | 'o' | 'u' | 's' => {
                let byte_len = lower.len() - last.len_utf8();
                if byte_len >= 3 {
                    return lower[..byte_len].to_string();
                }
            }
            _ => {}
        }
    }

    lower
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_latvian_stem() {
        let filter = LatvianStemTokenFilter::new();

        // Test noun ending removal
        let mut token = Token::new("grāmatas", 0, 12, 0);
        filter.filter(&mut token);
        assert!(token.term.len() < "grāmatas".len());

        // Short word unchanged
        let mut token = Token::new("un", 0, 2, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "un");
    }
}
