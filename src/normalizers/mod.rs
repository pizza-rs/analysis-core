//! Built-in normalizers.
//!
//! In Pizza's architecture, normalizers serve the role of both ES "char filters"
//! and pre-tokenization text transforms. They mutate the input string in-place
//! before tokenization.

mod case;
mod html_strip;
mod mapping;
mod pattern_replace;
mod unicode;

pub use case::CollapseWhitespaceNormalizer;
pub use case::TrimNormalizer;
// LowercaseNormalizer and UppercaseNormalizer are provided by pizza-engine
pub use html_strip::HtmlStripNormalizer;
pub use mapping::MappingNormalizer;
pub use pattern_replace::PatternReplaceNormalizer;
pub use pizza_engine::analysis::LowercaseNormalizer;
pub use pizza_engine::analysis::UppercaseNormalizer;
pub use unicode::UnicodeNormForm;
pub use unicode::UnicodeNormalizer;
