use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Bulgarian light stemmer based on the Lucene BulgarianStemmer.
///
/// Removes common Bulgarian suffixes to normalize inflected forms.
/// Handles noun, adjective, and verb endings.
#[derive(Clone, Debug, Default)]
pub struct BulgarianStemTokenFilter;

impl BulgarianStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for BulgarianStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() < 4 {
            return (false, None);
        }

        let stemmed = stem_bulgarian(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_bulgarian(word: &str) -> String {
    let mut result = word.to_lowercase();
    let len = result.chars().count();

    if len < 4 {
        return result;
    }

    // Remove definite article suffixes (longest first)
    if len > 5 {
        let suffixes_4 = ["ният", "ната", "ното", "ните"];
        for suffix in &suffixes_4 {
            if result.ends_with(suffix) {
                let byte_len = result.len() - suffix.len();
                result.truncate(byte_len);
                return result;
            }
        }
    }

    if len > 4 {
        let suffixes_3 = ["ият", "ето", "ата"];
        for suffix in &suffixes_3 {
            if result.ends_with(suffix) {
                let byte_len = result.len() - suffix.len();
                result.truncate(byte_len);
                return result;
            }
        }
    }

    if len > 3 {
        let suffixes_2 = ["ът", "та", "то", "те"];
        for suffix in &suffixes_2 {
            if result.ends_with(suffix) {
                let byte_len = result.len() - suffix.len();
                result.truncate(byte_len);
                return result;
            }
        }
    }

    // Remove plural suffixes
    if len > 5 {
        let plural_suffixes = ["ища", "ове", "ове", "ета"];
        for suffix in &plural_suffixes {
            if result.ends_with(suffix) {
                let byte_len = result.len() - suffix.len();
                result.truncate(byte_len);
                return result;
            }
        }
    }

    if len > 4 {
        if result.ends_with("ци") || result.ends_with("та") || result.ends_with("ни") {
            let byte_len = result.len() - "ци".len();
            result.truncate(byte_len);
            return result;
        }
        if result.ends_with("и")
            || result.ends_with("а")
            || result.ends_with("е")
            || result.ends_with("о")
        {
            let byte_len = result.len() - "и".len();
            result.truncate(byte_len);
            return result;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_bulgarian_stem() {
        let filter = BulgarianStemTokenFilter::new();

        // Test definite article removal
        let mut token = Token::new("книгата", 0, 14, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "книг");

        // Short words unchanged
        let mut token = Token::new("аз", 0, 4, 0);
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "аз");
    }
}
