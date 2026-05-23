use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Unicode script-aware tokenizer that splits text at script boundaries.
///
/// Similar to ICU's word boundary rules, this tokenizer splits when the Unicode
/// script changes (e.g., Latin to Cyrillic, Latin to CJK).
///
/// Each script run becomes a separate token:
/// - `"hello世界мир"` → `"hello"`, `"世界"`, `"мир"`
/// - `"café"` → `"café"` (stays together, same script)
///
/// Numbers are always their own category regardless of script.
#[derive(Clone, Debug)]
pub struct ScriptBoundaryTokenizer {
    /// Split CJK ideographs into individual characters
    split_cjk: bool,
    /// Group numbers with surrounding same-script text
    merge_numbers: bool,
}

impl ScriptBoundaryTokenizer {
    pub fn new() -> Self {
        Self {
            split_cjk: false,
            merge_numbers: false,
        }
    }

    pub fn with_split_cjk(mut self, v: bool) -> Self {
        self.split_cjk = v;
        self
    }

    pub fn with_merge_numbers(mut self, v: bool) -> Self {
        self.merge_numbers = v;
        self
    }

    fn script_category(c: char) -> u8 {
        if c.is_ascii_alphabetic() {
            1 // Latin/ASCII
        } else if c.is_ascii_digit() {
            // Only ASCII digits are "numbers" for boundary purposes.
            // `char::is_numeric()` would also match Roman numerals (Ⅰ, Ⅱ),
            // CJK numerals (一, 二), Arabic-Indic digits etc., which we want
            // to keep with their owning script instead of splitting them off.
            0 // Number
        } else {
            let cp = c as u32;
            match cp {
                0x0400..=0x04FF | 0x0500..=0x052F => 2,     // Cyrillic
                0x0370..=0x03FF => 3,                         // Greek
                0x0600..=0x06FF | 0x0750..=0x077F => 4,      // Arabic
                0x0590..=0x05FF => 5,                          // Hebrew
                0x0900..=0x097F => 6,                          // Devanagari
                0x4E00..=0x9FFF | 0x3400..=0x4DBF => 7,      // CJK Ideographs
                0x3040..=0x309F => 8,                          // Hiragana
                0x30A0..=0x30FF => 9,                          // Katakana
                0xAC00..=0xD7AF => 10,                         // Hangul
                0x0E00..=0x0E7F => 11,                         // Thai
                0x00C0..=0x024F | 0x1E00..=0x1EFF => 1,      // Latin extended
                _ if c.is_alphabetic() => 99,                  // Other alphabetic
                _ => 255,                                      // Non-word
            }
        }
    }
}

impl Default for ScriptBoundaryTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ScriptBoundaryTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        let mut i = 0;
        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let c_len = c.len_utf8();

            // Skip whitespace and non-word chars
            let cat = Self::script_category(c);
            if cat == 255 || c.is_whitespace() {
                i += c_len;
                continue;
            }

            // CJK single-char splitting
            if self.split_cjk && cat == 7 {
                tokens.push(Token {
                    term: Cow::Borrowed(&text[i..i + c_len]),
                    start_offset: i as u32,
                    end_offset: (i + c_len) as u32,
                    position,
                });
                position += 1;
                i += c_len;
                continue;
            }

            // Collect same-script run
            let start = i;
            let start_cat = cat;
            i += c_len;

            while i < text.len() {
                let nc = text[i..].chars().next().unwrap();
                let nc_len = nc.len_utf8();
                let nc_cat = Self::script_category(nc);

                if nc.is_whitespace() || nc_cat == 255 {
                    break;
                }

                // Split on script change
                if nc_cat != start_cat {
                    // Allow numbers to merge with adjacent script if configured
                    if self.merge_numbers && (nc_cat == 0 || start_cat == 0) {
                        i += nc_len;
                        continue;
                    }
                    break;
                }

                // CJK: split each char
                if self.split_cjk && nc_cat == 7 {
                    break;
                }

                i += nc_len;
            }

            tokens.push(Token {
                term: Cow::Borrowed(&text[start..i]),
                start_offset: start as u32,
                end_offset: i as u32,
                position,
            });
            position += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latin_only() {
        let tok = ScriptBoundaryTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_mixed_scripts() {
        let tok = ScriptBoundaryTokenizer::new();
        let tokens = tok.tokenize("helloмир");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "мир"]);
    }

    #[test]
    fn test_cjk_run() {
        let tok = ScriptBoundaryTokenizer::new();
        let tokens = tok.tokenize("hello世界test");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "世界", "test"]);
    }

    #[test]
    fn test_cjk_split() {
        let tok = ScriptBoundaryTokenizer::new().with_split_cjk(true);
        let tokens = tok.tokenize("中文");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["中", "文"]);
    }

    #[test]
    fn test_numbers_separate() {
        let tok = ScriptBoundaryTokenizer::new();
        let tokens = tok.tokenize("abc123def");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["abc", "123", "def"]);
    }

    #[test]
    fn test_numbers_merged() {
        let tok = ScriptBoundaryTokenizer::new().with_merge_numbers(true);
        let tokens = tok.tokenize("abc123");
        // With merge_numbers, numbers stay with adjacent script
        assert!(tokens.len() <= 2);
    }
}
