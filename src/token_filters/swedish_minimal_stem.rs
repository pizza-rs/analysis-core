use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Swedish minimal stemmer. Strips common plural suffixes for Swedish nouns.
/// Based on Jacques Savoy's algorithm, adapted for Swedish.
#[derive(Clone, Debug)]
pub struct SwedishMinimalStemTokenFilter;

impl SwedishMinimalStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SwedishMinimalStemTokenFilter {
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

impl TokenFilter for SwedishMinimalStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let chars: Vec<char> = text.chars().collect();
        let mut len = chars.len();

        // Remove genitiv -s
        if len > 4 && chars[len - 1] == 's' {
            len -= 1;
        }

        let new_len = if len > 6
            && (ends_with_str(&chars[..len], "arne")
                || ends_with_str(&chars[..len], "erna")
                || ends_with_str(&chars[..len], "arna")
                || ends_with_str(&chars[..len], "orna")
                || ends_with_str(&chars[..len], "aren"))
        {
            len - 4
        } else if len > 5 && ends_with_str(&chars[..len], "are") {
            len - 3
        } else if len > 4
            && (ends_with_str(&chars[..len], "ar")
                || ends_with_str(&chars[..len], "at")
                || ends_with_str(&chars[..len], "er")
                || ends_with_str(&chars[..len], "et")
                || ends_with_str(&chars[..len], "or")
                || ends_with_str(&chars[..len], "en"))
        {
            len - 2
        } else if len > 3 && matches!(chars[len - 1], 'a' | 'e' | 'n') {
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
