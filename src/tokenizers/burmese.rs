use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Burmese (Myanmar) tokenizer.
///
/// Segments Myanmar script text at syllable boundaries using Unicode
/// code point ranges and the "killer" character (virama).
/// Non-Myanmar text is split at whitespace/punctuation boundaries.
///
/// Equivalent to Lucene's MyanmarAnalyzer tokenizer component.
#[derive(Clone, Debug, Default)]
pub struct BurmeseTokenizer;

impl BurmeseTokenizer {
    pub fn new() -> Self {
        Self
    }
}

impl Tokenizer for BurmeseTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position: u32 = 0;

        if text.is_empty() {
            return tokens;
        }

        let mut start = 0;
        let mut in_myanmar = false;
        let mut last_syllable_end = 0;

        for (i, c) in text.char_indices() {
            let is_myanmar = is_myanmar_char(c);
            let is_whitespace = c.is_whitespace();
            let is_punct = c.is_ascii_punctuation() || is_myanmar_punctuation(c);

            if i == 0 {
                in_myanmar = is_myanmar;
                start = 0;
                last_syllable_end = 0;
                if is_whitespace || is_punct {
                    start = i + c.len_utf8();
                }
                continue;
            }

            if is_whitespace || is_punct {
                // Emit token for accumulated text
                if start < i {
                    let segment = &text[start..i];
                    if in_myanmar {
                        emit_myanmar_syllables(segment, start as u32, &mut tokens, &mut position);
                    } else {
                        tokens.push(Token {
                            term: Cow::Borrowed(segment),
                            start_offset: start as u32,
                            end_offset: i as u32,
                            position,
                        });
                        position += 1;
                    }
                }
                start = i + c.len_utf8();
                last_syllable_end = start;
                continue;
            }

            // Script boundary
            if is_myanmar != in_myanmar {
                if start < i {
                    let segment = &text[start..i];
                    if in_myanmar {
                        emit_myanmar_syllables(segment, start as u32, &mut tokens, &mut position);
                    } else {
                        tokens.push(Token {
                            term: Cow::Borrowed(segment),
                            start_offset: start as u32,
                            end_offset: i as u32,
                            position,
                        });
                        position += 1;
                    }
                }
                start = i;
                last_syllable_end = i;
                in_myanmar = is_myanmar;
            }
        }

        // Emit remaining text
        let text_len = text.len();
        if start < text_len {
            let segment = &text[start..text_len];
            if in_myanmar {
                emit_myanmar_syllables(segment, start as u32, &mut tokens, &mut position);
            } else if !segment.trim().is_empty() {
                tokens.push(Token {
                    term: Cow::Borrowed(segment),
                    start_offset: start as u32,
                    end_offset: text_len as u32,
                    position,
                });
            }
        }

        let _ = last_syllable_end;
        tokens
    }
}

/// Emit Myanmar syllables by breaking at consonant boundaries.
/// Myanmar syllables start with consonants (U+1000-U+1021).
fn emit_myanmar_syllables<'a>(
    text: &'a str,
    base_offset: u32,
    tokens: &mut Vec<Token<'a>>,
    position: &mut u32,
) {
    if text.is_empty() {
        return;
    }

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut syllable_start = 0;
    let mut i = 0;

    while i < chars.len() {
        let (byte_pos, c) = chars[i];

        // A new syllable starts at a consonant, but not if preceded by virama
        if i > 0 && is_myanmar_consonant(c) {
            // Check if previous char is virama (U+1039) — stacked consonant
            let (_, prev_c) = chars[i - 1];
            if prev_c != '\u{1039}' && prev_c != '\u{103A}' {
                // Emit previous syllable
                if syllable_start < byte_pos {
                    let segment = &text[syllable_start..byte_pos];
                    if !segment.trim().is_empty() {
                        tokens.push(Token {
                            term: Cow::Borrowed(segment),
                            start_offset: base_offset + syllable_start as u32,
                            end_offset: base_offset + byte_pos as u32,
                            position: *position,
                        });
                        *position += 1;
                    }
                }
                syllable_start = byte_pos;
            }
        }
        i += 1;
    }

    // Emit remaining syllable
    if syllable_start < text.len() {
        let segment = &text[syllable_start..];
        if !segment.trim().is_empty() {
            tokens.push(Token {
                term: Cow::Borrowed(segment),
                start_offset: base_offset + syllable_start as u32,
                end_offset: base_offset + text.len() as u32,
                position: *position,
            });
            *position += 1;
        }
    }
}

/// Check if a character is in the Myanmar Unicode block.
fn is_myanmar_char(c: char) -> bool {
    let cp = c as u32;
    // Myanmar: U+1000..U+109F
    // Myanmar Extended-A: U+AA60..U+AA7F
    // Myanmar Extended-B: U+A9E0..U+A9FF
    (0x1000..=0x109F).contains(&cp)
        || (0xAA60..=0xAA7F).contains(&cp)
        || (0xA9E0..=0xA9FF).contains(&cp)
}

/// Check if a character is a Myanmar consonant (U+1000-U+1021).
fn is_myanmar_consonant(c: char) -> bool {
    let cp = c as u32;
    (0x1000..=0x1021).contains(&cp)
}

/// Check if a character is Myanmar punctuation.
fn is_myanmar_punctuation(c: char) -> bool {
    c == '\u{104A}' || c == '\u{104B}'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_myanmar_char() {
        assert!(is_myanmar_char('\u{1000}')); // ka
        assert!(is_myanmar_char('\u{1019}')); // ma
        assert!(!is_myanmar_char('a'));
        assert!(!is_myanmar_char('中'));
    }

    #[test]
    fn test_ascii_text() {
        let tokenizer = BurmeseTokenizer::new();
        let tokens = tokenizer.tokenize("hello world");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term.as_ref(), "hello");
        assert_eq!(tokens[1].term.as_ref(), "world");
    }

    #[test]
    fn test_empty_input() {
        let tokenizer = BurmeseTokenizer::new();
        let tokens = tokenizer.tokenize("");
        assert!(tokens.is_empty());
    }
}
