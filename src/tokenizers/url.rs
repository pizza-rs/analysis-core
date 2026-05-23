use alloc::borrow::Cow;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::Tokenizer;

/// Decomposes URLs into structured parts as individual tokens.
///
/// For input `https://www.example.com:8080/path/to/page?q=search&lang=en#section`:
/// Emits: scheme, host parts, port, path segments, query keys/values, fragment
#[derive(Clone, Debug)]
pub struct UrlTokenizer {
    /// Emit the full URL as the first token
    preserve_original: bool,
    /// Emit host segments (split on `.`)
    split_host: bool,
    /// Emit path segments (split on `/`)
    split_path: bool,
    /// Emit query parameter keys and values
    split_query: bool,
}

impl UrlTokenizer {
    pub fn new() -> Self {
        Self {
            preserve_original: false,
            split_host: true,
            split_path: true,
            split_query: true,
        }
    }

    pub fn with_preserve_original(mut self, v: bool) -> Self {
        self.preserve_original = v;
        self
    }
}

impl Default for UrlTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for UrlTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let mut position = 0u32;

        // Treat the entire input as a potential URL
        let url = text.trim();
        if url.is_empty() {
            return tokens;
        }

        let base = url.as_ptr() as usize - text.as_ptr() as usize;

        if self.preserve_original {
            tokens.push(Token {
                term: Cow::Borrowed(url),
                start_offset: base as u32,
                end_offset: (base + url.len()) as u32,
                position,
            });
            position += 1;
        }

        let mut rest = url;

        // Extract scheme
        if let Some(scheme_end) = rest.find("://") {
            let scheme = &rest[..scheme_end];
            let offset = base + (scheme.as_ptr() as usize - url.as_ptr() as usize);
            tokens.push(Token {
                term: Cow::Borrowed(scheme),
                start_offset: offset as u32,
                end_offset: (offset + scheme.len()) as u32,
                position,
            });
            position += 1;
            rest = &rest[scheme_end + 3..];
        }

        // Split off fragment
        let (before_fragment, fragment) = if let Some(hash_pos) = rest.find('#') {
            (&rest[..hash_pos], Some(&rest[hash_pos + 1..]))
        } else {
            (rest, None)
        };

        // Split off query
        let (before_query, query) = if let Some(q_pos) = before_fragment.find('?') {
            (&before_fragment[..q_pos], Some(&before_fragment[q_pos + 1..]))
        } else {
            (before_fragment, None)
        };

        // Split host and path
        let (host_port, path) = if let Some(slash_pos) = before_query.find('/') {
            (&before_query[..slash_pos], Some(&before_query[slash_pos + 1..]))
        } else {
            (before_query, None)
        };

        // Parse host:port
        let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
            let potential_port = &host_port[colon_pos + 1..];
            if potential_port.chars().all(|c| c.is_ascii_digit()) && !potential_port.is_empty() {
                (&host_port[..colon_pos], Some(potential_port))
            } else {
                (host_port, None)
            }
        } else {
            (host_port, None)
        };

        // Emit host parts
        if self.split_host {
            for part in host.split('.') {
                if !part.is_empty() {
                    let offset = base + (part.as_ptr() as usize - url.as_ptr() as usize);
                    tokens.push(Token {
                        term: Cow::Borrowed(part),
                        start_offset: offset as u32,
                        end_offset: (offset + part.len()) as u32,
                        position,
                    });
                    position += 1;
                }
            }
        } else if !host.is_empty() {
            let offset = base + (host.as_ptr() as usize - url.as_ptr() as usize);
            tokens.push(Token {
                term: Cow::Borrowed(host),
                start_offset: offset as u32,
                end_offset: (offset + host.len()) as u32,
                position,
            });
            position += 1;
        }

        // Emit port
        if let Some(port_str) = port {
            let offset = base + (port_str.as_ptr() as usize - url.as_ptr() as usize);
            tokens.push(Token {
                term: Cow::Borrowed(port_str),
                start_offset: offset as u32,
                end_offset: (offset + port_str.len()) as u32,
                position,
            });
            position += 1;
        }

        // Emit path segments
        if let Some(path_str) = path {
            if self.split_path {
                for seg in path_str.split('/') {
                    if !seg.is_empty() {
                        let offset = base + (seg.as_ptr() as usize - url.as_ptr() as usize);
                        tokens.push(Token {
                            term: Cow::Borrowed(seg),
                            start_offset: offset as u32,
                            end_offset: (offset + seg.len()) as u32,
                            position,
                        });
                        position += 1;
                    }
                }
            } else if !path_str.is_empty() {
                let offset = base + (path_str.as_ptr() as usize - url.as_ptr() as usize);
                tokens.push(Token {
                    term: Cow::Borrowed(path_str),
                    start_offset: offset as u32,
                    end_offset: (offset + path_str.len()) as u32,
                    position,
                });
                position += 1;
            }
        }

        // Emit query params
        if let Some(query_str) = query {
            if self.split_query {
                for pair in query_str.split('&') {
                    for part in pair.split('=') {
                        if !part.is_empty() {
                            let offset = base + (part.as_ptr() as usize - url.as_ptr() as usize);
                            tokens.push(Token {
                                term: Cow::Borrowed(part),
                                start_offset: offset as u32,
                                end_offset: (offset + part.len()) as u32,
                                position,
                            });
                            position += 1;
                        }
                    }
                }
            }
        }

        // Emit fragment
        if let Some(frag) = fragment {
            if !frag.is_empty() {
                let offset = base + (frag.as_ptr() as usize - url.as_ptr() as usize);
                tokens.push(Token {
                    term: Cow::Borrowed(frag),
                    start_offset: offset as u32,
                    end_offset: (offset + frag.len()) as u32,
                    position,
                });
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_url() {
        let tok = UrlTokenizer::new();
        let tokens = tok.tokenize("https://www.example.com:8080/path/to/page?q=test#top");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert!(terms.contains(&"https"));
        assert!(terms.contains(&"www"));
        assert!(terms.contains(&"example"));
        assert!(terms.contains(&"com"));
        assert!(terms.contains(&"8080"));
        assert!(terms.contains(&"path"));
        assert!(terms.contains(&"to"));
        assert!(terms.contains(&"page"));
        assert!(terms.contains(&"q"));
        assert!(terms.contains(&"test"));
        assert!(terms.contains(&"top"));
    }

    #[test]
    fn test_simple_url() {
        let tok = UrlTokenizer::new();
        let tokens = tok.tokenize("http://example.com");
        let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(terms, vec!["http", "example", "com"]);
    }
}
