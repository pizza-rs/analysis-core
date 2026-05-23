use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Removes everything after the first apostrophe in each token.
///
/// Useful for Turkish and other languages where apostrophes separate
/// suffixes from proper nouns.
#[derive(Clone, Debug)]
pub struct ApostropheTokenFilter;

impl ApostropheTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ApostropheTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for ApostropheTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        // Match both ASCII apostrophe (U+0027) and the typographic right
        // single quotation mark (U+2019 `’`), which is what most modern
        // text (Word, iOS, macOS, smart-quoting web fonts) actually emits.
        let pos = token
            .term
            .find('\'')
            .or_else(|| token.term.find('\u{2019}'));
        if let Some(pos) = pos {
            if pos == 0 {
                return (true, None); // Token is just an apostrophe
            }
            let truncated = &token.term[..pos];
            token.term = Cow::Owned(truncated.to_string());
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apostrophe_filter() {
        let f = ApostropheTokenFilter::new();
        let mut token = Token::new("Istanbul'un", 0, 11, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "Istanbul");
    }

    #[test]
    fn test_apostrophe_no_change() {
        let f = ApostropheTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_apostrophe_curly_unicode() {
        // U+2019 (’) is what iOS / Word / smart-quoting fonts emit.
        let f = ApostropheTokenFilter::new();
        let mut token = Token::new("Istanbul\u{2019}un", 0, 13, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "Istanbul");
    }
}
