use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Greek stemmer that removes common Greek suffixes.
///
/// Based on the algorithm from Lucene's GreekStemmer, which applies
/// suffix-stripping rules for Greek morphology.
#[derive(Clone, Debug, Default)]
pub struct GreekStemTokenFilter;

impl GreekStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for GreekStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len < 4 {
            return (false, None);
        }

        if let Some(new_len) = stem_greek(&chars, len) {
            if new_len < len && new_len >= 3 {
                let stemmed: String = chars[..new_len].iter().collect();
                token.term = Cow::Owned(stemmed);
            }
        }

        (false, None)
    }
}

fn stem_greek(chars: &[char], len: usize) -> Option<usize> {
    // Greek suffixes (in order of length, longest first)

    // 4-char suffixes
    if len > 5 {
        let suffix: String = chars[len - 4..].iter().collect();
        match suffix.as_str() {
            "ιστα" | "ιστε" | "ιστη" | "ιστο" | "ουνε" | "ησου" | "ηθει" => {
                return Some(len - 4)
            }
            _ => {}
        }
    }

    // 3-char suffixes
    if len > 4 {
        let suffix: String = chars[len - 3..].iter().collect();
        match suffix.as_str() {
            "ιζα" | "ιζε" | "ιζω" | "ικα" | "ικε" | "ικο" | "ισα" | "ισε" | "ισω" | "ατα"
            | "ατε" | "ατο" | "ετα" | "ετε" | "ηκα" | "ηκε" | "ητα" | "ητε" | "ησε" | "ικη"
            | "ισω" | "ουν" | "ουσ" | "ωμα" | "ωσε" => return Some(len - 3),
            _ => {}
        }
    }

    // 2-char suffixes
    if len > 3 {
        let suffix: String = chars[len - 2..].iter().collect();
        match suffix.as_str() {
            "εα" | "ει" | "ες" | "ια" | "ιο" | "ις" | "οι" | "ος" | "ου" | "ων" | "ης" | "ας"
            | "αν" | "ηθ" | "ησ" | "αμ" | "αγ" | "ηγ" | "ημ" | "ηπ" | "υσ" | "αω" | "ηκ" | "ιε"
            | "εσ" => return Some(len - 2),
            _ => {}
        }
    }

    // 1-char suffixes
    if len > 3 {
        let last = chars[len - 1];
        match last {
            'α' | 'ε' | 'η' | 'ι' | 'ο' | 'υ' | 'ω' | 'ς' | 'ν' => return Some(len - 1),
            _ => {}
        }
    }

    None
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
    fn test_greek_stem() {
        let filter = GreekStemTokenFilter::new();
        let mut token = make_token("ελληνικα");
        filter.filter(&mut token);
        // Should remove suffix
        assert!(token.term.as_ref().len() < "ελληνικα".len());
    }

    #[test]
    fn test_short_word() {
        let filter = GreekStemTokenFilter::new();
        let mut token = make_token("και");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "και");
    }
}
