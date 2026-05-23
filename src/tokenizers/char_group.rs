use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits text on a configurable set of characters.
#[derive(Clone, Debug)]
pub struct CharGroupTokenizer {
    /// Characters to split on
    split_chars: Vec<char>,
    /// Whether whitespace is a split character
    split_on_whitespace: bool,
    /// Whether to split on a character class
    split_on_letter: bool,
    split_on_digit: bool,
    split_on_punctuation: bool,
    split_on_symbol: bool,
}

impl CharGroupTokenizer {
    /// Create a new `CharGroupTokenizer` that splits on the specified characters.
    pub fn new(split_chars: Vec<char>) -> Self {
        Self {
            split_chars,
            split_on_whitespace: false,
            split_on_letter: false,
            split_on_digit: false,
            split_on_punctuation: false,
            split_on_symbol: false,
        }
    }

    pub fn split_on_whitespace(mut self) -> Self {
        self.split_on_whitespace = true;
        self
    }

    pub fn split_on_letter(mut self) -> Self {
        self.split_on_letter = true;
        self
    }

    pub fn split_on_digit(mut self) -> Self {
        self.split_on_digit = true;
        self
    }

    pub fn split_on_punctuation(mut self) -> Self {
        self.split_on_punctuation = true;
        self
    }

    pub fn split_on_symbol(mut self) -> Self {
        self.split_on_symbol = true;
        self
    }

    fn is_split_char(&self, ch: char) -> bool {
        if self.split_chars.contains(&ch) {
            return true;
        }
        if self.split_on_whitespace && ch.is_whitespace() {
            return true;
        }
        if self.split_on_letter && ch.is_alphabetic() {
            return true;
        }
        if self.split_on_digit && ch.is_numeric() {
            // is_numeric covers ASCII digits AND Arabic-Indic, Devanagari,
            // Bengali, fullwidth, etc. — see Unicode UCD `Nd`/`Nl`/`No`.
            return true;
        }
        if self.split_on_punctuation && Self::is_punctuation(ch) {
            return true;
        }
        if self.split_on_symbol && !ch.is_alphanumeric() && !ch.is_whitespace() {
            return true;
        }
        false
    }

    /// Unicode-aware punctuation check covering ASCII, CJK (`、。！？「」`),
    /// Latin-1 supplement (`¡¿`), general punctuation (`‒–—…‘’“”`), and the
    /// CJK symbols block.
    fn is_punctuation(ch: char) -> bool {
        if ch.is_ascii_punctuation() {
            return true;
        }
        matches!(
            ch as u32,
            0x00A1 | 0x00A7 | 0x00B6 | 0x00BF              // ¡ § ¶ ¿
            | 0x2010..=0x2027                              // hyphens, dashes, quotes, …
            | 0x2030..=0x205E                              // ‰ ′ ″ ‹ › etc.
            | 0x2E00..=0x2E7F                              // supplemental punctuation
            | 0x3000..=0x303F                              // CJK symbols & punctuation
            | 0xFF01..=0xFF0F | 0xFF1A..=0xFF20            // fullwidth ASCII punctuation
            | 0xFF3B..=0xFF40 | 0xFF5B..=0xFF65
        )
    }
}

impl Tokenizer for CharGroupTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut start = None;

        for (i, ch) in text.char_indices() {
            if self.is_split_char(ch) {
                if let Some(s) = start {
                    tokens.push(Token::new(&text[s..i], s as u32, i as u32, position));
                    position += 1;
                    start = None;
                }
            } else if start.is_none() {
                start = Some(i);
            }
        }

        if let Some(s) = start {
            tokens.push(Token::new(
                &text[s..],
                s as u32,
                text.len() as u32,
                position,
            ));
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_group() {
        let t = CharGroupTokenizer::new(vec!['-', '_']).split_on_whitespace();
        let tokens = t.tokenize("hello-world_test foo");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world", "test", "foo"]);
    }

    #[test]
    fn test_split_on_cjk_punctuation() {
        let t = CharGroupTokenizer::new(vec![]).split_on_punctuation();
        let tokens = t.tokenize("你好，世界。再见！");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["你好", "世界", "再见"]);
    }

    #[test]
    fn test_split_on_unicode_digits() {
        let t = CharGroupTokenizer::new(vec![]).split_on_digit();
        // Arabic-Indic digits ٠-٩ should also split.
        let tokens = t.tokenize("abc٤٢def");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["abc", "def"]);
    }
}
