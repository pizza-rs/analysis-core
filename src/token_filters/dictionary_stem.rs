use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Dictionary-based stemmer token filter.
///
/// Uses a simple dictionary mapping from inflected forms to their stems.
/// This is a simplified alternative to a full Hunspell implementation,
/// suitable for cases where you have a pre-computed stem dictionary.
///
/// The dictionary maps surface forms (e.g., "running") to stems (e.g., "run").
/// If a word is not found in the dictionary, it passes through unchanged.
///
/// This is useful for:
/// - Languages with irregular morphology
/// - Domain-specific vocabularies
/// - Overriding algorithmic stemmers for specific words
#[derive(Clone, Debug)]
pub struct DictionaryStemTokenFilter {
    dictionary: HashMap<String, String>,
    case_insensitive: bool,
}

impl DictionaryStemTokenFilter {
    /// Create a new dictionary stemmer from a list of (word, stem) pairs.
    pub fn new(entries: Vec<(String, String)>) -> Self {
        Self {
            dictionary: entries.into_iter().collect(),
            case_insensitive: true,
        }
    }

    /// Create from a tab-separated format: "word\tstem" per line.
    pub fn from_tab_separated(content: &str) -> Self {
        let entries: Vec<(String, String)> = content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_owned(), parts[1].to_owned()))
                } else {
                    None
                }
            })
            .collect();
        Self::new(entries)
    }

    /// Create from an arrow-separated format: "word => stem" per line.
    /// Compatible with Elasticsearch's stemmer_override format.
    pub fn from_arrow_separated(content: &str) -> Self {
        let entries: Vec<(String, String)> = content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let parts: Vec<&str> = line.splitn(2, "=>").collect();
                if parts.len() == 2 {
                    Some((parts[0].trim().to_owned(), parts[1].trim().to_owned()))
                } else {
                    None
                }
            })
            .collect();
        Self::new(entries)
    }

    pub fn with_case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    fn lookup(&self, word: &str) -> Option<&String> {
        if self.case_insensitive {
            let lower = word.to_lowercase();
            self.dictionary.get(&lower)
        } else {
            self.dictionary.get(word)
        }
    }
}

impl TokenFilter for DictionaryStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let word = token.term.as_ref();
        if let Some(stem) = self.lookup(word) {
            token.term = Cow::Owned(stem.clone());
        }
        (false, None)
    }
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
    fn test_basic_lookup() {
        let filter = DictionaryStemTokenFilter::new(vec![
            ("running".to_owned(), "run".to_owned()),
            ("cats".to_owned(), "cat".to_owned()),
        ]);

        let mut token = make_token("running");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "run");
    }

    #[test]
    fn test_not_found() {
        let filter = DictionaryStemTokenFilter::new(vec![("running".to_owned(), "run".to_owned())]);

        let mut token = make_token("walking");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "walking"); // unchanged
    }

    #[test]
    fn test_case_insensitive() {
        let filter = DictionaryStemTokenFilter::new(vec![("running".to_owned(), "run".to_owned())]);

        let mut token = make_token("Running");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "run");
    }

    #[test]
    fn test_tab_separated() {
        let content = "running\trun\ncats\tcat\n# comment\n\nwalking\twalk";
        let filter = DictionaryStemTokenFilter::from_tab_separated(content);

        let mut t1 = make_token("running");
        let mut t2 = make_token("cats");
        let mut t3 = make_token("walking");
        filter.filter(&mut t1);
        filter.filter(&mut t2);
        filter.filter(&mut t3);
        assert_eq!(t1.term.as_ref(), "run");
        assert_eq!(t2.term.as_ref(), "cat");
        assert_eq!(t3.term.as_ref(), "walk");
    }

    #[test]
    fn test_arrow_separated() {
        let content = "running => run\ncats => cat";
        let filter = DictionaryStemTokenFilter::from_arrow_separated(content);

        let mut token = make_token("running");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "run");
    }
}
