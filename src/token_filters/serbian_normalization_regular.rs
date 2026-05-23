use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Serbian normalization filter (regular/standard variant).
/// Normalizes Serbian Cyrillic characters to their Latin equivalents
/// using the standard mapping (not the IETF variant).
#[derive(Clone, Debug)]
pub struct SerbianNormalizationRegularTokenFilter;

impl SerbianNormalizationRegularTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SerbianNormalizationRegularTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for SerbianNormalizationRegularTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            let mapped = match c {
                'а' => 'a', 'б' => 'b', 'в' => 'v', 'г' => 'g',
                'д' => 'd', 'е' => 'e', 'ж' => 'ž', 'з' => 'z',
                'и' => 'i', 'к' => 'k', 'л' => 'l', 'м' => 'm',
                'н' => 'n', 'о' => 'o', 'п' => 'p', 'р' => 'r',
                'с' => 's', 'т' => 't', 'у' => 'u', 'ф' => 'f',
                'х' => 'h', 'ц' => 'c', 'ч' => 'č', 'ш' => 'š',
                'ђ' => 'đ', 'ј' => 'j', 'љ' => 'l', 'њ' => 'n',
                'ћ' => 'ć', 'џ' => 'd', 'і' => 'i',
                'А' => 'A', 'Б' => 'B', 'В' => 'V', 'Г' => 'G',
                'Д' => 'D', 'Е' => 'E', 'Ж' => 'Ž', 'З' => 'Z',
                'И' => 'I', 'К' => 'K', 'Л' => 'L', 'М' => 'M',
                'Н' => 'N', 'О' => 'O', 'П' => 'P', 'Р' => 'R',
                'С' => 'S', 'Т' => 'T', 'У' => 'U', 'Ф' => 'F',
                'Х' => 'H', 'Ц' => 'C', 'Ч' => 'Č', 'Ш' => 'Š',
                'Ђ' => 'Đ', 'Ј' => 'J', 'Љ' => 'L', 'Њ' => 'N',
                'Ћ' => 'Ć', 'Џ' => 'D', 'І' => 'I',
                other => other,
            };
            if mapped != c {
                changed = true;
            }
            result.push(mapped);
        }

        if changed {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}
