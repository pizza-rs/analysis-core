use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Tokenizes phone numbers into components for flexible search.
///
/// Handles formats like:
/// - `+1-800-555-0123` → `+1`, `800`, `555`, `0123`, `8005550123`
/// - `(123) 456-7890` → `123`, `456`, `7890`, `1234567890`
/// - `+44 20 7946 0958` → `+44`, `20`, `7946`, `0958`
///
/// Emits the normalized (digits-only) form plus individual number groups.
#[derive(Clone, Debug)]
pub struct PhoneNumberTokenizer {
    /// Also emit the full original phone string
    preserve_original: bool,
    /// Emit a digits-only normalized form
    emit_normalized: bool,
}

impl PhoneNumberTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_original: true,
            emit_normalized: true,
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }

    pub fn with_emit_normalized(mut self, v: bool) -> Self {
        self.emit_normalized = v;
        self
    }

    /// A candidate looks like a phone number when:
    /// - it has 7..=15 digits (E.164 max is 15), and
    /// - it begins with `+` OR `(`, OR contains at least one of `-`, ` `, `.`,
    ///   OR is a pure digit string of 7..=15 digits.
    ///
    /// This rejects IPv4 (`192.168.1.100` — has dots but >15 digits is rare;
    /// we additionally require >=50% digits and no four-segment dotted form
    /// to be safe), dates (`2024-01-15` has only 8 digits but would match —
    /// we require either a leading `+`/`(` OR at least 8 digits without a
    /// `YYYY-MM-DD` shape).
    fn looks_like_phone(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        let digit_count = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
        if !(7..=15).contains(&digit_count) {
            return false;
        }
        let total_count = trimmed.chars().count();
        if (digit_count as f32) / (total_count as f32) <= 0.5 {
            return false;
        }

        // Reject ISO-like dates "YYYY-MM-DD" (8 digits, three groups by '-').
        if Self::looks_like_iso_date(trimmed) {
            return false;
        }
        // Reject IPv4 "a.b.c.d" (four dot-separated digit groups).
        if Self::looks_like_ipv4(trimmed) {
            return false;
        }

        let first = trimmed.chars().next().unwrap();
        let has_phone_marker = trimmed.contains('+')
            || trimmed.contains('(')
            || trimmed.contains('-')
            || trimmed.contains(' ')
            || trimmed.contains('.');

        // Plain digit run: only accept 10..=15 digits (typical phone lengths).
        if !has_phone_marker {
            return (10..=15).contains(&digit_count);
        }
        // Requires a phone-ish lead OR sufficient digits.
        first == '+' || first == '(' || digit_count >= 7
    }

    fn looks_like_iso_date(s: &str) -> bool {
        let bytes = s.as_bytes();
        if bytes.len() != 10 {
            return false;
        }
        bytes[4] == b'-'
            && bytes[7] == b'-'
            && bytes[..4].iter().all(|b| b.is_ascii_digit())
            && bytes[5..7].iter().all(|b| b.is_ascii_digit())
            && bytes[8..10].iter().all(|b| b.is_ascii_digit())
    }

    fn looks_like_ipv4(s: &str) -> bool {
        let parts: Vec<&str> = s.split('.').collect();
        parts.len() == 4
            && parts
                .iter()
                .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    }

    fn extract_digits(s: &str) -> String {
        s.chars().filter(|c| c.is_ascii_digit()).collect()
    }
}

impl Default for PhoneNumberTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for PhoneNumberTokenizer {
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

            // Detect phone number start: +, (, or digit
            if c == '+' || c == '(' || c.is_ascii_digit() {
                let start = i;
                // Consume phone-like characters: digits, +, -, (, ), space, .
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_ascii_digit()
                        || nc == '+'
                        || nc == '-'
                        || nc == '('
                        || nc == ')'
                        || nc == ' '
                        || nc == '.'
                    {
                        i += nc.len_utf8();
                    } else {
                        break;
                    }
                }

