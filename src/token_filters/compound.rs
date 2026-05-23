use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Dictionary-based compound word decomposition filter.
///
/// Splits compound words (common in Germanic languages like German, Dutch,
/// Norwegian, Swedish) into their constituent parts using a dictionary.
///
/// For example, with a dictionary containing ["donner", "wetter"]:
/// "donnerwetter" → ["donner", "wetter"]
///
/// Configuration:
/// - `min_word_size`: minimum total word length to attempt decomposition (default: 5)
/// - `min_subword_size`: minimum length of a component part (default: 2)
/// - `max_subword_size`: maximum length of a component part (default: 15)
/// - `only_longest_match`: if true, only emit the longest matching decomposition
#[derive(Clone, Debug)]
pub struct DictionaryDecompounderTokenFilter {
    dictionary: HashSet<String>,
    min_word_size: usize,
    min_subword_size: usize,
    max_subword_size: usize,
    only_longest_match: bool,
}

impl DictionaryDecompounderTokenFilter {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            dictionary: words.into_iter().collect(),
            min_word_size: 5,
            min_subword_size: 2,
            max_subword_size: 15,
            only_longest_match: false,
        }
    }

    pub fn with_min_word_size(mut self, size: usize) -> Self {
        self.min_word_size = size;
        self
    }

    pub fn with_min_subword_size(mut self, size: usize) -> Self {
        self.min_subword_size = size;
        self
    }

    pub fn with_max_subword_size(mut self, size: usize) -> Self {
        self.max_subword_size = size;
        self
    }

    pub fn with_only_longest_match(mut self, only_longest: bool) -> Self {
        self.only_longest_match = only_longest;
        self
    }

    /// Attempt to decompose a word into parts found in the dictionary.
    fn decompose(&self, word: &str) -> Vec<String> {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let len = chars.len();

        if len < self.min_word_size {
            return Vec::new();
        }

        let mut parts: Vec<String> = Vec::new();

        // Try all possible decompositions
        for start in 0..len {
            let max_end = (start + self.max_subword_size).min(len);
            for end in (start + self.min_subword_size)..=max_end {
                let subword: String = chars[start..end].iter().collect();
                if self.dictionary.contains(&subword) {
                    if !parts.contains(&subword) {
                        parts.push(subword);
                    }
                }
            }
        }

        if self.only_longest_match && parts.len() > 1 {
            // Keep only the longest match at each position
            parts.sort_by(|a, b| b.len().cmp(&a.len()));
            parts.truncate(1);
        }

        parts
    }
}

impl TokenFilter for DictionaryDecompounderTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts = self.decompose(text);

        if parts.is_empty() {
            return (false, None);
        }

        // Emit decomposed parts as additional tokens at the same position
        let extra: Vec<Token<'a>> = parts
            .into_iter()
            .map(|part| Token {
                term: Cow::Owned(part),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            })
            .collect();

        (false, Some(extra))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_decompose() {
        let filter = DictionaryDecompounderTokenFilter::new(vec![
            "donner".to_string(),
            "wetter".to_string(),
        ]);
        let mut token = make_token("donnerwetter");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "donnerwetter"); // original preserved
        let extra = extra.unwrap();
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"donner"));
        assert!(terms.contains(&"wetter"));
    }

    #[test]
    fn test_too_short() {
        let filter = DictionaryDecompounderTokenFilter::new(vec!["ab".to_string()]);
        let mut token = make_token("abc");
        let (_, extra) = filter.filter(&mut token);
        assert!(extra.is_none()); // word too short (< min_word_size=5)
    }

    #[test]
    fn test_no_match() {
        let filter = DictionaryDecompounderTokenFilter::new(vec!["xyz".to_string()]);
        let mut token = make_token("something");
        let (_, extra) = filter.filter(&mut token);
        assert!(extra.is_none());
    }
}
