use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Keeps only tokens found in the provided word set. All other tokens are removed.
/// This is the inverse of the stop word filter.
#[derive(Clone, Debug)]
pub struct KeepWordsTokenFilter {
    words: HashSet<String>,
    ignore_case: bool,
}

impl KeepWordsTokenFilter {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            words: words.into_iter().collect(),
            ignore_case: false,
        }
    }

    pub fn with_ignore_case(mut self, ignore_case: bool) -> Self {
        if ignore_case {
            self.words = self.words.iter().map(|w| w.to_lowercase()).collect();
        }
        self.ignore_case = ignore_case;
        self
    }
}

impl TokenFilter for KeepWordsTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let found = if self.ignore_case {
            let lower = term.to_lowercase();
            self.words.contains(lower.as_str())
        } else {
            self.words.contains(term)
        };
        // Remove if NOT in the keep list
        (!found, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::borrow::Cow;
    use alloc::string::ToString;

    fn make_token(term: &str) -> Token<'_> {
        Token {
            term: Cow::Borrowed(term),
            start_offset: 0,
            end_offset: term.len() as u32,
            position: 0,
        }
    }

    #[test]
    fn test_keep_words() {
        let filter = KeepWordsTokenFilter::new(vec!["cat".to_string(), "dog".to_string()]);
        let mut t1 = make_token("cat");
        let (remove, _) = filter.filter(&mut t1);
        assert!(!remove);

        let mut t2 = make_token("bird");
        let (remove, _) = filter.filter(&mut t2);
        assert!(remove);
    }

    #[test]
    fn test_keep_words_ignore_case() {
        let filter = KeepWordsTokenFilter::new(vec!["Cat".to_string()]).with_ignore_case(true);
        let mut t1 = make_token("CAT");
        let (remove, _) = filter.filter(&mut t1);
        assert!(!remove);

        let mut t2 = make_token("cat");
        let (remove, _) = filter.filter(&mut t2);
        assert!(!remove);
    }
}