                // Trim trailing non-digit chars
                while i > start && !text[..i].ends_with(|c: char| c.is_ascii_digit()) {
                    i -= 1;
                }

                let candidate = &text[start..i];
                if Self::looks_like_phone(candidate) {
                    // It's phone-like
                    if self.preserve_original {
                        tokens.push(Token {
                            term: Cow::Borrowed(candidate),
                            start_offset: start as u32,
                            end_offset: i as u32,
                            position,
                        });
                    }

                    // Emit digit groups
                    let mut group_start = start;
                    let mut in_digits = false;
                    for (idx, ch) in candidate.char_indices() {
                        let abs_idx = start + idx;
                        if ch.is_ascii_digit() || ch == '+' {
                            if !in_digits {
                                group_start = abs_idx;
                                in_digits = true;
                            }
                        } else {
                            if in_digits {
                                let group = &text[group_start..abs_idx];
                                if !group.is_empty() {
                                    tokens.push(Token {
                                        term: Cow::Borrowed(group),
                                        start_offset: group_start as u32,
                                        end_offset: abs_idx as u32,
                                        position,
                                    });
                                }
                            }
                            in_digits = false;
                        }
                    }
                    // Last group
                    if in_digits && group_start < i {
                        let group = &text[group_start..i];
                        if !group.is_empty() {
                            tokens.push(Token {
                                term: Cow::Borrowed(group),
                                start_offset: group_start as u32,
                                end_offset: i as u32,
                                position,
                            });
                        }
                    }

                    // Normalized form
                    if self.emit_normalized {
                        let normalized = Self::extract_digits(candidate);
                        if normalized.len() >= 7 {
                            tokens.push(Token {
                                term: Cow::Owned(normalized),
                                start_offset: start as u32,
                                end_offset: i as u32,
                                position,
                            });
                        }
                    }

                    position += 1;
                } else {
                    // Not phone-like, emit as regular token
                    if start < i {
                        let word = candidate.trim();
                        if !word.is_empty() {
                            tokens.push(Token {
                                term: Cow::Borrowed(&text[start..i]),
                                start_offset: start as u32,
                                end_offset: i as u32,
                                position,
                            });
                            position += 1;
                        }
                    }
                }
                continue;
            }

            // Regular alphanumeric
            if c.is_alphanumeric() {
                let start = i;
                while i < text.len() {
                    let nc = text[i..].chars().next().unwrap();
                    if nc.is_alphanumeric() {
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

            i += c_len;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_phone() {
        let tok = PhoneNumberTokenizer::new();
        let tokens = tok.tokenize("+1-800-555-0123");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"+1-800-555-0123"));
        assert!(terms.contains(&"+1"));
        assert!(terms.contains(&"800"));
        assert!(terms.contains(&"555"));
        assert!(terms.contains(&"0123"));
        assert!(terms.contains(&"18005550123"));
    }

    #[test]
    fn test_parens_format() {
        let tok = PhoneNumberTokenizer::new();
        let tokens = tok.tokenize("(123) 456-7890");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"123"));
        assert!(terms.contains(&"456"));
        assert!(terms.contains(&"7890"));
        assert!(terms.contains(&"1234567890"));
    }

    #[test]
    fn test_non_phone() {
        let tok = PhoneNumberTokenizer::new();
        let tokens = tok.tokenize("hello world");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["hello", "world"]);
    }

    #[test]
    fn test_ipv4_not_phone() {
        let tok = PhoneNumberTokenizer::new();
        let tokens = tok.tokenize("192.168.1.100");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        // Must not produce a normalized phone term.
        assert!(!terms.iter().any(|t| *t == "1921681100"));
    }

    #[test]
    fn test_iso_date_not_phone() {
        let tok = PhoneNumberTokenizer::new();
        let tokens = tok.tokenize("2024-01-15");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(!terms.iter().any(|t| *t == "20240115"));
    }
}
