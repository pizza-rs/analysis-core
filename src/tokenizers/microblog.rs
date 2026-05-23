use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Tokenizes social media / microblog text, preserving:
/// - @mentions as tokens
/// - #hashtags as tokens
/// - URLs as tokens
/// - Emojis as tokens
/// - Regular words
///
/// Useful for Twitter/X, Mastodon, or similar content.
#[derive(Clone, Debug)]
pub struct MicroBlogTokenizer {
    /// Strip the `@` prefix from mentions
    strip_mention_prefix: bool,
    /// Strip the `#` prefix from hashtags
    strip_hashtag_prefix: bool,
}

impl MicroBlogTokenizer {
    pub fn new() -> Self {
        Self {
            strip_mention_prefix: false,
            strip_hashtag_prefix: false,
        }
    }

    pub fn with_strip_prefixes(mut self, strip: bool) -> Self {
        self.strip_mention_prefix = strip;
        self.strip_hashtag_prefix = strip;
        self
    }

    fn is_url_start(text: &str) -> bool {
        text.starts_with("http://") || text.starts_with("https://") || text.starts_with("www.")
    }

    /// Returns true for characters that can start an emoji sequence.
    /// Mirrors the (narrower) ranges used by [`super::emoji::EmojiTokenizer`]
    /// so we don't mis-classify CJK / Arabic / generic punctuation as emoji.
    fn is_emoji_start(c: char) -> bool {
        let cp = c as u32;
        matches!(cp,
            0x1F600..=0x1F64F | // Emoticons
            0x1F300..=0x1F5FF | // Misc Symbols and Pictographs
            0x1F680..=0x1F6FF | // Transport and Map
            0x1F1E0..=0x1F1FF | // Flags (Regional Indicators)
            0x2600..=0x26FF   | // Misc symbols
            0x2700..=0x27BF   | // Dingbats
            0x1F900..=0x1F9FF | // Supplemental Symbols
            0x1FA00..=0x1FA6F | // Chess Symbols
            0x1FA70..=0x1FAFF | // Symbols and Pictographs Extended-A
            0x231A..=0x231B   |
            0x23E9..=0x23F3   |
            0x23F8..=0x23FA   |
            0x2B05..=0x2B07   |
            0x2B1B..=0x2B1C   |
            0x2B50            |
            0x2B55
        )
    }
}

impl Default for MicroBlogTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for MicroBlogTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < text.len() {
            let c = text[i..].chars().next().unwrap();

            // Skip whitespace
            if c.is_whitespace() {
                i += c.len_utf8();
                continue;
            }

            // URLs
            if Self::is_url_start(&text[i..]) {
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_whitespace() {
                        break;
                    }
                    i += nc.len_utf8();
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

            // @mentions
            if c == '@' {
                let start = i;
                i += 1;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() || nc == '_' || nc == '.' {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                if i > start + 1 {
                    let token_start = if self.strip_mention_prefix {
                        start + 1
                    } else {
                        start
                    };
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[token_start..i]),
                        start_offset: token_start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
                continue;
            }

            // #hashtags
            if c == '#' {
                let start = i;
                i += 1;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() || nc == '_' {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                if i > start + 1 {
                    let token_start = if self.strip_hashtag_prefix {
                        start + 1
                    } else {
                        start
                    };
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[token_start..i]),
                        start_offset: token_start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
                continue;
            }

            // Emoji detection using precise Unicode ranges (not "any non-ASCII non-letter").
            if Self::is_emoji_start(c) {
                let start = i;
                i += c.len_utf8();
                // Consume combining characters and variation selectors
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc == '\u{200D}'
                        || nc == '\u{FE0F}'
                        || nc == '\u{FE0E}'
                        || ('\u{1F3FB}'..='\u{1F3FF}').contains(&nc)
                    {
                        i += nc.len_utf8();
                        // Consume the next char after ZWJ
                        if nc == '\u{200D}' && i < text.len() {
                            let next = text[i..].chars().next().unwrap();
                            i += next.len_utf8();
                        }
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

            // Regular words
            if c.is_alphanumeric() || c == '\'' {
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() || nc == '\'' || nc == '-' {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }
                // Trim trailing punctuation
                while i > start && matches!(text.as_bytes()[i - 1], b'\'' | b'-') {
                    i -= 1;
                }
                if i > start {
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[start..i]),
                        start_offset: start as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
                continue;
            }

            // Skip other punctuation
            i += c.len_utf8();
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microblog_all_types() {
        let tok = MicroBlogTokenizer::new();
        let tokens = tok.tokenize("Hello @user check #rust https://example.com");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"Hello"));
        assert!(terms.contains(&"@user"));
        assert!(terms.contains(&"#rust"));
        assert!(terms.contains(&"https://example.com"));
    }

    #[test]
    fn test_strip_prefixes() {
        let tok = MicroBlogTokenizer::new().with_strip_prefixes(true);
        let tokens = tok.tokenize("@alice #coding");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["alice", "coding"]);
    }

    #[test]
    fn test_regular_text() {
        let tok = MicroBlogTokenizer::new();
        let tokens = tok.tokenize("just a normal tweet");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["just", "a", "normal", "tweet"]);
    }

    #[test]
    fn test_cjk_punctuation_not_emoji() {
        // Chinese commas/periods used to be wrongly tokenized as emoji.
        let tok = MicroBlogTokenizer::new();
        let tokens = tok.tokenize("你好，世界。");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // Only the CJK words should be present — no punctuation tokens.
        assert!(!terms.iter().any(|t| *t == "，" || *t == "。"));
        assert!(terms.contains(&"你好"));
        assert!(terms.contains(&"世界"));
    }

    #[test]
    fn test_real_emoji_still_works() {
        let tok = MicroBlogTokenizer::new();
        let tokens = tok.tokenize("hi 👋 world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"👋"));
    }
}
