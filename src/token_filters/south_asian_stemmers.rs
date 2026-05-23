use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Telugu stemmer based on suffix stripping.
///
/// Removes common Telugu inflectional suffixes (vibhakti, tense markers).
#[derive(Clone, Debug, Default)]
pub struct TeluguStemTokenFilter;

impl TeluguStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for TeluguStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let char_count = text.chars().count();
        if char_count < 3 {
            return (false, None);
        }

        let stemmed = stem_telugu(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_telugu(word: &str) -> String {
    let lower = word.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let len = chars.len();

    if len < 3 {
        return lower;
    }

    // 3-char suffixes (case markers / postpositions)
    if len > 4 {
        let suffix3: String = chars[len - 3..].iter().collect();
        match suffix3.as_str() {
            "లో" | "కు" | "తో" | "లను" | "లకు" | "లతో" | "ము" | "డు" | "ను" | "గా" =>
            {
                // These are multi-char Telugu suffixes
            }
            _ => {}
        }
        // Telugu vibhakti (case suffixes)
        let suffixes_3 = ["లను", "లకు", "లతో", "లలో"];
        for suffix in &suffixes_3 {
            if lower.ends_with(suffix) {
                let byte_len = lower.len() - suffix.len();
                if byte_len >= 6 {
                    // At least 2 Telugu chars
                    return lower[..byte_len].to_string();
                }
            }
        }
    }

    // 2-char suffixes
    if len > 3 {
        let suffixes_2 = ["లో", "కు", "తో", "ను", "డు", "ము", "గా", "కి", "ని", "లు", "ల"];
        for suffix in &suffixes_2 {
            if lower.ends_with(suffix) {
                let byte_len = lower.len() - suffix.len();
                if byte_len >= 3 {
                    return lower[..byte_len].to_string();
                }
            }
        }
    }

    lower
}

/// Kannada stemmer based on suffix stripping.
///
/// Removes common Kannada inflectional suffixes.
#[derive(Clone, Debug, Default)]
pub struct KannadaStemTokenFilter;

impl KannadaStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for KannadaStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let char_count = text.chars().count();
        if char_count < 3 {
            return (false, None);
        }

        let stemmed = stem_kannada(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_kannada(word: &str) -> String {
    let lower = word.to_lowercase();

    if lower.chars().count() < 3 {
        return lower;
    }

    // Kannada case/plural suffixes (vibhakti pratyaya)
    let suffixes_3 = ["ಗಳನ್ನು", "ಗಳಿಗೆ", "ಗಳಲ್ಲಿ", "ಗಳಿಂದ"];
    for suffix in &suffixes_3 {
        if lower.ends_with(suffix) {
            let byte_len = lower.len() - suffix.len();
            if byte_len >= 3 {
                return lower[..byte_len].to_string();
            }
        }
    }

    let suffixes_2 = ["ಗಳು", "ಗಳ", "ನ್ನು", "ಇಗೆ", "ಅಲ್ಲಿ", "ಇಂದ", "ಲ್ಲಿ", "ನಲ್ಲಿ"];
    for suffix in &suffixes_2 {
        if lower.ends_with(suffix) {
            let byte_len = lower.len() - suffix.len();
            if byte_len >= 3 {
                return lower[..byte_len].to_string();
            }
        }
    }

    let suffixes_1 = ["ವು", "ನು", "ಗೆ", "ದ", "ರು"];
    for suffix in &suffixes_1 {
        if lower.ends_with(suffix) {
            let byte_len = lower.len() - suffix.len();
            if byte_len >= 6 {
                return lower[..byte_len].to_string();
            }
        }
    }

    lower
}

/// Tamil stemmer based on suffix stripping.
///
/// Removes common Tamil inflectional suffixes (case markers, plural).
#[derive(Clone, Debug, Default)]
pub struct TamilStemTokenFilter;

impl TamilStemTokenFilter {
    pub fn new() -> Self {
        Self
    }
}

impl TokenFilter for TamilStemTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let char_count = text.chars().count();
        if char_count < 3 {
            return (false, None);
        }

        let stemmed = stem_tamil(text);
        if stemmed != text {
            token.term = Cow::Owned(stemmed);
        }
        (false, None)
    }
}

fn stem_tamil(word: &str) -> String {
    let lower = word.to_lowercase();

    if lower.chars().count() < 3 {
        return lower;
    }

    // Tamil case suffixes (longest first)
    let suffixes = [
        "களுக்கு",
        "களால்",
        "களில்",
        "களை",
        "க்கு",
        "ுக்கு",
        "ால்",
        "ில்",
        "ுடன்",
        "கள்",
        "களை",
        "என்",
        "ை",
        "ல்",
        "ன்",
        "ம்",
        "ர்",
    ];

    for suffix in &suffixes {
        if lower.ends_with(suffix) {
            let byte_len = lower.len() - suffix.len();
            if byte_len >= 6 {
                return lower[..byte_len].to_string();
            }
        }
    }

    lower
}

#[cfg(test)]
mod tests {
    use super::*;
    use pizza_engine::analysis::TokenFilter;

    #[test]
    fn test_telugu_stem() {
        let filter = TeluguStemTokenFilter::new();
        let mut token = Token::new("పుస్తకాలు", 0, 30, 0);
        filter.filter(&mut token);
        // Should strip plural suffix "లు"
        assert!(token.term.len() < "పుస్తకాలు".len());
    }

    #[test]
    fn test_kannada_stem() {
        let filter = KannadaStemTokenFilter::new();
        let mut token = Token::new("ಮನೆಗಳು", 0, 21, 0);
        filter.filter(&mut token);
        // Should strip plural suffix "ಗಳು"
        assert!(token.term.len() < "ಮನೆಗಳು".len());
    }

    #[test]
    fn test_tamil_stem() {
        let filter = TamilStemTokenFilter::new();
        let mut token = Token::new("புத்தகங்கள்", 0, 33, 0);
        filter.filter(&mut token);
        // Should strip plural suffix
        assert!(token.term.len() < "புத்தகங்கள்".len());
    }
}
