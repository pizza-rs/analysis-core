use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Hyphenation-based compound word decomposition filter.
///
/// Unlike the dictionary-based decompounder which uses a flat dictionary,
/// this filter uses hyphenation patterns to find likely word boundaries
/// in compound words, then validates the parts against a dictionary.
///
/// This approach works well for languages with productive compounding
/// like German, Dutch, Norwegian, and Swedish.
#[derive(Clone, Debug)]
pub struct HyphenationDecompounderTokenFilter {
    dictionary: HashSet<String>,
    min_word_size: usize,
    min_subword_size: usize,
    max_subword_size: usize,
    /// Positions where hyphenation is allowed (character indices).
    /// In a full implementation, these would come from a hyphenation patterns file.
    /// For simplicity, we use a heuristic: try splitting at each position.
    only_longest_match: bool,
}

impl HyphenationDecompounderTokenFilter {
    pub fn new(dictionary: Vec<String>) -> Self {
        Self {
            dictionary: dictionary.into_iter().map(|w| w.to_lowercase()).collect(),
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

    /// Try to decompose the word by finding valid splits where both parts
    /// are in the dictionary.
    fn decompose(&self, word: &str) -> Vec<String> {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let len = chars.len();

        if len < self.min_word_size {
            return Vec::new();
        }
        // Need room for at least two sub-words of `min_subword_size` chars.
        // Without this guard, `len - self.min_subword_size + 1` below
        // would underflow when `min_subword_size > len`, panicking on
        // misconfigured filters (e.g. min_subword_size=5 on a 4-char word).
        if len < self.min_subword_size.saturating_mul(2) {
            return Vec::new();
        }

        let mut best_parts: Vec<String> = Vec::new();

        // Try every possible split point
        for split in self.min_subword_size..(len - self.min_subword_size + 1) {
            let left: String = chars[..split].iter().collect();
            let right: String = chars[split..].iter().collect();

            let left_valid = left.len() >= self.min_subword_size
                && left.len() <= self.max_subword_size
                && self.dictionary.contains(&left);

            let right_valid = right.len() >= self.min_subword_size
                && right.len() <= self.max_subword_size
                && self.dictionary.contains(&right);

            if left_valid && right_valid {
                if self.only_longest_match {
                    if best_parts.is_empty()
                        || left.len() + right.len()
                            > best_parts.iter().map(|p| p.len()).sum::<usize>()
                    {
                        best_parts = vec![left, right];
                    }
                } else {
                    // Return first valid decomposition
                    return vec![left, right];
                }
            }
        }

        best_parts
    }
}

impl TokenFilter for HyphenationDecompounderTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let parts = self.decompose(text);

        if parts.is_empty() {
            return (false, None);
        }

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
    fn test_decompose_german() {
        let filter =
            HyphenationDecompounderTokenFilter::new(vec!["butter".to_string(), "brot".to_string()]);
        let mut token = make_token("butterbrot");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        let extra = extra.unwrap();
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"butter"));
        assert!(terms.contains(&"brot"));
    }

    #[test]
    fn test_no_valid_split() {
        let filter = HyphenationDecompounderTokenFilter::new(vec!["butter".to_string()]);
        let mut token = make_token("butterfly");
        let (_, extra) = filter.filter(&mut token);
        // "fly" is not in dictionary
        assert!(extra.is_none());
    }

    #[test]
    fn test_too_short() {
        let filter =
            HyphenationDecompounderTokenFilter::new(vec!["ab".to_string(), "cd".to_string()]);
        let mut token = make_token("abcd");
        let (_, extra) = filter.filter(&mut token);
        assert!(extra.is_none()); // word too short (< 5)
    }
}
