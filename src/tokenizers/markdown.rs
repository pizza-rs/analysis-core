use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Strips Markdown formatting and tokenizes the plain text content.
///
/// Handles:
/// - Headers (`#`, `##`, etc.) → just the text
/// - Bold/italic (`**bold**`, `*italic*`, `_italic_`) → just the word
/// - Links `[text](url)` → text and optionally url
/// - Code spans `` `code` `` → code content
/// - Code blocks (``` ... ```) → content
/// - Lists (`-`, `*`, `1.`) → just the text
/// - Blockquotes (`>`) → just the text
#[derive(Clone, Debug)]
pub struct MarkdownTokenizer {
    /// Include URL targets from links
    include_urls: bool,
    /// Include code span content
    include_code: bool,
}

impl MarkdownTokenizer {
    pub fn new() -> Self {
        Self {
            include_urls: false,
            include_code: true,
        }
    }

    pub fn with_include_urls(mut self, v: bool) -> Self {
        self.include_urls = v;
        self
    }

    pub fn with_include_code(mut self, v: bool) -> Self {
        self.include_code = v;
        self
    }

    /// Strip markdown formatting. Returns the plain text plus a parallel
    /// `offsets` vector: `offsets[i]` is the byte offset in the **original**
    /// `text` that the byte at position `i` in the returned string came from.
    /// `offsets.len() == stripped.len()`. This lets the tokenizer emit Token
    /// offsets that refer to the source `text`, preserving the offset contract
    /// (required for highlighting/snippets).
    fn strip_markdown(&self, text: &str) -> (String, Vec<u32>) {
        let mut result = String::with_capacity(text.len());
        let mut offsets: Vec<u32> = Vec::with_capacity(text.len());
        let mut chars = text.char_indices().peekable();

        // Helper: push a char along with its source byte offset, recording
        // one entry per UTF-8 byte of the emitted char.
        let push_char = |out: &mut String, offs: &mut Vec<u32>, src_off: usize, ch: char| {
            let before = out.len();
            out.push(ch);
            for _ in before..out.len() {
                offs.push(src_off as u32);
            }
        };

        while let Some((src_off, c)) = chars.next() {
            match c {
                // Headers: skip leading # and space
                '#' => {
                    while let Some(&(_, p)) = chars.peek() {
                        if p == '#' {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if let Some(&(_, ' ')) = chars.peek() {
                        chars.next();
                    }
                }
                // Bold/italic markers
                '*' | '_' => {
                    while let Some(&(_, p)) = chars.peek() {
                        if p == c {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                // Links: [text](url)
                '[' => {
                    let mut depth = 1;
                    while let Some((loff, lc)) = chars.next() {
                        if lc == '[' {
                            depth += 1;
                            push_char(&mut result, &mut offsets, loff, lc);
                        } else if lc == ']' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            push_char(&mut result, &mut offsets, loff, lc);
                        } else {
                            push_char(&mut result, &mut offsets, loff, lc);
                        }
                    }
                    // separator space mapped to position after the `]`
                    push_char(&mut result, &mut offsets, src_off, ' ');

                    // Check for (url)
                    if let Some(&(_, '(')) = chars.peek() {
                        chars.next(); // skip (
                        while let Some((uoff, uc)) = chars.next() {
                            if uc == ')' {
                                break;
                            }
                            if self.include_urls {
                                push_char(&mut result, &mut offsets, uoff, uc);
                            }
                        }
                        if self.include_urls {
                            push_char(&mut result, &mut offsets, src_off, ' ');
                        }
                    }
                }
                // Code span
                '`' => {
                    let mut backtick_count = 1;
                    while let Some(&(_, '`')) = chars.peek() {
                        chars.next();
                        backtick_count += 1;
                    }

                    if backtick_count >= 3 {
                        // Code block: skip to closing ```
                        while let Some((_, bc)) = chars.next() {
                            if bc == '\n' {
                                break;
                            }
                        }
                        let mut close_count = 0;
                        while let Some((boff, bc)) = chars.next() {
                            if bc == '`' {
                                close_count += 1;
                                if close_count >= 3 {
                                    break;
                                }
                            } else {
                                if close_count > 0 {
                                    for _ in 0..close_count {
                                        if self.include_code {
                                            push_char(&mut result, &mut offsets, boff, '`');
                                        }
                                    }
                                    close_count = 0;
                                }
                                if self.include_code {
                                    push_char(&mut result, &mut offsets, boff, bc);
                                }
                            }
                        }
                        if self.include_code {
                            push_char(&mut result, &mut offsets, src_off, ' ');
                        }
                    } else {
                        let mut end_count = 0;
                        while let Some((coff, cc)) = chars.next() {
                            if cc == '`' {
                                end_count += 1;
                                if end_count >= backtick_count {
                                    break;
                                }
                            } else {
                                if end_count > 0 {
                                    for _ in 0..end_count {
                                        if self.include_code {
                                            push_char(&mut result, &mut offsets, coff, '`');
                                        }
                                    }
                                    end_count = 0;
                                }
                                if self.include_code {
                                    push_char(&mut result, &mut offsets, coff, cc);
                                }
                            }
                        }
                        if self.include_code {
                            push_char(&mut result, &mut offsets, src_off, ' ');
                        }
                    }
                }
                // Blockquotes
                '>' => {
                    if let Some(&(_, ' ')) = chars.peek() {
                        chars.next();
                    }
                }
                // List markers at start of line
                '-' if result.ends_with('\n') || result.is_empty() => {
                    if let Some(&(_, ' ')) = chars.peek() {
                        chars.next();
                    }
                }
                // Regular character
                _ => {
                    push_char(&mut result, &mut offsets, src_off, c);
                }
            }
        }

        debug_assert_eq!(result.len(), offsets.len());
        (result, offsets)
    }
}

impl Default for MarkdownTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for MarkdownTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let (stripped, src_offsets) = self.strip_markdown(text);

        let mut tokens = Vec::new();
        let mut position = 0u32;
        let mut i = 0;

        while i < stripped.len() {
            let c = stripped[i..].chars().next().unwrap();
            if !c.is_alphanumeric() {
                i += c.len_utf8();
                continue;
            }

            let start = i;
            while i < stripped.len() {
                let nc = stripped[i..].chars().next().unwrap();
                if nc.is_alphanumeric() || nc == '\'' {
                    i += nc.len_utf8();
                } else {
                    break;
                }
            }

            let word = &stripped[start..i];
            // Map stripped offsets back to source `text` offsets so highlighting works.
            let src_start = src_offsets[start];
            // Find the last char of the word: its offset in `stripped` and its byte length.
            let (last_char_off, last_char) =
                word.char_indices().next_back().expect("word is non-empty");
            let last_src = src_offsets[start + last_char_off];
            let src_end = last_src + last_char.len_utf8() as u32;

            tokens.push(Token {
                term: Cow::Owned(word.to_string()),
                start_offset: src_start,
                end_offset: src_end,
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
    fn test_header_stripped() {
        let tok = MarkdownTokenizer::new();
        let tokens = tok.tokenize("## Hello World");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"Hello"));
        assert!(terms.contains(&"World"));
    }

    #[test]
    fn test_bold_italic() {
        let tok = MarkdownTokenizer::new();
        let tokens = tok.tokenize("**bold** and *italic*");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"bold"));
        assert!(terms.contains(&"italic"));
        assert!(terms.contains(&"and"));
    }

    #[test]
    fn test_link() {
        let tok = MarkdownTokenizer::new();
        let tokens = tok.tokenize("[click here](https://example.com)");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"click"));
        assert!(terms.contains(&"here"));
    }

    #[test]
    fn test_code_span() {
        let tok = MarkdownTokenizer::new();
        let tokens = tok.tokenize("use `println!` macro");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"println"));
        assert!(terms.contains(&"use"));
        assert!(terms.contains(&"macro"));
    }

    #[test]
    fn test_link_with_urls() {
        let tok = MarkdownTokenizer::new().with_include_urls(true);
        let tokens = tok.tokenize("[docs](https://docs.rs)");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"docs"));
    }

    #[test]
    fn test_offsets_refer_to_source() {
        // Offsets must index into the original markdown text, not the stripped form.
        let src = "## Hello **world**";
        let tok = MarkdownTokenizer::new();
        let tokens = tok.tokenize(src);
        for t in &tokens {
            let slice = &src[t.start_offset as usize..t.end_offset as usize];
            assert_eq!(slice, t.term.as_ref(), "offset slice must equal token term");
        }
    }
}
