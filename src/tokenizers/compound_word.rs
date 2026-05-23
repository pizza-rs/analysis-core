use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Case-insensitive ASCII-fast equality between a (possibly mixed-case) candidate
/// and an already-lowercased dictionary entry. Falls back to full Unicode
/// lowercasing only when non-ASCII bytes are present, ensuring no byte-index
/// mismatch can occur in the caller.
fn eq_ignore_case_ascii_or_unicode(candidate: &str, lower_dict_entry: &str) -> bool {
    if candidate.is_ascii() && lower_dict_entry.is_ascii() {
        candidate.eq_ignore_ascii_case(lower_dict_entry)
    } else {
        // Compare codepoint-by-codepoint with lowercase folding.
        let mut a = candidate.chars().flat_map(|c| c.to_lowercase());
        let mut b = lower_dict_entry.chars();
        loop {
            match (a.next(), b.next()) {
                (None, None) => return true,
                (Some(x), Some(y)) if x == y => continue,
                _ => return false,
            }
        }
    }
}

/// Splits compound words using a dictionary-based approach.
///
/// Common in German, Dutch, Finnish etc. where words are glued together:
/// "Krankenhaus" → "Kranken" + "haus" (hospital)
///
/// Uses a minimum component length and greedy longest-match from either end.
#[derive(Clone, Debug)]
pub struct CompoundWordTokenizer {
    /// Known word parts / dictionary entries (lowercase)
    dictionary: Vec<String>,
    /// Minimum length of a sub-word to be considered
    min_subword_size: usize,
    /// Maximum length of a sub-word
    max_subword_size: usize,
    /// Minimum length of the whole word to attempt decomposition
    min_word_length: usize,
    /// Also emit the original compound token
    only_longest_match: bool,
}

impl CompoundWordTokenizer {
    pub fn new(dictionary: Vec<String>) -> Self {
        Self {
            dictionary,
            min_subword_size: 3,
            max_subword_size: 20,
            min_word_length: 5,
            only_longest_match: true,
        }
    }

    pub fn with_min_subword_size(mut self, size: usize) -> Self {
        self.min_subword_size = size;
        self
    }

    pub fn with_max_subword_size(mut self, size: usize) -> Self {
        self.max_subword_size = size;
        self
    }

    pub fn with_min_word_length(mut self, len: usize) -> Self {
        self.min_word_length = len;
        self
    }

    /// Returns byte-index pairs `(start, end)` into the **original** `word`
    /// (not a lowercased copy) so callers can safely slice `word` without
    /// risking UTF-8 boundary panics caused by length changes from
    /// `to_lowercase()` (e.g. Turkish `İ`, German `ß`).
    fn find_subwords(&self, word: &str) -> Vec<(usize, usize)> {
        let mut parts: Vec<(usize, usize)> = Vec::new();
        let len = word.len();

        if word.chars().count() < self.min_word_length {
            return parts;
        }

        // Walk char boundaries of the ORIGINAL word.
        let boundaries: Vec<usize> = word
            .char_indices()
            .map(|(i, _)| i)
            .chain(core::iter::once(len))
            .collect();

        let mut bi = 0usize;
        while bi + 1 < boundaries.len() {
            let i = boundaries[bi];
            let mut best_end = 0usize;
            let mut best_bi = bi;

            // Try every char-aligned end position from min..=max sub-word chars.
            let max_bi = (bi + self.max_subword_size).min(boundaries.len() - 1);
            for end_bi in (bi + self.min_subword_size)..=max_bi {
                let j = boundaries[end_bi];
                let sub = &word[i..j];
                if self
                    .dictionary
                    .iter()
                    .any(|d| eq_ignore_case_ascii_or_unicode(sub, d))
                {
                    best_end = j;
                    best_bi = end_bi;
                    if !self.only_longest_match {
                        parts.push((i, j));
                    }
                }
            }

            if self.only_longest_match && best_end > 0 {
                parts.push((i, best_end));
                bi = best_bi;
            } else if best_end > 0 {
                bi = best_bi;
            } else {
                bi += 1;
            }
        }

        parts
    }
}

impl Default for CompoundWordTokenizer {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Tokenizer for CompoundWordTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        // Split on whitespace first
        for word in text.split_whitespace() {
            let word_start = word.as_ptr() as usize - text.as_ptr() as usize;

            // Always emit original
            tokens.push(Token {
                term: Cow::Borrowed(word),
                start_offset: word_start as u32,
                end_offset: (word_start + word.len()) as u32,
                position,
            });

            // Try decomposition
            let subwords = self.find_subwords(word);
            if subwords.len() >= 2 {
                for (s, e) in &subwords {
                    let part = &word[*s..*e];
                    tokens.push(Token {
                        term: Cow::Borrowed(part),
                        start_offset: (word_start + s) as u32,
                        end_offset: (word_start + e) as u32,
                        position,
                    });
                }
            }

            position += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_decomposition() {
        let dict = vec![
            "super".into(),
            "market".into(),
            "fire".into(),
            "truck".into(),
        ];
        let tok = CompoundWordTokenizer::new(dict);
        let tokens = tok.tokenize("supermarket");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"supermarket")); // original
        assert!(terms.contains(&"super"));
        assert!(terms.contains(&"market"));
    }

    #[test]
    fn test_no_decomposition_short_word() {
        let dict = vec!["cat".into()];
        let tok = CompoundWordTokenizer::new(dict).with_min_word_length(5);
        let tokens = tok.tokenize("cat");
        assert_eq!(tokens.len(), 1); // only original, too short
    }

    #[test]
    fn test_multiple_words() {
        let dict = vec!["fire".into(), "truck".into(), "house".into()];
        let tok = CompoundWordTokenizer::new(dict);
        let tokens = tok.tokenize("firetruck firehouse");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"fire"));
        assert!(terms.contains(&"truck"));
        assert!(terms.contains(&"house"));
    }
}
