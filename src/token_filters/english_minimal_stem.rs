use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// English minimal (S-stemmer) from Donna Harman's "How Effective Is Suffixing?"
/// Only removes plural 's' with minimal exceptions.
#[derive(Clone, Debug)]
pub struct EnglishMinimalStemTokenFilter;

impl EnglishMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnglishMinimalStemTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for EnglishMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 3 || chars[len - 1] != 's' {
            return (false, None);
        }

        let new_len = match chars[len - 2] {
            'u' | 's' => len, // not a plural form
            'e' => {
                if len > 3 && chars[len - 3] == 'i' && chars[len - 4] != 'a' && chars[len - 4] != 'e' {
                    // -ies → -y (but not -aies, -eies)
                    let mut new_chars = chars[..len - 2].to_vec();
                    new_chars[len - 3] = 'y';
                    let result: String = new_chars.iter().collect();
                    token.term = Cow::Owned(result);
                    return (false, None);
                }
                if len > 3
                    && (chars[len - 3] == 'i'
                        || chars[len - 3] == 'a'
                        || chars[len - 3] == 'o'
                        || chars[len - 3] == 'e')
                {
                    len // keep -ies, -aes, -oes, -ees
                } else {
                    len - 1 // remove -es
                }
            }
            _ => len - 1, // remove -s
        };

        if new_len < len {
            let result: String = chars[..new_len].iter().collect();
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
