use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Sorani (Central Kurdish) stemmer.
/// Removes common suffixes: postpositions, possessive pronouns, singular/plural markers.
#[derive(Clone, Debug)]
pub struct SoraniStemTokenFilter;

impl SoraniStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SoraniStemTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

fn ends_with_str(chars: &[char], suffix: &str) -> bool {
    let suffix_chars: Vec<char> = suffix.chars().collect();
    if chars.len() < suffix_chars.len() {
        return false;
    }
    let start = chars.len() - suffix_chars.len();
    chars[start..] == suffix_chars[..]
}

impl TokenFilter for SoraniStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let mut len = chars.len();

        // postposition
        if len > 5 && ends_with_str(&chars[..len], "دا") {
            len -= 2;
        } else if len > 4 && ends_with_str(&chars[..len], "نا") {
            len -= 1;
        } else if len > 6 && ends_with_str(&chars[..len], "ەوە") {
            len -= 3;
        }

        // possessive pronoun
        if len > 6
            && (ends_with_str(&chars[..len], "مان")
                || ends_with_str(&chars[..len], "یان")
                || ends_with_str(&chars[..len], "تان"))
        {
            len -= 3;
        }

        // indefinite singular ezafe
        let new_len = if len > 6 && ends_with_str(&chars[..len], "ێکی") {
            len - 3
        } else if len > 7 && ends_with_str(&chars[..len], "یەکی") {
            len - 4
        }
        // indefinite singular
        else if len > 5 && ends_with_str(&chars[..len], "ێک") {
            len - 2
        } else if len > 6 && ends_with_str(&chars[..len], "یەک") {
            len - 3
        }
        // definite singular
        else if len > 6 && ends_with_str(&chars[..len], "ەکە") {
            len - 3
        } else if len > 5 && ends_with_str(&chars[..len], "کە") {
            len - 2
        }
        // definite plural
        else if len > 7 && ends_with_str(&chars[..len], "ەکان") {
            len - 4
        } else if len > 6 && ends_with_str(&chars[..len], "کان") {
            len - 3
        }
        // indefinite plural ezafe
        else if len > 7 && ends_with_str(&chars[..len], "یانی") {
            len - 4
        } else if len > 6 && ends_with_str(&chars[..len], "انی") {
            len - 3
        }
        // indefinite plural
        else if len > 6 && ends_with_str(&chars[..len], "یان") {
            len - 3
        } else if len > 5 && ends_with_str(&chars[..len], "ان") {
            len - 2
        }
        // demonstrative plural
        else if len > 7 && ends_with_str(&chars[..len], "یانە") {
            len - 4
        } else if len > 6 && ends_with_str(&chars[..len], "انە") {
            len - 3
        }
        // demonstrative singular
        else if len > 5
            && (ends_with_str(&chars[..len], "ایە") || ends_with_str(&chars[..len], "ەیە"))
        {
            len - 2
        } else if len > 4 && ends_with_str(&chars[..len], "ە") {
            len - 1
        }
        // absolute singular ezafe
        else if len > 4 && ends_with_str(&chars[..len], "ی") {
            len - 1
        } else {
            len
        };

        if new_len < chars.len() {
            let result: String = chars[..new_len].iter().collect();
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
