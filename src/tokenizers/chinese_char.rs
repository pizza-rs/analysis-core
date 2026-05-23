use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Single-character tokenizer for CJK (Chinese/Japanese/Korean) text.
///
/// Emits each CJK character as an individual token while keeping
/// non-CJK runs (ASCII words, numbers) as whole tokens.
/// This is the simplest unigram approach for Chinese text without a dictionary.
#[derive(Clone, Debug)]
pub struct ChineseCharTokenizer;

impl ChineseCharTokenizer {
    pub fn new() -> Self {
        Self
    }

    fn is_cjk(c: char) -> bool {
        // NOTE: the `CJK Symbols and Punctuation` block (U+3000-U+303F) is
        // intentionally excluded — it contains ideographic punctuation
        // (`\u{3001}`, `\u{3002}`, `\u{300C}`, etc.) and the ideographic
        // space `\u{3000}` which must be treated as token *separators*, not
        // emitted as content tokens. Including the range caused
        // `\"你好\u{3002}再见\"` to tokenize as `['你','好','\u{3002}','再','见']`.
        matches!(c as u32,
            0x4E00..=0x9FFF |     // CJK Unified Ideographs
            0x3400..=0x4DBF |     // CJK Extension A
            0x20000..=0x2A6DF |   // CJK Extension B
            0x2A700..=0x2B73F |   // CJK Extension C
            0x2B740..=0x2B81F |   // CJK Extension D
            0x2B820..=0x2CEAF |   // CJK Extension E
            0x2CEB0..=0x2EBEF |   // CJK Extension F
            0xF900..=0xFAFF |     // CJK Compatibility Ideographs
            0x2F800..=0x2FA1F |   // CJK Compat Supplement
            0x3040..=0x309F |     // Hiragana
            0x30A0..=0x30FF |     // Katakana
            0x31F0..=0x31FF |     // Katakana Extensions
            0xAC00..=0xD7AF |     // Hangul Syllables
            0x1100..=0x11FF       // Hangul Jamo
        )
    }
}

impl Default for ChineseCharTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ChineseCharTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();
            let c_len = c.len_utf8();

            if c.is_whitespace() {
                i += c_len;
                continue;
            }

            if Self::is_cjk(c) {
                // Emit each CJK character as a separate token
                tokens.push(Token {
                    term: Cow::Borrowed(&text[i..i + c_len]),
                    start_offset: i as u32,
                    end_offset: (i + c_len) as u32,
                    position,
                });
                position += 1;
                i += c_len;
            } else if c.is_alphanumeric() {
                // Non-CJK alphanumeric: collect run
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() && !Self::is_cjk(nc) {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    term: Cow::Borrowed(&text[start..i]),
                    start_offset: start as u32,
                    end_offset: i as u32,
                    position,
                });
                position += 1;
            } else {
                // Skip punctuation and other non-token chars
                i += c_len;
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_chars() {
        let tok = ChineseCharTokenizer::new();
        let tokens = tok.tokenize("中华人民共和国");
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].term.as_ref(), "中");
        assert_eq!(tokens[1].term.as_ref(), "华");
        assert_eq!(tokens[6].term.as_ref(), "国");
    }

    #[test]
    fn test_mixed_cjk_ascii() {
        let tok = ChineseCharTokenizer::new();
        let tokens = tok.tokenize("hello世界test");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "世", "界", "test"]);
    }

    #[test]
    fn test_pure_ascii() {
        let tok = ChineseCharTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_japanese_hiragana() {
        let tok = ChineseCharTokenizer::new();
        let tokens = tok.tokenize("こんにちは");
        assert_eq!(tokens.len(), 5); // each hiragana is a token
    }

    #[test]
    fn test_skips_ideographic_punctuation() {
        // Regression: U+3001/U+3002 were previously matched by `is_cjk`
        // and emitted as content tokens.
        let tok = ChineseCharTokenizer::new();
        let tokens = tok.tokenize("你好。再见、朋友");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["你", "好", "再", "见", "朋", "友"]);
    }
}
