use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Serbian Cyrillic and Latin diacritics to plain ASCII Latin.
///
/// Equivalent to Lucene's `SerbianNormalizationFilter` (aggressive/bald variant).
///
/// Converts Cyrillic characters to their Latin equivalents and removes
/// Latin diacritics (ž→z, č→c, ć→c, š→s, đ→dj).
#[derive(Clone, Debug, Default)]
pub struct SerbianNormalizationTokenFilter;

impl SerbianNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for SerbianNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        for c in text.chars() {
            match c {
                // Cyrillic lowercase
                'а' => result.push('a'),
                'б' => result.push('b'),
                'в' => result.push('v'),
                'г' => result.push('g'),
                'д' => result.push('d'),
                'ђ' => {
                    result.push('d');
                    result.push('j');
                }
                'е' => result.push('e'),
                'ж' => result.push('z'),
                'з' => result.push('z'),
                'и' => result.push('i'),
                'ј' => result.push('j'),
                'к' => result.push('k'),
                'л' => result.push('l'),
                'љ' => {
                    result.push('l');
                    result.push('j');
                }
                'м' => result.push('m'),
                'н' => result.push('n'),
                'њ' => {
                    result.push('n');
                    result.push('j');
                }
                'о' => result.push('o'),
                'п' => result.push('p'),
                'р' => result.push('r'),
                'с' => result.push('s'),
                'т' => result.push('t'),
                'ћ' => result.push('c'),
                'у' => result.push('u'),
                'ф' => result.push('f'),
                'х' => result.push('h'),
                'ц' => result.push('c'),
                'ч' => result.push('c'),
                'џ' => {
                    result.push('d');
                    result.push('z');
                }
                'ш' => result.push('s'),
                // Cyrillic uppercase
                'А' => result.push('A'),
                'Б' => result.push('B'),
                'В' => result.push('V'),
                'Г' => result.push('G'),
                'Д' => result.push('D'),
                'Ђ' => {
                    result.push('D');
                    result.push('j');
                }
                'Е' => result.push('E'),
                'Ж' => result.push('Z'),
                'З' => result.push('Z'),
                'И' => result.push('I'),
                'Ј' => result.push('J'),
                'К' => result.push('K'),
                'Л' => result.push('L'),
                'Љ' => {
                    result.push('L');
                    result.push('j');
                }
                'М' => result.push('M'),
                'Н' => result.push('N'),
                'Њ' => {
                    result.push('N');
                    result.push('j');
                }
                'О' => result.push('O'),
                'П' => result.push('P'),
                'Р' => result.push('R'),
                'С' => result.push('S'),
                'Т' => result.push('T'),
                'Ћ' => result.push('C'),
                'У' => result.push('U'),
                'Ф' => result.push('F'),
                'Х' => result.push('H'),
                'Ц' => result.push('C'),
                'Ч' => result.push('C'),
                'Џ' => {
                    result.push('D');
                    result.push('z');
                }
                'Ш' => result.push('S'),
                // Latin diacritics
                'ž' | 'Ž' => result.push(if c == 'ž' { 'z' } else { 'Z' }),
                'č' | 'Č' => result.push(if c == 'č' { 'c' } else { 'C' }),
                'ć' | 'Ć' => result.push(if c == 'ć' { 'c' } else { 'C' }),
                'š' | 'Š' => result.push(if c == 'š' { 's' } else { 'S' }),
                'đ' | 'Đ' => {
                    if c == 'đ' {
                        result.push('d');
                        result.push('j');
                    } else {
                        result.push('D');
                        result.push('j');
                    }
                }
                _ => {
                    result.push(c);
                    continue;
                }
            }
            changed = true;
        }

        if changed {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyrillic_to_latin() {
        let f = SerbianNormalizationTokenFilter::new();
        let mut token = Token::new("Београд", 0, 14, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "Beograd");
    }

    #[test]
    fn test_cyrillic_dje() {
        let f = SerbianNormalizationTokenFilter::new();
        let mut token = Token::new("ђак", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "djak");
    }

    #[test]
    fn test_latin_diacritics() {
        let f = SerbianNormalizationTokenFilter::new();
        let mut token = Token::new("čaša", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "casa");
    }

    #[test]
    fn test_latin_dj() {
        let f = SerbianNormalizationTokenFilter::new();
        let mut token = Token::new("đak", 0, 4, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "djak");
    }
}
