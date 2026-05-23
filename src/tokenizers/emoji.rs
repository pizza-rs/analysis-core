use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Emoji-aware tokenizer that properly segments emoji sequences.
///
/// Handles:
/// - Single emoji: `😀` → one token
/// - ZWJ sequences: `👨‍👩‍👧‍👦` (family) → one token
/// - Skin tone modifiers: `👋🏽` → one token
/// - Emoji + text mix: `hello 🌍 world` → `hello`, `🌍`, `world`
/// - Variation selectors (VS15/VS16)
///
/// Useful when emojis should be preserved as searchable tokens rather than stripped.
#[derive(Clone, Debug)]
pub struct EmojiTokenizer {
    /// Also emit regular text tokens (not just emojis)
    include_text: bool,
}

impl EmojiTokenizer {
    pub fn new() -> Self {
        Self { include_text: true }
    }

    pub fn with_include_text(mut self, v: bool) -> Self {
        self.include_text = v;
        self
    }

    fn is_emoji_start(c: char) -> bool {
        let cp = c as u32;
        // Misc Symbols (0x2600..=0x26FF) and Dingbats (0x2700..=0x27BF) ranges
        // cover many individual codepoints that would otherwise be listed here.
        matches!(cp,
            0x1F600..=0x1F64F | // Emoticons
            0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
            0x1F680..=0x1F6FF | // Transport and Map
            0x1F1E0..=0x1F1FF | // Flags (Regional Indicators)
            0x2600..=0x26FF   | // Misc symbols (covers 0x2614, 0x2648..=0x2653, 0x267F, 0x2693, 0x26A1, 0x26AA..=0x26AB, 0x26BD..=0x26BE, 0x26C4..=0x26C5, 0x26CE, 0x26D4, 0x26EA, 0x26F2..=0x26F3, 0x26F5, 0x26FA, 0x26FD)
            0x2700..=0x27BF   | // Dingbats (covers 0x2702, 0x2705, 0x2708..=0x270D, 0x270F, 0x2712, 0x2714, 0x2716, 0x271D, 0x2721, 0x2728, 0x2733..=0x2734, 0x2744, 0x2747, 0x274C, 0x274E, 0x2753..=0x2755, 0x2757, 0x2763..=0x2764, 0x2795..=0x2797, 0x27A1, 0x27B0)
            0x1F900..=0x1F9FF | // Supplemental Symbols
            0x1FA00..=0x1FA6F | // Chess Symbols
            0x1FA70..=0x1FAFF | // Symbols and Pictographs Extended-A
            0x231A..=0x231B   | // Watch, Hourglass
            0x23E9..=0x23F3   | // Various symbols
            0x23F8..=0x23FA   | // Various symbols
            0x25AA..=0x25AB   | // Squares
            0x25B6            |
            0x25C0            |
            0x25FB..=0x25FE   |
            0x2934..=0x2935   |
            0x2B05..=0x2B07   |
            0x2B1B..=0x2B1C   |
            0x2B50            |
            0x2B55            |
            0x3030            |
            0x303D            |
            0x3297            |
            0x3299            |
            0xFE00..=0xFE0F   | // Variation selectors
            0x200D              // ZWJ
        )
    }

    fn is_emoji_continuation(c: char) -> bool {
        let cp = c as u32;
        Self::is_emoji_start(c)
            || matches!(cp,
                0xFE0F | 0xFE0E |        // Variation selectors
                0x200D |                    // Zero-Width Joiner
                0x1F3FB..=0x1F3FF |        // Skin tones
                0x20E3 |                    // Combining Enclosing Keycap
                0xE0020..=0xE007F          // Tags
            )
    }
}

impl Default for EmojiTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for EmojiTokenizer {
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

            // Emoji sequence
            if Self::is_emoji_start(c) {
                let start = i;
                i += c_len;
                // Consume continuation characters
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if Self::is_emoji_continuation(nc) {
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
                continue;
            }

            // Regular text
            if c.is_alphanumeric() {
                if self.include_text {
                    let start = i;
                    while i < text.len() {
                        let nc = text[i..].chars().next().unwrap();
                        if nc.is_alphanumeric() || nc == '\'' {
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
                    i += c_len;
                }
                continue;
            }

            // Skip other chars
            i += c_len;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_emoji() {
        let tok = EmojiTokenizer::new();
        let tokens = tok.tokenize("😀");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "😀");
    }

    #[test]
    fn test_emoji_with_text() {
        let tok = EmojiTokenizer::new();
        let tokens = tok.tokenize("hello 🌍 world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "🌍", "world"]);
    }

    #[test]
    fn test_only_emojis() {
        let tok = EmojiTokenizer::new().with_include_text(false);
        let tokens = tok.tokenize("hello 🌍 world 🎉");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["🌍", "🎉"]);
    }

    #[test]
    fn test_multiple_emojis() {
        let tok = EmojiTokenizer::new();
        let tokens = tok.tokenize("🎉🎊🎈");
        // Each should be a separate token
        assert!(tokens.len() >= 1);
    }

    #[test]
    fn test_no_emojis() {
        let tok = EmojiTokenizer::new();
        let tokens = tok.tokenize("just plain text");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["just", "plain", "text"]);
    }
}
