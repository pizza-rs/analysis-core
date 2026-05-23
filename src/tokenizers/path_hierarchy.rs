use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Splits filesystem paths into hierarchical tokens.
///
/// Example: `/a/b/c` → `[/a, /a/b, /a/b/c]`
#[derive(Clone, Debug)]
pub struct PathHierarchyTokenizer {
    pub separator: char,
    pub replacement: char,
    pub skip: usize,
    pub reverse: bool,
}

impl PathHierarchyTokenizer {
    pub fn new() -> Self {
        Self {
            separator: '/',
            replacement: '/',
            skip: 0,
            reverse: false,
        }
    }

    pub fn with_separator(mut self, sep: char) -> Self {
        self.separator = sep;
        self
    }

    pub fn with_replacement(mut self, rep: char) -> Self {
        self.replacement = rep;
        self
    }

    pub fn with_skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    pub fn reversed(mut self) -> Self {
        self.reverse = true;
        self
    }
}

impl Default for PathHierarchyTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for PathHierarchyTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        if self.reverse {
            // Reverse mode: /a/b/c → [/a/b/c, /a/b, /a]
            let parts: Vec<&str> = text.split(self.separator).collect();
            let total = parts.len();
            let start_at = if text.starts_with(self.separator) {
                1
            } else {
                0
            };

            for i in (start_at + self.skip..total).rev() {
                let joined: String = if self.separator == self.replacement {
                    parts[..=i].join(&self.separator.to_string())
                } else {
                    parts[..=i].join(&self.replacement.to_string())
                };
                let pos = (total - 1 - i) as u32;
                tokens.push(Token {
                    term: Cow::Owned(joined),
                    start_offset: 0,
                    end_offset: text.len() as u32,
                    position: pos,
                });
            }
        } else {
            // Normal mode: /a/b/c → [/a, /a/b, /a/b/c]
            let mut position = 0u32;
            let mut skipped = 0usize;
            let mut last_end = 0;

            // Handle leading separator
            if text.starts_with(self.separator) {
                last_end = self.separator.len_utf8();
            }

            for (i, ch) in text[last_end..].char_indices() {
                if ch == self.separator {
                    let end = last_end + i;
                    if skipped < self.skip {
                        skipped += 1;
                    } else {
                        let segment = &text[..end];
                        if self.separator == self.replacement {
                            tokens.push(Token::new(segment, 0, end as u32, position));
                        } else {
                            let replaced =
                                segment.replace(self.separator, &self.replacement.to_string());
                            tokens.push(Token {
                                term: Cow::Owned(replaced),
                                start_offset: 0,
                                end_offset: end as u32,
                                position,
                            });
                        }
                        position += 1;
                    }
                }
            }

            // Emit the full path
            if skipped >= self.skip {
                if self.separator == self.replacement {
                    tokens.push(Token::new(text, 0, text.len() as u32, position));
                } else {
                    let replaced = text.replace(self.separator, &self.replacement.to_string());
                    tokens.push(Token {
                        term: Cow::Owned(replaced),
                        start_offset: 0,
                        end_offset: text.len() as u32,
                        position,
                    });
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_hierarchy() {
        let t = PathHierarchyTokenizer::new();
        let tokens = t.tokenize("/a/b/c");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["/a", "/a/b", "/a/b/c"]);
    }

    #[test]
    fn test_path_hierarchy_skip() {
        let t = PathHierarchyTokenizer::new().with_skip(1);
        let tokens = t.tokenize("/a/b/c");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["/a/b", "/a/b/c"]);
    }
}
