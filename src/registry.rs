//! High-level analysis registry providing ergonomic access to the full toolkit.
//!
//! [`AnalysisRegistry`] wraps [`AnalysisFactory`] with a simpler API and
//! pre-registers all built-in components from `analysis-core`.

use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::AnalyzerConfig;

use crate::analyzers::register_all;

/// A high-level analysis registry with all built-in components pre-registered.
///
/// This is the recommended entry point for using the analysis toolkit.
/// It provides a simpler API over the raw [`AnalysisFactory`] and includes
/// convenience methods for common operations.
///
/// # Example
///
/// ```rust
/// use pizza_analysis_core::AnalysisRegistry;
///
/// let registry = AnalysisRegistry::new();
///
/// // Use a pre-built analyzer
/// let terms = registry.analyze("english", "The quick brown foxes");
///
/// // Create a custom analyzer from config
/// let terms = registry.analyze_custom(&["lowercase"], "standard", &["asciifolding"], "Café");
/// ```
pub struct AnalysisRegistry {
    factory: AnalysisFactory,
}

impl AnalysisRegistry {
    /// Create a new registry with all built-in components registered.
    pub fn new() -> Self {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        Self { factory }
    }

    /// Analyze text using a named pre-built analyzer.
    /// Returns the resulting terms, or an empty vec if the analyzer doesn't exist.
    pub fn analyze(&self, analyzer_name: &str, text: &str) -> Vec<String> {
        if let Some(analyzer) = self.factory.get_analyzer(analyzer_name) {
            let mut input = String::from(text);
            let tokens = analyzer.analyze_and_return_tokens(&mut input);
            tokens.into_iter().map(|t| t.term.into_owned()).collect()
        } else {
            Vec::new()
        }
    }

    /// Analyze text and return detailed token info: `(term, position, start, end)`.
    pub fn analyze_detailed(
        &self,
        analyzer_name: &str,
        text: &str,
    ) -> Vec<(String, u32, u32, u32)> {
        if let Some(analyzer) = self.factory.get_analyzer(analyzer_name) {
            let mut input = String::from(text);
            let tokens = analyzer.analyze_and_return_tokens(&mut input);
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
        } else {
            Vec::new()
        }
    }

    /// Create and use a custom analyzer from component names.
    /// Returns the resulting terms, or an empty vec if the tokenizer doesn't exist.
    pub fn analyze_custom(
        &self,
        normalizers: &[&str],
        tokenizer: &str,
        token_filters: &[&str],
        text: &str,
    ) -> Vec<String> {
        let config =
            AnalyzerConfig::from_parts(normalizers.to_vec(), tokenizer, token_filters.to_vec());
        if let Some(analyzer) = self.factory.create_analyzer_from_config(&config) {
            let mut input = String::from(text);
            let tokens = analyzer.analyze_and_return_tokens(&mut input);
            tokens.into_iter().map(|t| t.term.into_owned()).collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a named analyzer exists in the registry.
    pub fn has_analyzer(&self, name: &str) -> bool {
        self.factory.get_analyzer(name).is_some()
    }

    /// Check if a named tokenizer exists in the registry.
    pub fn has_tokenizer(&self, name: &str) -> bool {
        self.factory.get_tokenizer(name).is_some()
    }

    /// Check if a named token filter exists in the registry.
    pub fn has_token_filter(&self, name: &str) -> bool {
        self.factory.get_token_filter(name).is_some()
    }

    /// Check if a named normalizer exists in the registry.
    pub fn has_normalizer(&self, name: &str) -> bool {
        self.factory.get_normalizer(name).is_some()
    }

    /// Get a reference to the underlying factory for advanced usage.
    pub fn factory(&self) -> &AnalysisFactory {
        &self.factory
    }

    /// Get a mutable reference to register additional custom components.
    pub fn factory_mut(&mut self) -> &mut AnalysisFactory {
        &mut self.factory
    }

    /// List all registered analyzer names (useful for discovery).
    pub fn analyzer_names(&self) -> Vec<&str> {
        // We know the set of analyzers we registered
        let mut names: Vec<&str> = Vec::new();
        let known = [
            "afrikaans",
            "amharic",
            "arabic",
            "armenian",
            "azerbaijani",
            "basque",
            "bengali",
            "brazilian",
            "bulgarian",
            "catalan",
            "cjk",
            "croatian",
            "czech",
            "danish",
            "dutch",
            "english",
            "estonian",
            "filipino",
            "fingerprint",
            "finnish",
            "french",
            "galician",
            "georgian",
            "german",
            "greek",
            "hebrew",
            "hindi",
            "hungarian",
            "indonesian",
            "irish",
            "italian",
            "keyword",
            "latvian",
            "lithuanian",
            "malay",
            "marathi",
            "mongolian",
            "nepali",
            "norwegian",
            "pattern",
            "persian",
            "polish",
            "portuguese",
            "romanian",
            "russian",
            "serbian",
            "simple",
            "slovak",
            "slovenian",
            "sorani",
            "spanish",
            "stop",
            "swahili",
            "swedish",
            "tagalog",
            "tamil",
            "thai",
            "turkish",
            "ukrainian",
            "urdu",
            "vietnamese",
            "whitespace",
        ];
        for name in &known {
            if self.factory.get_analyzer(name).is_some() {
                names.push(name);
            }
        }
        names
    }
}

impl Default for AnalysisRegistry {
    fn default() -> Self {
        Self::new()
    }
}
