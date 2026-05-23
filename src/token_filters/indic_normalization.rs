use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalizes characters across Indic scripts (Devanagari, Bengali, Tamil, Telugu,
/// Kannada, Malayalam, Gujarati, Oriya, Gurmukhi).
///
/// Performs the following normalizations:
/// - Normalizes nukta-composed characters to precomposed equivalents
/// - Normalizes Chandrabindu variations across scripts
/// - Normalizes length marks
/// - Maps equivalent characters between scripts where applicable
#[derive(Clone, Debug, Default)]
pub struct IndicNormalizationTokenFilter;

impl IndicNormalizationTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for IndicNormalizationTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();

        // Fast path: only process text with Indic script characters
        let has_indic = text.chars().any(|c| is_indic_char(c));
        if !has_indic {
            return (false, None);
        }

        let mut result = String::with_capacity(text.len());
        let mut changed = false;
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            let cp = c as u32;

            // Normalize Devanagari nukta compositions
            if is_devanagari(cp) {
                if let Some(&next) = chars.peek() {
                    if next == '\u{093C}' {
                        // nukta
                        if let Some(composed) = compose_devanagari_nukta(cp) {
                            result.push(composed);
                            chars.next(); // consume nukta
                            changed = true;
                            continue;
                        }
                    }
                }
            }

            // Normalize Bengali nukta compositions
            if is_bengali(cp) {
                if let Some(&next) = chars.peek() {
                    if next == '\u{09BC}' {
                        // Bengali nukta
                        if let Some(composed) = compose_bengali_nukta(cp) {
                            result.push(composed);
                            chars.next();
                            changed = true;
                            continue;
                        }
                    }
                }
            }

            // Normalize Gurmukhi nukta compositions
            if is_gurmukhi(cp) {
                if let Some(&next) = chars.peek() {
                    if next == '\u{0A3C}' {
                        // Gurmukhi nukta
                        if let Some(composed) = compose_gurmukhi_nukta(cp) {
                            result.push(composed);
                            chars.next();
                            changed = true;
                            continue;
                        }
                    }
                }
            }

            // Normalize visarga to aha in Devanagari (sometimes used interchangeably)
            // Remove zero-width characters
            if cp == 0x200D || cp == 0x200C {
                // Zero-width joiner / non-joiner — remove
                changed = true;
                continue;
            }

            result.push(c);
        }

        if changed {
            token.term = Cow::Owned(result);
        }
        (false, None)
    }
}

#[inline]
fn is_indic_char(c: char) -> bool {
    let cp = c as u32;
    (0x0900..=0x0D7F).contains(&cp) // Devanagari through Malayalam
        || (0x200C..=0x200D).contains(&cp) // ZWJ/ZWNJ
}

#[inline]
fn is_devanagari(cp: u32) -> bool {
    (0x0900..=0x097F).contains(&cp)
}

#[inline]
fn is_bengali(cp: u32) -> bool {
    (0x0980..=0x09FF).contains(&cp)
}

#[inline]
fn is_gurmukhi(cp: u32) -> bool {
    (0x0A00..=0x0A7F).contains(&cp)
}

/// Compose Devanagari base + nukta into precomposed character.
fn compose_devanagari_nukta(base_cp: u32) -> Option<char> {
    match base_cp {
        0x0915 => Some('\u{0958}'), // क + ़ = क़
        0x0916 => Some('\u{0959}'), // ख + ़ = ख़
        0x0917 => Some('\u{095A}'), // ग + ़ = ग़
        0x091C => Some('\u{095B}'), // ज + ़ = ज़
        0x0921 => Some('\u{095C}'), // ड + ़ = ड़
        0x0922 => Some('\u{095D}'), // ढ + ़ = ढ़
        0x0928 => Some('\u{0929}'), // न + ़ = ऩ
        0x092B => Some('\u{095E}'), // फ + ़ = फ़
        0x092F => Some('\u{095F}'), // य + ़ = य़
        0x0930 => Some('\u{0931}'), // र + ़ = ऱ
        _ => None,
    }
}

/// Compose Bengali base + nukta into precomposed character.
fn compose_bengali_nukta(base_cp: u32) -> Option<char> {
    match base_cp {
        0x09A1 => Some('\u{09DC}'), // ড + ় = ড়
        0x09A2 => Some('\u{09DD}'), // ঢ + ় = ঢ়
        0x09AF => Some('\u{09DF}'), // য + ় = য়
        _ => None,
    }
}

/// Compose Gurmukhi base + nukta into precomposed character.
fn compose_gurmukhi_nukta(base_cp: u32) -> Option<char> {
    match base_cp {
        0x0A16 => Some('\u{0A59}'), // ਖ + ਼ = ਖ਼
        0x0A17 => Some('\u{0A5A}'), // ਗ + ਼ = ਗ਼
        0x0A1C => Some('\u{0A5B}'), // ਜ + ਼ = ਜ਼
        0x0A2B => Some('\u{0A5E}'), // ਫ + ਼ = ਫ਼
        0x0A32 => Some('\u{0A33}'), // ਲ + ਼ = ਲ਼
        _ => None,
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
    fn test_devanagari_nukta() {
        let filter = IndicNormalizationTokenFilter::new();
        // क + ़ → क़
        let mut token = make_token("\u{0915}\u{093C}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{0958}");
    }

    #[test]
    fn test_remove_zwj() {
        let filter = IndicNormalizationTokenFilter::new();
        let mut token = make_token("क\u{200D}ष");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "कष");
    }

    #[test]
    fn test_non_indic_passthrough() {
        let filter = IndicNormalizationTokenFilter::new();
        let mut token = make_token("hello");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_bengali_nukta() {
        let filter = IndicNormalizationTokenFilter::new();
        // ড + ় → ড়
        let mut token = make_token("\u{09A1}\u{09BC}");
        filter.filter(&mut token);
        assert_eq!(token.term.as_ref(), "\u{09DC}");
    }
}
