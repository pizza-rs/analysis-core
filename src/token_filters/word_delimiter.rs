use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Configuration for WordDelimiterTokenFilter behavior.
#[derive(Clone, Debug)]
pub struct WordDelimiterConfig {
    /// Split on letter-number transitions (e.g. "Wi-Fi" → "Wi", "Fi")
    pub split_on_case_change: bool,
    /// Split on numeric-alpha transitions (e.g. "SD500" → "SD", "500")
    pub split_on_numerics: bool,
    /// Generate word parts (subwords)
    pub generate_word_parts: bool,
    /// Generate number parts (subnumbers)
    pub generate_number_parts: bool,
    /// Concatenate all word parts ("Wi-Fi" also produces "WiFi")
    pub catenate_words: bool,
    /// Concatenate all number parts ("500-42" also produces "50042")
    pub catenate_numbers: bool,
    /// Preserve original token as well
    pub preserve_original: bool,
}

impl Default for WordDelimiterConfig {
    fn default() -> Self {
        Self {
            split_on_case_change: true,
            split_on_numerics: true,
            generate_word_parts: true,
            generate_number_parts: true,
            catenate_words: false,
            catenate_numbers: false,
            preserve_original: false,
        }
    }
}

/// Splits tokens at word delimiters (non-alphanumeric chars, camelCase boundaries, letter-number transitions).
#[derive(Clone, Debug)]
pub struct WordDelimiterTokenFilter {
    config: WordDelimiterConfig,
}

impl WordDelimiterTokenFilter {
    pub fn new(config: WordDelimiterConfig) -> Self {
        Self { config }
    }
}

