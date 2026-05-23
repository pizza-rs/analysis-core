use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes Hindi/Devanagari text. Equivalent to Lucene's `HindiNormalizationFilter`.
///
/// Rules:
/// - Dead consonant + halant → anusvara (bindu)
/// - Chandrabindu → bindu
/// - Nukta → deleted
/// - Nukta-composed forms → base consonant
/// - ZWJ/ZWNJ → deleted
/// - Virama → deleted
/// - Short/Chandra vowels → standard equivalents
/// - Long vowels → short equivalents
#[derive(Clone, Debug, Default)]
pub struct HindiNormalizationTokenFilter;

impl HindiNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for HindiNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;

        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];
            match c {
                // Nukta → delete
                '\u{093C}' => {
                    changed = true;
                }
                // Chandrabindu → Bindu (Anusvara)
                '\u{0901}' => {
                    result.push('\u{0902}');
                    changed = true;
                }
                // ZWJ/ZWNJ → delete
                '\u{200D}' | '\u{200C}' => {
                    changed = true;
                }
                // Virama → delete
                '\u{094D}' => {
                    // Check for dead n + halant → anusvara
                    // If previous char was NA (\u0928), replace it with bindu
                    if !result.is_empty() {
                        let last = result.chars().last().unwrap();
                        if last == '\u{0928}' {
                            // Replace the trailing NA with bindu
                            let new_len = result.len() - '\u{0928}'.len_utf8();
                            result.truncate(new_len);
                            result.push('\u{0902}');
                            changed = true;
                        } else {
                            changed = true;
                        }
                    } else {
                        changed = true;
                    }
                }
                // Nukta-composed consonants → base
                '\u{0929}' => {
                    result.push('\u{0928}');
                    changed = true;
                }
                '\u{0931}' => {
                    result.push('\u{0930}');
                    changed = true;
                }
                '\u{0934}' => {
                    result.push('\u{0933}');
                    changed = true;
                }
                '\u{0958}' => {
                    result.push('\u{0915}');
                    changed = true;
                }
                '\u{0959}' => {
                    result.push('\u{0916}');
                    changed = true;
                }
                '\u{095A}' => {
                    result.push('\u{0917}');
                    changed = true;
                }
                '\u{095B}' => {
                    result.push('\u{091C}');
                    changed = true;
                }
                '\u{095C}' => {
                    result.push('\u{0921}');
                    changed = true;
                }
                '\u{095D}' => {
                    result.push('\u{0922}');
                    changed = true;
                }
                '\u{095E}' => {
                    result.push('\u{092B}');
                    changed = true;
                }
                '\u{095F}' => {
                    result.push('\u{092F}');
                    changed = true;
                }
                // Independent vowels: chandra/short → standard
                '\u{090D}' | '\u{090E}' => {
                    result.push('\u{090F}');
                    changed = true;
                }
                '\u{0911}' | '\u{0912}' => {
                    result.push('\u{0913}');
                    changed = true;
                }
                '\u{0972}' => {
                    result.push('\u{0905}');
                    changed = true;
                }
                // Independent vowels: long → short
                '\u{0906}' => {
                    result.push('\u{0905}');
                    changed = true;
                }
                '\u{0908}' => {
                    result.push('\u{0907}');
                    changed = true;
                }
                '\u{090A}' => {
                    result.push('\u{0909}');
                    changed = true;
                }
                '\u{0960}' => {
                    result.push('\u{090B}');
                    changed = true;
                }
                '\u{0961}' => {
                    result.push('\u{090C}');
                    changed = true;
                }
                '\u{0910}' => {
                    result.push('\u{090F}');
                    changed = true;
                }
                '\u{0914}' => {
                    result.push('\u{0913}');
                    changed = true;
                }
                // Dependent vowels: chandra/short → standard
                '\u{0945}' | '\u{0946}' => {
                    result.push('\u{0947}');
                    changed = true;
                }
                '\u{0949}' | '\u{094A}' => {
                    result.push('\u{094B}');
                    changed = true;
                }
                // Dependent vowels: long → short
                '\u{0940}' => {
                    result.push('\u{093F}');
                    changed = true;
                }
                '\u{0942}' => {
                    result.push('\u{0941}');
                    changed = true;
                }
                '\u{0944}' => {
                    result.push('\u{0943}');
                    changed = true;
                }
                '\u{0963}' => {
                    result.push('\u{0962}');
                    changed = true;
                }
                '\u{0948}' => {
                    result.push('\u{0947}');
                    changed = true;
                }
                '\u{094C}' => {
                    result.push('\u{094B}');
                    changed = true;
                }
                _ => {
                    result.push(c);
                }
            }
            i += 1;
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
    fn test_chandrabindu_to_bindu() {
        let f = HindiNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0901}", 0, 3, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0902}");
    }

    #[test]
    fn test_nukta_deletion() {
        let f = HindiNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0915}\u{093C}", 0, 6, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0915}");
    }

    #[test]
    fn test_nukta_composed() {
        let f = HindiNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0958}", 0, 3, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0915}");
    }

    #[test]
    fn test_long_to_short_vowel() {
        let f = HindiNormalizationTokenFilter::new();
        let mut token = Token::new("\u{0906}", 0, 3, 0);
        f.filter(&mut token);
        assert_eq!(token.term, "\u{0905}");
    }
}
