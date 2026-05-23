use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Generates n-grams of specified sizes from each token.
///
/// For example, with min_gram=2, max_gram=3, the token "pizza" produces:
/// `["pi", "iz", "zz", "za", "piz", "izz", "zza"]`
#[derive(Clone, Debug)]
pub struct NgramTokenFilter {
    min_gram: usize,
    max_gram: usize,
    preserve_original: bool,
}

impl NgramTokenFilter {
    pub fn new(min_gram: usize, max_gram: usize) -> Self {
        Self {
            min_gram: min_gram.max(1),
            max_gram: max_gram.max(min_gram),
            preserve_original: false,
        }
    }

    pub fn with_preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }
}

impl Default for NgramTokenFilter {
    fn default() -> Self {
        Self::new(1, 2)
    }
}

impl TokenFilter for NgramTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.to_string();
        let chars: Vec<char> = text.chars().collect();
        let char_count = chars.len();

        if char_count == 0 {
            return (true, None);
        }

        let mut extra_tokens: Vec<Token<'a>> = Vec::new();
        let mut first = true;

        for n in self.min_gram..=self.max_gram {
            if n > char_count {
                break;
            }
            for start in 0..=(char_count - n) {
                let gram: String = chars[start..start + n].iter().collect();
                if first {
                    token.term = Cow::Owned(gram);
                    first = false;
                } else {
                    let mut t = Token {
                        term: Cow::Owned(gram),
                        start_offset: token.start_offset,
                        end_offset: token.end_offset,
                        position: token.position,
                    };
                    let _ = &mut t;
                    extra_tokens.push(t);
                }
            }
        }

        if self.preserve_original && !first {
            extra_tokens.push(Token {
                term: Cow::Owned(text.to_string()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            });
        }

        if first {
            // No n-grams produced (token too short)
            if self.preserve_original {
                return (false, None);
            }
            return (true, None);
        }

        if extra_tokens.is_empty() {
            (false, None)
        } else {
            (false, Some(extra_tokens))
        }
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
    fn test_bigrams() {
        let filter = NgramTokenFilter::new(2, 2);
        let mut token = make_token("pizza");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "pi");
        let extra = extra.unwrap();
        let terms: Vec<&str> = extra.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["iz", "zz", "za"]);
    }

    #[test]
    fn test_min_max() {
        let filter = NgramTokenFilter::new(2, 3);
        let mut token = make_token("abcd");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        let extra = extra.unwrap();
        // 2-grams: ab, bc, cd; 3-grams: abc, bcd
        let mut all = vec![token.term.to_string()];
        all.extend(extra.iter().map(|t| t.term.to_string()));
        assert_eq!(all, vec!["ab", "bc", "cd", "abc", "bcd"]);
    }

    #[test]
    fn test_short_token() {
        let filter = NgramTokenFilter::new(3, 3);
        let mut token = make_token("ab");
        let (remove, _) = filter.filter(&mut token);
        assert!(remove); // too short, removed
    }

    #[test]
    fn test_preserve_original() {
        let filter = NgramTokenFilter::new(2, 2).with_preserve_original(true);
        let mut token = make_token("abc");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        let extra = extra.unwrap();
        let last = extra.last().unwrap();
        assert_eq!(last.term.as_ref(), "abc");
    }
}