impl Default for WordDelimiterTokenFilter {
    fn default() -> Self {
        Self {
            config: WordDelimiterConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CharType {
    Upper,
    Lower,
    Digit,
    Delimiter,
}

fn char_type(c: char) -> CharType {
    if c.is_ascii_digit() || c.is_numeric() {
        CharType::Digit
    } else if c.is_uppercase() {
        CharType::Upper
    } else if c.is_lowercase() {
        CharType::Lower
    } else {
        CharType::Delimiter
    }
}

/// Split the term into subword segments based on delimiters and transitions.
fn split_word(term: &str, split_on_case: bool, split_on_numerics: bool) -> Vec<(usize, usize)> {
    let chars: Vec<(usize, char)> = term.char_indices().collect();
    if chars.is_empty() {
        return Vec::new();
    }

    let mut segments: Vec<(usize, usize)> = Vec::new();
    let mut seg_start = 0;

    for i in 1..chars.len() {
        let prev_type = char_type(chars[i - 1].1);
        let curr_type = char_type(chars[i].1);

        let should_split = match (prev_type, curr_type) {
            (CharType::Delimiter, _) | (_, CharType::Delimiter) => true,
            (CharType::Lower, CharType::Upper) if split_on_case => true,
            (CharType::Upper, CharType::Upper) => false, // consecutive uppers stay together unless followed by lower
            (CharType::Digit, CharType::Upper) | (CharType::Digit, CharType::Lower)
                if split_on_numerics =>
            {
                true
            }
            (CharType::Upper, CharType::Digit) | (CharType::Lower, CharType::Digit)
                if split_on_numerics =>
            {
                true
            }
            _ => false,
        };

        if should_split {
            // Only add non-delimiter segments
            let seg_text = &term[chars[seg_start].0..chars[i - 1].0 + chars[i - 1].1.len_utf8()];
            if seg_text
                .chars()
                .any(|c| char_type(c) != CharType::Delimiter)
            {
                segments.push((
                    chars[seg_start].0,
                    chars[i - 1].0 + chars[i - 1].1.len_utf8(),
                ));
            }
            seg_start = i;
        }
    }

    // Last segment
    let last = chars.last().unwrap();
    let seg_text = &term[chars[seg_start].0..last.0 + last.1.len_utf8()];
    if seg_text
        .chars()
        .any(|c| char_type(c) != CharType::Delimiter)
    {
        segments.push((chars[seg_start].0, last.0 + last.1.len_utf8()));
    }

    segments
}

impl TokenFilter for WordDelimiterTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();
        let segments = split_word(
            term,
            self.config.split_on_case_change,
            self.config.split_on_numerics,
        );

        // If no splitting happened, keep original
        if segments.len() <= 1 && !self.config.preserve_original {
            return (false, None);
        }
        if segments.is_empty() {
            return (true, None); // all delimiters, remove
        }

        let mut extra_tokens: Vec<Token<'a>> = Vec::new();
        let base_offset = token.start_offset;
        let base_pos = token.position;

        let mut word_parts = String::new();
        let mut number_parts = String::new();

        for (idx, &(start, end)) in segments.iter().enumerate() {
            let part = &term[start..end];
            let is_numeric = part.chars().all(|c| c.is_ascii_digit() || c.is_numeric());

            if is_numeric {
                if self.config.generate_number_parts {
                    extra_tokens.push(Token {
                        term: Cow::Owned(part.to_string()),
                        start_offset: base_offset + start as u32,
                        end_offset: base_offset + end as u32,
                        position: base_pos + idx as u32,
                    });
                }
                if self.config.catenate_numbers {
                    number_parts.push_str(part);
                }
            } else {
                if self.config.generate_word_parts {
                    extra_tokens.push(Token {
                        term: Cow::Owned(part.to_string()),
                        start_offset: base_offset + start as u32,
                        end_offset: base_offset + end as u32,
                        position: base_pos + idx as u32,
                    });
                }
                if self.config.catenate_words {
                    word_parts.push_str(part);
                }
            }
        }

        // Add catenated forms
        if self.config.catenate_words && !word_parts.is_empty() && segments.len() > 1 {
            extra_tokens.push(Token {
                term: Cow::Owned(word_parts),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: base_pos,
            });
        }
        if self.config.catenate_numbers && !number_parts.is_empty() && segments.len() > 1 {
            extra_tokens.push(Token {
                term: Cow::Owned(number_parts),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: base_pos,
            });
        }

        if self.config.preserve_original {
            // Keep original token, emit extras as injections
            (
                false,
                if extra_tokens.is_empty() {
                    None
                } else {
                    Some(extra_tokens)
                },
            )
        } else {
            // Replace original with first part, inject rest
            if let Some(first) = extra_tokens.first() {
                token.term = first.term.clone();
                token.start_offset = first.start_offset;
                token.end_offset = first.end_offset;
                let rest = extra_tokens[1..].to_vec();
                (false, if rest.is_empty() { None } else { Some(rest) })
            } else {
                (true, None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_on_delimiter() {
        let f = WordDelimiterTokenFilter::default();
        let mut token = Token::new("wi-fi", 0, 5, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term, "wi");
        let extra = extra.unwrap();
        assert_eq!(extra[0].term, "fi");
    }

    #[test]
    fn test_split_camel_case() {
        let f = WordDelimiterTokenFilter::default();
        let mut token = Token::new("camelCase", 0, 9, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term, "camel");
        let extra = extra.unwrap();
        assert_eq!(extra[0].term, "Case");
    }

    #[test]
    fn test_split_numerics() {
        let f = WordDelimiterTokenFilter::default();
        let mut token = Token::new("SD500", 0, 5, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term, "SD");
        let extra = extra.unwrap();
        assert_eq!(extra[0].term, "500");
    }

    #[test]
    fn test_no_split_needed() {
        let f = WordDelimiterTokenFilter::default();
        let mut token = Token::new("hello", 0, 5, 0);
        let (remove, extra) = f.filter(&mut token);
        assert!(!remove);
        assert!(extra.is_none());
    }

    #[test]
    fn test_catenate_words() {
        let mut cfg = WordDelimiterConfig::default();
        cfg.catenate_words = true;
        let f = WordDelimiterTokenFilter::new(cfg);
        let mut token = Token::new("wi-fi", 0, 5, 0);
        let (_, extra) = f.filter(&mut token);
        let extra = extra.unwrap();
        // Should have "fi" and "wifi" (catenated)
        assert!(extra.iter().any(|t| t.term == "wifi"));
    }
}
