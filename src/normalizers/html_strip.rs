use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Normalizer;

/// Strips HTML tags and decodes common HTML entities from the input text.
///
/// Strips HTML tags from text, implemented
/// as a Pizza normalizer (pre-tokenization string transform).
#[derive(Clone, Debug)]
pub struct HtmlStripNormalizer {
    /// HTML tags that should NOT be stripped.
    pub escaped_tags: Vec<String>,
}

impl HtmlStripNormalizer {
    pub fn new() -> Self {
        Self {
            escaped_tags: Vec::new(),
        }
    }

    pub fn with_escaped_tags(mut self, tags: Vec<String>) -> Self {
        self.escaped_tags = tags;
        self
    }

    fn should_escape_tag(&self, tag_name: &str) -> bool {
        self.escaped_tags
            .iter()
            .any(|t| t.eq_ignore_ascii_case(tag_name))
    }
}

impl Default for HtmlStripNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Normalizer for HtmlStripNormalizer {
    fn normalize(&self, text: &mut String) {
        let input = text.as_bytes();
        let len = input.len();
        let mut result = String::with_capacity(len);
        let mut i = 0;

        while i < len {
            if input[i] == b'<' {
                // Potential HTML tag
                let tag_start = i;
                i += 1;

                // Skip optional '/'
                let is_closing = i < len && input[i] == b'/';
                if is_closing {
                    i += 1;
                }

                // Read tag name
                let name_start = i;
                while i < len && input[i] != b'>' && input[i] != b' ' && input[i] != b'/' {
                    i += 1;
                }
                let tag_name = &text[name_start..i];

                // Skip to end of tag
                while i < len && input[i] != b'>' {
                    i += 1;
                }
                if i < len {
                    i += 1; // skip '>'
                }

                if self.should_escape_tag(tag_name) {
                    // Keep the tag
                    result.push_str(&text[tag_start..i]);
                } else {
                    // Replace block tags with a space
                    if is_block_tag(tag_name) && !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                }
            } else if input[i] == b'&' {
                // HTML entity. Supports:
                //   * named entities listed in `decode_entity`
                //   * decimal numeric entities: &#NNN;
                //   * hex numeric entities:     &#xHHH; / &#XHHH;
                let entity_start = i;
                i += 1;
                let ent_name_start = i;
                // Entity bodies are short; cap the scan to avoid eating into
                // normal text when the closing ';' is missing.
                while i < len && input[i] != b';' && i - entity_start < 12 {
                    i += 1;
                }
                if i < len && input[i] == b';' {
                    let entity = &text[ent_name_start..i];
                    i += 1; // skip ';'

                    // Numeric entity?
                    let decoded_numeric = entity.strip_prefix('#').and_then(|rest| {
                        let cp = if let Some(hex) = rest
                            .strip_prefix('x')
                            .or_else(|| rest.strip_prefix('X'))
                        {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            rest.parse::<u32>().ok()
                        };
                        cp.and_then(char::from_u32)
                    });

                    if let Some(ch) = decoded_numeric {
                        result.push(ch);
                    } else if let Some(decoded) = decode_entity(entity) {
                        result.push_str(decoded);
                    } else {
                        result.push_str(&text[entity_start..i]);
                    }
                } else {
                    // Not a valid entity, output as-is
                    i = entity_start + 1;
                    result.push('&');
                }
            } else {
                let ch = text[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        }

        *text = result;
    }
}

fn is_block_tag(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "p" | "div"
            | "br"
            | "hr"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "tr"
            | "td"
            | "th"
            | "blockquote"
            | "pre"
            | "section"
            | "article"
            | "header"
            | "footer"
    )
}

fn decode_entity(entity: &str) -> Option<&'static str> {
    match entity {
        "amp" => Some("&"),
        "lt" => Some("<"),
        "gt" => Some(">"),
        "quot" => Some("\""),
        "apos" => Some("'"),
        "nbsp" => Some(" "),
        "copy" => Some("©"),
        "reg" => Some("®"),
        "trade" => Some("™"),
        "mdash" => Some("—"),
        "ndash" => Some("–"),
        "hellip" => Some("…"),
        "laquo" => Some("«"),
        "raquo" => Some("»"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_strip_basic() {
        let n = HtmlStripNormalizer::new();
        let mut text = String::from("<p>Hello <b>World</b></p>");
        n.normalize(&mut text);
        assert_eq!(text, " Hello World ");
    }

    #[test]
    fn test_html_strip_entities() {
        let n = HtmlStripNormalizer::new();
        let mut text = String::from("&amp; &lt; &gt; &quot;");
        n.normalize(&mut text);
        assert_eq!(text, "& < > \"");
    }

    #[test]
    fn test_html_strip_escaped_tags() {
        let n = HtmlStripNormalizer::new().with_escaped_tags(vec!["b".to_string()]);
        let mut text = String::from("<p>Hello <b>World</b></p>");
        n.normalize(&mut text);
        assert!(text.contains("<b>"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn test_html_strip_numeric_entities() {
        let n = HtmlStripNormalizer::new();
        // &#65; -> 'A', &#x4E2D; -> '中'
        let mut text = String::from("&#65;BC &#x4E2D;");
        n.normalize(&mut text);
        assert_eq!(text, "ABC 中");
    }
}
