use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes stop words from the token stream.
///
/// Supports configurable word lists.
#[derive(Clone, Debug)]
pub struct StopTokenFilter {
    stop_words: HashSet<String>,
    ignore_case: bool,
}

impl StopTokenFilter {
    /// Create with a custom stop word list.
    pub fn new(words: &[&str]) -> Self {
        Self {
            stop_words: words.iter().map(|&s| s.to_string()).collect(),
            ignore_case: false,
        }
    }

    /// Create with default English stop words.
    pub fn english() -> Self {
        Self::new(&ENGLISH_STOP_WORDS)
    }

    /// Create a stop filter for a specific language using built-in stop word lists.
    ///
    /// Supports 40+ languages. Returns `None` if the language is not recognized.
    ///
    /// # Example
    /// ```rust
    /// use pizza_analysis_core::StopTokenFilter;
    ///
    /// let french_stop = StopTokenFilter::for_language("french").unwrap();
    /// let german_stop = StopTokenFilter::for_language("german").unwrap();
    /// ```
    pub fn for_language(language: &str) -> Option<Self> {
        super::stopwords::get_stop_words(language).map(|words| Self::new(words))
    }

    /// Get the list of supported languages for `for_language()`.
    pub fn supported_languages() -> &'static [&'static str] {
        &[
            "afrikaans",
            "amharic",
            "arabic",
            "armenian",
            "azerbaijani",
            "basque",
            "bengali",
            "brazilian",
            "bulgarian",
            "catalan",
            "chinese",
            "cjk",
            "croatian",
            "czech",
            "danish",
            "dutch",
            "english",
            "estonian",
            "finnish",
            "french",
            "galician",
            "georgian",
            "german",
            "greek",
            "hebrew",
            "hindi",
            "hungarian",
            "indonesian",
            "irish",
            "italian",
            "japanese",
            "korean",
            "latvian",
            "lithuanian",
            "malay",
            "marathi",
            "mongolian",
            "nepali",
            "norwegian",
            "persian",
            "polish",
            "portuguese",
            "romanian",
            "russian",
            "serbian",
            "slovak",
            "slovenian",
            "sorani",
            "spanish",
            "swahili",
            "swedish",
            "tagalog",
            "tamil",
            "thai",
            "turkish",
            "ukrainian",
            "urdu",
            "vietnamese",
        ]
    }

    pub fn with_ignore_case(mut self, ignore_case: bool) -> Self {
        if ignore_case {
            // Convert all stop words to lowercase
            self.stop_words = self
                .stop_words
                .into_iter()
                .map(|w| w.to_lowercase())
                .collect();
        }
        self.ignore_case = ignore_case;
        self
    }

    fn is_stop_word(&self, term: &str) -> bool {
        if self.ignore_case {
            self.stop_words.contains(&term.to_lowercase())
        } else {
            self.stop_words.contains(term)
        }
    }
}

impl Default for StopTokenFilter {
    fn default() -> Self {
        Self::english()
    }
}

impl TokenFilter for StopTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if self.is_stop_word(&token.term) {
            return (true, None);
        }
        (false, None)
    }
}

/// Default English stop words.
pub const ENGLISH_STOP_WORDS: [&str; 33] = [
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into", "is", "it",
    "no", "not", "of", "on", "or", "such", "that", "the", "their", "then", "there", "these",
    "they", "this", "to", "was", "will", "with",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_filter() {
        let f = StopTokenFilter::english();
        let mut token_the = Token::new("the", 0, 3, 0);
        let mut token_hello = Token::new("hello", 4, 9, 1);

        assert_eq!(f.filter(&mut token_the).0, true);
        assert_eq!(f.filter(&mut token_hello).0, false);
    }

    #[test]
    fn test_stop_filter_ignore_case() {
        let f = StopTokenFilter::english().with_ignore_case(true);
        let mut token = Token::new("The", 0, 3, 0);
        assert_eq!(f.filter(&mut token).0, true);
    }

    #[test]
    fn test_custom_stop_words() {
        let f = StopTokenFilter::new(&["foo", "bar"]);
        let mut t1 = Token::new("foo", 0, 3, 0);
        let mut t2 = Token::new("baz", 4, 7, 1);
        assert_eq!(f.filter(&mut t1).0, true);
        assert_eq!(f.filter(&mut t2).0, false);
    }
}
