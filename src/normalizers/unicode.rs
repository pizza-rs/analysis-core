use pizza_engine::analysis::Normalizer;

/// Unicode normalization normalizer.
///
/// Applies Unicode NFC, NFD, NFKC, or NFKD normalization to the input text.
/// This is important for consistent handling of composed vs decomposed characters.
///
/// For example, "é" (U+00E9) vs "e" + combining acute (U+0065 U+0301).
///
/// Modes:
/// - NFC: Canonical Decomposition, followed by Canonical Composition (default)
/// - NFD: Canonical Decomposition
/// - NFKC: Compatibility Decomposition, followed by Canonical Composition
/// - NFKD: Compatibility Decomposition
#[derive(Clone, Debug)]
pub enum UnicodeNormForm {
    Nfc,
    Nfd,
    Nfkc,
    Nfkd,
}

#[derive(Clone, Debug)]
pub struct UnicodeNormalizer {
    form: UnicodeNormForm,
}

impl Default for UnicodeNormalizer {
    fn default() -> Self {
        Self {
            form: UnicodeNormForm::Nfkc,
        }
    }
}

impl UnicodeNormalizer {
    pub fn new(form: UnicodeNormForm) -> Self {
        Self { form }
    }

    pub fn nfc() -> Self {
        Self::new(UnicodeNormForm::Nfc)
    }

    pub fn nfd() -> Self {
        Self::new(UnicodeNormForm::Nfd)
    }

    pub fn nfkc() -> Self {
        Self::new(UnicodeNormForm::Nfkc)
    }

    pub fn nfkd() -> Self {
        Self::new(UnicodeNormForm::Nfkd)
    }

    /// Perform simple NFC-like normalization by composing common sequences.
    /// This is a simplified implementation that handles the most common cases
    /// without requiring a full Unicode database.
    fn normalize_text(&self, text: &str) -> Option<String> {
        match self.form {
            UnicodeNormForm::Nfkc | UnicodeNormForm::Nfc => {
                // Compose decomposed sequences
                self.compose(text)
            }
            UnicodeNormForm::Nfkd | UnicodeNormForm::Nfd => {
                // Decompose composed sequences
                self.decompose(text)
            }
        }
    }

    fn compose(&self, text: &str) -> Option<String> {
        let mut result = String::with_capacity(text.len());
        let mut changed = false;
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            // Check for base + combining mark sequences
            if let Some(&next) = chars.peek() {
                if is_combining_mark(next) {
                    if let Some(composed) = compose_pair(ch, next) {
                        result.push(composed);
                        chars.next(); // consume combining mark
                        changed = true;
                        continue;
                    }
                }
            }

            // NFKC: also apply compatibility mappings
            if matches!(self.form, UnicodeNormForm::Nfkc) {
                if let Some(compat) = compatibility_map(ch) {
                    result.push_str(compat);
                    changed = true;
                    continue;
                }
            }

            result.push(ch);
        }

