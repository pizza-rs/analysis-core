use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Word delimiter graph token filter.
///
/// Splits tokens into sub-words based on word boundaries:
/// - Transitions between letters and digits: "SD500" → "SD", "500"
/// - Case transitions (camelCase): "WiFi" → "Wi", "Fi"
/// - Non-alphanumeric delimiters: "wi-fi" → "wi", "fi"
///
/// Configuration:
/// - `split_on_case_change`: Split on camelCase (default: true)
/// - `split_on_numerics`: Split on letter-digit boundaries (default: true)
/// - `generate_word_parts`: Emit alphabetic parts (default: true)
/// - `generate_number_parts`: Emit numeric parts (default: true)
/// - `concatenate_words`: Concatenate all word parts (default: false)
/// - `concatenate_numbers`: Concatenate all number parts (default: false)
/// - `concatenate_all`: Concatenate all parts (default: false)
/// - `preserve_original`: Keep original token (default: false)
/// - `stem_english_possessive`: Remove trailing 's (default: true)
#[derive(Clone, Debug)]
pub struct WordDelimiterGraphTokenFilter {
    pub split_on_case_change: bool,
    pub split_on_numerics: bool,
    pub generate_word_parts: bool,
    pub generate_number_parts: bool,
    pub concatenate_words: bool,
    pub concatenate_numbers: bool,
    pub concatenate_all: bool,
    pub preserve_original: bool,
    pub stem_english_possessive: bool,
}

impl Default for WordDelimiterGraphTokenFilter {
    fn default() -> Self {
        Self {
            split_on_case_change: true,
            split_on_numerics: true,
            generate_word_parts: true,
            generate_number_parts: true,
            concatenate_words: false,
            concatenate_numbers: false,
            concatenate_all: false,
            preserve_original: false,
            stem_english_possessive: true,
        }
    }
}

impl WordDelimiterGraphTokenFilter {
    pub fn new() -> Self {
        Self::default()
    }

    fn split_token(&self, text: &str) -> Vec<Part> {
        let mut parts = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        if len == 0 {
            return parts;
        }

        let mut start = 0;
        let mut i = 1;

        while i <= len {
            let should_split = if i == len {
                true
            } else {
                self.is_boundary(chars[i - 1], chars[i])
            };

            if should_split {
                let part_str: String = chars[start..i].iter().collect();
                let trimmed = part_str.trim_matches(|c: char| !c.is_alphanumeric());
                if !trimmed.is_empty() {
                    let part_type = classify_chars(trimmed);
                    parts.push(Part {
                        text: trimmed.to_owned(),
                        part_type,
                    });
                }
                // Skip non-alphanumeric delimiters
                while i < len && !chars[i].is_alphanumeric() {
                    i += 1;
                }
                start = i;
            }
            i += 1;
        }

        parts
    }

