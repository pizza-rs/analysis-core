use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashSet;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes leading elisions (e.g. l', d', qu') from tokens.
///
/// Common in French, Italian, Catalan etc.
#[derive(Clone, Debug)]
pub struct ElisionTokenFilter {
    articles: HashSet<String>,
}

impl ElisionTokenFilter {
    /// Create with a list of articles to strip.
    pub fn new(articles: &[&str]) -> Self {
        Self {
            articles: articles.iter().map(|a| a.to_lowercase()).collect(),
        }
    }

    /// French default articles.
    pub fn french() -> Self {
        Self::new(&[
            "l", "m", "t", "qu", "n", "s", "j", "d", "c", "jusqu", "quoiqu", "lorsqu", "puisqu",
        ])
    }

    /// Italian default articles.
    pub fn italian() -> Self {
        Self::new(&[
            "c", "l", "all", "dall", "dell", "nell", "sull", "coll", "pell", "gl", "agl", "dagl",
            "degl", "negl", "sugl", "un", "m", "t", "s", "v", "d",
        ])
    }

    /// Catalan default articles.
    pub fn catalan() -> Self {
        Self::new(&["d", "l", "m", "n", "s", "t"])
    }
}

impl TokenFilter for ElisionTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        // Look for apostrophe or right single quotation mark.
        if let Some(apos_pos) = term.find(|c| c == '\'' || c == '\u{2019}') {
            // The matched apostrophe may be U+2019 (3 UTF-8 bytes); compute
            // its actual byte length instead of assuming 1. The previous
            // `apos_pos + 1` panicked with "byte index N is not a char
            // boundary" on real-world smart-quoted input like `l\u{2019}homme`.
            let apos_len = term[apos_pos..]
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            let prefix = &term[..apos_pos];
            if self.articles.contains(&prefix.to_lowercase()) {
                let remainder = &term[apos_pos + apos_len..];
                if !remainder.is_empty() {
                    token.term = Cow::Owned(remainder.to_string());
                    token.start_offset += (apos_pos as u32) + (apos_len as u32);
                }
            }
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_french_elision() {
        let f = ElisionTokenFilter::french();
        let mut token = Token::new("l'avion", 0, 7, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "avion");
    }

    #[test]
    fn test_no_elision() {
        let f = ElisionTokenFilter::french();
        let mut token = Token::new("hello", 0, 5, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "hello");
    }

    #[test]
    fn test_non_article() {
        let f = ElisionTokenFilter::french();
        let mut token = Token::new("foo'bar", 0, 7, 0);
        f.filter(&mut token);
        // "foo" is not in articles, so no change
        assert_eq!(token.term, "foo'bar");
    }

    #[test]
    fn test_case_insensitive() {
        let f = ElisionTokenFilter::french();
        let mut token = Token::new("L'homme", 0, 7, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "homme");
    }

    #[test]
    fn test_curly_apostrophe_no_panic() {
        // U+2019 is 3 bytes in UTF-8. The previous implementation panicked
        // here with `byte index 2 is not a char boundary`.
        let f = ElisionTokenFilter::french();
        let term = "l\u{2019}homme";
        let mut token = Token::new(term, 0, term.len() as u32, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "homme");
    }
}
