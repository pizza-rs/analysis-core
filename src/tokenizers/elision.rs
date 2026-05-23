use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Elision tokenizer that strips leading articles/particles from tokens.
///
/// Common in French (`l'homme` → `homme`), Italian (`l'uomo` → `uomo`),
/// and other Romance languages where articles are contracted with apostrophes.
///
/// Can also handle other configurable prefixes.
#[derive(Clone, Debug)]
pub struct ElisionTokenizer {
    /// Articles/particles to strip (case-insensitive)
    articles: Vec<String>,
}

impl ElisionTokenizer {
    pub fn new(articles: Vec<String>) -> Self {
        Self {
            articles: articles.into_iter().map(|a| a.to_lowercase()).collect(),
        }
    }

    /// Create with French defaults
    pub fn french() -> Self {
        Self::new(vec![
            "l".into(),
            "m".into(),
            "t".into(),
            "qu".into(),
            "n".into(),
            "s".into(),
            "j".into(),
            "d".into(),
            "c".into(),
            "jusqu".into(),
            "quoiqu".into(),
            "lorsqu".into(),
            "puisqu".into(),
        ])
    }

    /// Create with Italian defaults
    pub fn italian() -> Self {
        Self::new(vec![
            "l".into(),
            "dell".into(),
            "nell".into(),
            "all".into(),
            "d".into(),
            "un".into(),
            "sull".into(),
            "quest".into(),
            "quell".into(),
        ])
    }

    fn strip_elision<'a>(&self, word: &'a str) -> &'a str {
        // Look for apostrophe (ASCII or Unicode right-single-quote).
        let apos = word
            .find('\'')
            .map(|p| (p, 1usize))
            .or_else(|| word.find('\u{2019}').map(|p| (p, '\u{2019}'.len_utf8())));

        if let Some((apos_pos, apos_len)) = apos {
            let prefix_lower = word[..apos_pos].to_lowercase();
            if self.articles.iter().any(|a| a == &prefix_lower) {
                return &word[apos_pos + apos_len..];
            }
        }
        word
    }
}

impl Default for ElisionTokenizer {
    fn default() -> Self {
        Self::french()
    }
}

impl Tokenizer for ElisionTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        for word in text.split_whitespace() {
            let stripped = self.strip_elision(word);

            if stripped.is_empty() {
                continue;
            }

            let token_offset = stripped.as_ptr() as usize - text.as_ptr() as usize;
            tokens.push(Token {
                term: Cow::Borrowed(stripped),
                start_offset: token_offset as u32,
                end_offset: (token_offset + stripped.len()) as u32,
                position,
            });
            position += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_french_elision() {
        let tok = ElisionTokenizer::french();
        let tokens = tok.tokenize("l'homme l'avion");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["homme", "avion"]);
    }

    #[test]
    fn test_no_elision() {
        let tok = ElisionTokenizer::french();
        let tokens = tok.tokenize("bonjour monde");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["bonjour", "monde"]);
    }

    #[test]
    fn test_italian() {
        let tok = ElisionTokenizer::italian();
        let tokens = tok.tokenize("l'uomo dell'arte");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["uomo", "arte"]);
    }

    #[test]
    fn test_case_insensitive() {
        let tok = ElisionTokenizer::french();
        let tokens = tok.tokenize("L'Homme");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["Homme"]);
    }

    #[test]
    fn test_unknown_prefix() {
        let tok = ElisionTokenizer::french();
        let tokens = tok.tokenize("xyz'hello");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["xyz'hello"]); // not stripped
    }
}