    fn is_boundary(&self, prev: char, curr: char) -> bool {
        // Non-alphanumeric characters are always boundaries
        if !prev.is_alphanumeric() || !curr.is_alphanumeric() {
            return true;
        }

        // Numeric/alpha transitions
        if self.split_on_numerics {
            if prev.is_alphabetic() && curr.is_numeric() {
                return true;
            }
            if prev.is_numeric() && curr.is_alphabetic() {
                return true;
            }
        }

        // Case change transitions
        if self.split_on_case_change {
            if prev.is_lowercase() && curr.is_uppercase() {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PartType {
    Word,
    Number,
}

#[derive(Debug, Clone)]
struct Part {
    text: String,
    part_type: PartType,
}

fn classify_chars(s: &str) -> PartType {
    if s.chars().all(|c| c.is_numeric()) {
        PartType::Number
    } else {
        PartType::Word
    }
}

impl TokenFilter for WordDelimiterGraphTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let mut text = token.term.as_ref().to_owned();

        // Stem English possessive
        if self.stem_english_possessive {
            // U+2019 (right single quotation mark) is 3 bytes in UTF-8, so we
            // must trim by the actual byte length of the apostrophe + "s",
            // not by a fixed `2`. Truncating mid-codepoint would panic.
            if text.ends_with("\u{2019}s") {
                let cut = '\u{2019}'.len_utf8() + 1;
                let new_len = text.len() - cut;
                text.truncate(new_len);
            } else if text.ends_with("'s") {
                let new_len = text.len() - 2;
                text.truncate(new_len);
            }
        }

        let parts = self.split_token(&text);

        if parts.len() <= 1 && !self.preserve_original {
            // Single part or no split needed - update in place if possessive was stripped
            if token.term.as_ref() != text {
                token.term = Cow::Owned(text);
            }
            return (false, None);
        }

        if parts.is_empty() {
            return (false, None);
        }

        let mut output_tokens: Vec<Token<'a>> = Vec::new();

        // Emit individual parts
        for part in &parts {
            let should_emit = match part.part_type {
                PartType::Word => self.generate_word_parts,
                PartType::Number => self.generate_number_parts,
            };
            if should_emit {
                output_tokens.push(Token {
                    term: Cow::Owned(part.text.clone()),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        // Concatenate words
        if self.concatenate_words {
            let word_parts: String = parts
                .iter()
                .filter(|p| p.part_type == PartType::Word)
                .map(|p| p.text.as_str())
                .collect();
            if !word_parts.is_empty()
                && parts
                    .iter()
                    .filter(|p| p.part_type == PartType::Word)
                    .count()
                    > 1
            {
                output_tokens.push(Token {
                    term: Cow::Owned(word_parts),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        // Concatenate numbers
        if self.concatenate_numbers {
            let num_parts: String = parts
                .iter()
                .filter(|p| p.part_type == PartType::Number)
                .map(|p| p.text.as_str())
                .collect();
            if !num_parts.is_empty()
                && parts
                    .iter()
                    .filter(|p| p.part_type == PartType::Number)
                    .count()
                    > 1
            {
                output_tokens.push(Token {
                    term: Cow::Owned(num_parts),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                });
            }
        }

        // Concatenate all
        if self.concatenate_all && parts.len() > 1 {
            let all_parts: String = parts.iter().map(|p| p.text.as_str()).collect();
            output_tokens.push(Token {
                term: Cow::Owned(all_parts),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            });
        }

        if output_tokens.is_empty() {
            return (false, None);
        }

        if self.preserve_original {
            // Keep original, emit parts as extra
            (false, Some(output_tokens))
        } else {
            // Replace original with first part, rest are extra
            let first = output_tokens.remove(0);
            token.term = first.term;
            if output_tokens.is_empty() {
                (false, None)
            } else {
                (false, Some(output_tokens))
            }
        }
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

    fn filter_and_collect(filter: &WordDelimiterGraphTokenFilter, text: &str) -> Vec<String> {
        let mut token = make_token(text);
        let (remove, extra) = filter.filter(&mut token);
        let mut result = Vec::new();
        if !remove {
            result.push(token.term.into_owned());
        }
        if let Some(extras) = extra {
            for t in extras {
                result.push(t.term.into_owned());
            }
        }
        result
    }

    #[test]
    fn test_camel_case() {
        let filter = WordDelimiterGraphTokenFilter::new();
        let terms = filter_and_collect(&filter, "camelCase");
        assert!(terms.contains(&"camel".to_owned()));
        assert!(terms.contains(&"Case".to_owned()));
    }

    #[test]
    fn test_numeric_split() {
        let filter = WordDelimiterGraphTokenFilter::new();
        let terms = filter_and_collect(&filter, "SD500");
        assert!(terms.contains(&"SD".to_owned()));
        assert!(terms.contains(&"500".to_owned()));
    }

    #[test]
    fn test_delimiter_split() {
        let filter = WordDelimiterGraphTokenFilter::new();
        let terms = filter_and_collect(&filter, "wi-fi");
        assert!(terms.contains(&"wi".to_owned()));
        assert!(terms.contains(&"fi".to_owned()));
    }

    #[test]
    fn test_possessive() {
        let filter = WordDelimiterGraphTokenFilter::new();
        let mut token = make_token("O'Neil's");
        filter.filter(&mut token);
        // Should strip possessive
        let text = token.term.as_ref().to_owned();
        assert!(!text.contains("'s"));
    }

    #[test]
    fn test_preserve_original() {
        let mut filter = WordDelimiterGraphTokenFilter::new();
        filter.preserve_original = true;
        let mut token = make_token("camelCase");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove);
        assert_eq!(token.term.as_ref(), "camelCase"); // original preserved
        let extra = extra.unwrap();
        assert!(extra.iter().any(|t| t.term.as_ref() == "camel"));
        assert!(extra.iter().any(|t| t.term.as_ref() == "Case"));
    }

    #[test]
    fn test_concatenate_words() {
        let mut filter = WordDelimiterGraphTokenFilter::new();
        filter.concatenate_words = true;
        let terms = filter_and_collect(&filter, "wi-fi");
        assert!(terms.contains(&"wifi".to_owned()));
    }

    #[test]
    fn test_curly_apostrophe_possessive_no_panic() {
        // U+2019 is 3 bytes in UTF-8; trimming by a fixed 2 bytes used to
        // panic with "byte index N is not a char boundary".
        let filter = WordDelimiterGraphTokenFilter::new();
        let mut token = make_token("running\u{2019}s");
        let (_remove, _extra) = filter.filter(&mut token);
        let text = token.term.as_ref();
        assert!(!text.contains('\u{2019}'));
        assert!(!text.ends_with('s') || text == "running");
    }
}