        if changed {
            Some(result)
        } else {
            None
        }
    }

    fn decompose(&self, text: &str) -> Option<String> {
        let mut result = String::with_capacity(text.len() + text.len() / 4);
        let mut changed = false;

        for ch in text.chars() {
            if let Some((base, combining)) = decompose_char(ch) {
                result.push(base);
                result.push(combining);
                changed = true;
            } else if matches!(self.form, UnicodeNormForm::Nfkd) {
                if let Some(compat) = compatibility_map(ch) {
                    result.push_str(compat);
                    changed = true;
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        if changed {
            Some(result)
        } else {
            None
        }
    }
}

impl Normalizer for UnicodeNormalizer {
    fn normalize(&self, text: &mut String) {
        if let Some(normalized) = self.normalize_text(text) {
            *text = normalized;
        }
    }
}

/// Check if a character is a combining mark.
fn is_combining_mark(ch: char) -> bool {
    ('\u{0300}'..='\u{036F}').contains(&ch) // Combining Diacritical Marks
        || ('\u{1AB0}'..='\u{1AFF}').contains(&ch) // Combining Diacritical Marks Extended
        || ('\u{1DC0}'..='\u{1DFF}').contains(&ch) // Combining Diacritical Marks Supplement
        || ('\u{20D0}'..='\u{20FF}').contains(&ch) // Combining Diacritical Marks for Symbols
        || ('\u{FE20}'..='\u{FE2F}').contains(&ch) // Combining Half Marks
}

/// Compose a base character + combining mark into a single character.
fn compose_pair(base: char, combining: char) -> Option<char> {
    match (base, combining) {
        // Latin vowels with acute
        ('a', '\u{0301}') => Some('á'),
        ('e', '\u{0301}') => Some('é'),
        ('i', '\u{0301}') => Some('í'),
        ('o', '\u{0301}') => Some('ó'),
        ('u', '\u{0301}') => Some('ú'),
        ('A', '\u{0301}') => Some('Á'),
        ('E', '\u{0301}') => Some('É'),
        ('I', '\u{0301}') => Some('Í'),
        ('O', '\u{0301}') => Some('Ó'),
        ('U', '\u{0301}') => Some('Ú'),
        // Latin with grave
        ('a', '\u{0300}') => Some('à'),
        ('e', '\u{0300}') => Some('è'),
        ('i', '\u{0300}') => Some('ì'),
        ('o', '\u{0300}') => Some('ò'),
        ('u', '\u{0300}') => Some('ù'),
        ('A', '\u{0300}') => Some('À'),
        ('E', '\u{0300}') => Some('È'),
        ('I', '\u{0300}') => Some('Ì'),
        ('O', '\u{0300}') => Some('Ò'),
        ('U', '\u{0300}') => Some('Ù'),
        // Latin with circumflex
        ('a', '\u{0302}') => Some('â'),
        ('e', '\u{0302}') => Some('ê'),
        ('i', '\u{0302}') => Some('î'),
        ('o', '\u{0302}') => Some('ô'),
        ('u', '\u{0302}') => Some('û'),
        // Latin with tilde
        ('a', '\u{0303}') => Some('ã'),
        ('n', '\u{0303}') => Some('ñ'),
        ('o', '\u{0303}') => Some('õ'),
        ('N', '\u{0303}') => Some('Ñ'),
        // Latin with diaeresis
        ('a', '\u{0308}') => Some('ä'),
        ('e', '\u{0308}') => Some('ë'),
        ('i', '\u{0308}') => Some('ï'),
        ('o', '\u{0308}') => Some('ö'),
        ('u', '\u{0308}') => Some('ü'),
        ('A', '\u{0308}') => Some('Ä'),
        ('O', '\u{0308}') => Some('Ö'),
        ('U', '\u{0308}') => Some('Ü'),
        // Latin with cedilla
        ('c', '\u{0327}') => Some('ç'),
        ('C', '\u{0327}') => Some('Ç'),
        // Latin with ring above
        ('a', '\u{030A}') => Some('å'),
        ('A', '\u{030A}') => Some('Å'),
        _ => None,
    }
}

/// Decompose a precomposed character into base + combining mark.
fn decompose_char(ch: char) -> Option<(char, char)> {
    match ch {
        'á' => Some(('a', '\u{0301}')),
        'é' => Some(('e', '\u{0301}')),
        'í' => Some(('i', '\u{0301}')),
        'ó' => Some(('o', '\u{0301}')),
        'ú' => Some(('u', '\u{0301}')),
        'à' => Some(('a', '\u{0300}')),
        'è' => Some(('e', '\u{0300}')),
        'ì' => Some(('i', '\u{0300}')),
        'ò' => Some(('o', '\u{0300}')),
        'ù' => Some(('u', '\u{0300}')),
        'â' => Some(('a', '\u{0302}')),
        'ê' => Some(('e', '\u{0302}')),
        'î' => Some(('i', '\u{0302}')),
        'ô' => Some(('o', '\u{0302}')),
        'û' => Some(('u', '\u{0302}')),
        'ã' => Some(('a', '\u{0303}')),
        'ñ' => Some(('n', '\u{0303}')),
        'õ' => Some(('o', '\u{0303}')),
        'ä' => Some(('a', '\u{0308}')),
        'ë' => Some(('e', '\u{0308}')),
        'ï' => Some(('i', '\u{0308}')),
        'ö' => Some(('o', '\u{0308}')),
        'ü' => Some(('u', '\u{0308}')),
        'ç' => Some(('c', '\u{0327}')),
        'å' => Some(('a', '\u{030A}')),
        'Á' => Some(('A', '\u{0301}')),
        'É' => Some(('E', '\u{0301}')),
        'Í' => Some(('I', '\u{0301}')),
        'Ó' => Some(('O', '\u{0301}')),
        'Ú' => Some(('U', '\u{0301}')),
        'Ä' => Some(('A', '\u{0308}')),
        'Ö' => Some(('O', '\u{0308}')),
        'Ü' => Some(('U', '\u{0308}')),
        'Ñ' => Some(('N', '\u{0303}')),
        'Ç' => Some(('C', '\u{0327}')),
        _ => None,
    }
}

/// Compatibility mappings for NFKC/NFKD.
fn compatibility_map(ch: char) -> Option<&'static str> {
    match ch {
        // Ligatures
        '\u{FB00}' => Some("ff"),
        '\u{FB01}' => Some("fi"),
        '\u{FB02}' => Some("fl"),
        '\u{FB03}' => Some("ffi"),
        '\u{FB04}' => Some("ffl"),
        // Superscripts
        '\u{00B2}' => Some("2"),
        '\u{00B3}' => Some("3"),
        '\u{00B9}' => Some("1"),
        // Fractions
        '\u{00BC}' => Some("1/4"),
        '\u{00BD}' => Some("1/2"),
        '\u{00BE}' => Some("3/4"),
        // Fullwidth ASCII
        '\u{FF01}'..='\u{FF5E}' => None, // Handled by CJK width filter
        // Roman numerals
        '\u{2160}' => Some("I"),
        '\u{2161}' => Some("II"),
        '\u{2162}' => Some("III"),
        '\u{2163}' => Some("IV"),
        '\u{2164}' => Some("V"),
        '\u{2170}' => Some("i"),
        '\u{2171}' => Some("ii"),
        '\u{2172}' => Some("iii"),
        '\u{2173}' => Some("iv"),
        '\u{2174}' => Some("v"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfc_compose() {
        let norm = UnicodeNormalizer::nfc();
        let mut text = "e\u{0301}".to_string(); // e + combining acute
        norm.normalize(&mut text);
        assert_eq!(text, "é");
    }

    #[test]
    fn test_nfd_decompose() {
        let norm = UnicodeNormalizer::nfd();
        let mut text = "é".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "e\u{0301}");
    }

    #[test]
    fn test_nfkc_compat() {
        let norm = UnicodeNormalizer::nfkc();
        let mut text = "\u{FB01}nd".to_string(); // fi ligature + nd
        norm.normalize(&mut text);
        assert_eq!(text, "find");
    }

    #[test]
    fn test_already_normalized() {
        let norm = UnicodeNormalizer::nfc();
        let mut text = "hello".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "hello"); // unchanged
    }
}
