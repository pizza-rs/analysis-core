//! Fluent builder for constructing custom analyzers at runtime.
//!
//! # Example
//!
//! ```rust
//! use pizza_analysis_core::AnalyzerBuilder;
//! use pizza_analysis_core::AsciiFoldingTokenFilter;
//! use pizza_analysis_core::HtmlStripNormalizer;
//! use pizza_analysis_core::LowercaseTokenFilter;
//! use pizza_analysis_core::StandardTokenizer;
//! use pizza_analysis_core::StopTokenFilter;
//!
//! let analyzer = AnalyzerBuilder::new()
//!     .normalizer(HtmlStripNormalizer::new())
//!     .tokenizer(StandardTokenizer::new())
//!     .filter(LowercaseTokenFilter::new())
//!     .filter(AsciiFoldingTokenFilter::new())
//!     .filter(StopTokenFilter::new(&["the", "a", "is"]))
//!     .build();
//! ```

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Analyzer;
use pizza_engine::analysis::Normalizer;
use pizza_engine::analysis::StandardTokenizer;
use pizza_engine::analysis::TokenFilter;
use pizza_engine::analysis::Tokenizer;

/// A fluent builder for constructing [`Analyzer`] instances from individual components.
///
/// The builder requires at minimum a tokenizer (defaults to [`StandardTokenizer`] if not set).
/// Normalizers and token filters are optional and applied in insertion order.
pub struct AnalyzerBuilder {
    normalizers: Vec<Box<dyn Normalizer>>,
    tokenizer: Option<Box<dyn Tokenizer>>,
    token_filters: Vec<Box<dyn TokenFilter>>,
}

impl AnalyzerBuilder {
    /// Create a new empty builder.
    pub fn new() -> Self {
        Self {
            normalizers: Vec::new(),
            tokenizer: None,
            token_filters: Vec::new(),
        }
    }

    /// Add a normalizer to the pipeline. Normalizers run in insertion order
    /// before tokenization.
    pub fn normalizer<N: Normalizer + 'static>(mut self, normalizer: N) -> Self {
        self.normalizers.push(Box::new(normalizer));
        self
    }

    /// Set the tokenizer. Only one tokenizer is allowed per analyzer.
    /// Calling this multiple times replaces the previous tokenizer.
    pub fn tokenizer<T: Tokenizer + 'static>(mut self, tokenizer: T) -> Self {
        self.tokenizer = Some(Box::new(tokenizer));
        self
    }

    /// Add a token filter to the pipeline. Filters run in insertion order
    /// after tokenization.
    pub fn filter<F: TokenFilter + 'static>(mut self, filter: F) -> Self {
        self.token_filters.push(Box::new(filter));
        self
    }

    /// Add multiple token filters at once.
    pub fn filters<I, F>(mut self, filters: I) -> Self
    where
        I: IntoIterator<Item = F>,
        F: TokenFilter + 'static,
    {
        for filter in filters {
            self.token_filters.push(Box::new(filter));
        }
        self
    }

    /// Build the [`Analyzer`]. Uses [`StandardTokenizer`] if no tokenizer was set.
    pub fn build(self) -> Analyzer {
        let tokenizer = self
            .tokenizer
            .unwrap_or_else(|| Box::new(StandardTokenizer::new()));
        Analyzer::new(self.normalizers, tokenizer, self.token_filters)
    }
}

impl Default for AnalyzerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Convenience Analysis Utilities ────────────────────────────────────────

/// Analyze text using an [`Analyzer`] and return the resulting token terms as strings.
///
/// This is the simplest way to invoke an analyzer pipeline and get results.
///
/// # Example
///
/// ```rust
/// use pizza_analysis_core::analyze_text;
/// use pizza_analysis_core::AnalyzerBuilder;
/// use pizza_analysis_core::LowercaseTokenFilter;
/// use pizza_analysis_core::StandardTokenizer;
///
/// let analyzer = AnalyzerBuilder::new()
///     .tokenizer(StandardTokenizer::new())
///     .filter(LowercaseTokenFilter::new())
///     .build();
///
/// let terms = analyze_text(&analyzer, "Hello World");
/// assert_eq!(terms, vec!["hello", "world"]);
/// ```
pub fn analyze_text(analyzer: &Analyzer, input: &str) -> Vec<String> {
    let mut text = String::from(input);
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    tokens.into_iter().map(|t| t.term.into_owned()).collect()
}

/// Analyze text and return `(term, position, start_offset, end_offset)` tuples.
///
/// Useful for debugging or verifying analysis pipeline behavior.
pub fn analyze_text_detailed(analyzer: &Analyzer, input: &str) -> Vec<(String, u32, u32, u32)> {
    let mut text = String::from(input);
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    tokens
        .into_iter()
        .map(|t| {
            (
                t.term.into_owned(),
                t.position,
                t.start_offset,
                t.end_offset,
            )
        })
        .collect()
}

/// Analyze text and return only the unique terms (deduped, preserving first occurrence order).
pub fn analyze_text_unique(analyzer: &Analyzer, input: &str) -> Vec<String> {
    let terms = analyze_text(analyzer, input);
    let mut seen = hashbrown::HashSet::new();
    let mut result = Vec::new();
    for term in terms {
        if seen.insert(term.clone()) {
            result.push(term);
        }
    }
    result
}
