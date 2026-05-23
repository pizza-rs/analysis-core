//! Generic stemmer token filter that dispatches by language name.
//!
//! Equivalent to Elasticsearch's `stemmer` token filter which takes a
//! `language` parameter and applies the appropriate stemming algorithm.

use alloc::borrow::Cow;
use alloc::string::String;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

use super::arabic_stem::ArabicStemTokenFilter;
use super::bengali_stem::BengaliStemTokenFilter;
use super::brazilian_stem::BrazilianStemTokenFilter;
use super::bulgarian_stem::BulgarianStemTokenFilter;
use super::european_stemmers::*;
use super::finnish_hungarian_indonesian_stemmers::*;
use super::french_stem::*;
use super::galician_stem::*;
use super::german_stem::*;
use super::greek_stem::GreekStemTokenFilter;
use super::hindi_stem::HindiStemTokenFilter;
use super::latvian_stem::LatvianStemTokenFilter;
use super::misc_stemmers::*;
use super::persian_stem::PersianStemTokenFilter;
use super::south_asian_stemmers::*;
use alloc::vec::Vec;

/// Which stemming variant to use for a language.
#[derive(Clone, Debug)]
pub enum StemmerLanguage {
    Arabic,
    Bengali,
    Brazilian,
    Bulgarian,
    Czech,
    Dutch,
    English, // KStem (default for English)
    Finnish,
    French,
    FrenchMinimal,
    Galician,
    GalicianMinimal,
    German,
    GermanMinimal,
    Greek,
    Hindi,
    Hungarian,
    Indonesian,
    Italian,
    Kannada,
    Latvian,
    Norwegian,
    Persian,
    Portuguese,
    Russian,
    Spanish,
    Tamil,
    Telugu,
}

impl StemmerLanguage {
    /// Parse a language name string into a [`StemmerLanguage`] variant.
    /// Returns `None` for unsupported languages.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "arabic" => Some(Self::Arabic),
            "bengali" => Some(Self::Bengali),
            "brazilian" | "brazilian_portuguese" => Some(Self::Brazilian),
            "bulgarian" => Some(Self::Bulgarian),
            "czech" => Some(Self::Czech),
            "dutch" | "dutch_kp" => Some(Self::Dutch),
            "english" | "light_english" | "porter" | "porter2" => Some(Self::English),
            "finnish" | "light_finnish" => Some(Self::Finnish),
            "french" | "light_french" => Some(Self::French),
            "minimal_french" => Some(Self::FrenchMinimal),
            "galician" => Some(Self::Galician),
            "minimal_galician" => Some(Self::GalicianMinimal),
            "german" | "light_german" => Some(Self::German),
            "minimal_german" => Some(Self::GermanMinimal),
            "greek" => Some(Self::Greek),
            "hindi" => Some(Self::Hindi),
            "hungarian" | "light_hungarian" => Some(Self::Hungarian),
            "indonesian" => Some(Self::Indonesian),
            "italian" | "light_italian" => Some(Self::Italian),
            "kannada" => Some(Self::Kannada),
            "latvian" => Some(Self::Latvian),
            "norwegian" | "light_norwegian" | "light_nynorsk" => Some(Self::Norwegian),
            "persian" => Some(Self::Persian),
            "portuguese" | "light_portuguese" => Some(Self::Portuguese),
            "russian" | "light_russian" => Some(Self::Russian),
            "spanish" | "light_spanish" => Some(Self::Spanish),
            "tamil" => Some(Self::Tamil),
            "telugu" => Some(Self::Telugu),
            _ => None,
        }
    }

    /// Get the list of all supported language names.
    pub fn supported_languages() -> &'static [&'static str] {
        &[
            "arabic",
            "bengali",
            "brazilian",
            "bulgarian",
            "czech",
            "dutch",
            "english",
            "finnish",
            "french",
            "galician",
            "german",
            "greek",
            "hindi",
            "hungarian",
            "indonesian",
            "italian",
            "kannada",
            "latvian",
            "norwegian",
            "persian",
            "portuguese",
            "russian",
            "spanish",
            "tamil",
            "telugu",
        ]
    }
}

/// A generic stemmer token filter that applies stemming based on a configured language.
///
/// This is the equivalent of Elasticsearch's `stemmer` filter, which accepts a
/// `language` parameter and internally dispatches to the appropriate algorithm.
///
/// # Example
///
/// ```rust
/// use pizza_analysis_core::StemmerTokenFilter;
///
/// // Create a stemmer for English
/// let stemmer = StemmerTokenFilter::new("english").unwrap();
///
/// // Create a stemmer for French
/// let stemmer = StemmerTokenFilter::new("french").unwrap();
/// ```
#[derive(Clone)]
pub struct StemmerTokenFilter {
    language: StemmerLanguage,
}

impl StemmerTokenFilter {
    /// Create a new stemmer filter for the given language.
    /// Returns `None` if the language is not supported.
    pub fn new(language: &str) -> Option<Self> {
        StemmerLanguage::from_name(language).map(|lang| Self { language: lang })
    }

    /// Create a stemmer filter from a [`StemmerLanguage`] enum value directly.
    pub fn from_language(language: StemmerLanguage) -> Self {
        Self { language }
    }

    fn apply_stemming<'a>(&self, term: &str) -> Cow<'a, str> {
        let result = match &self.language {
            StemmerLanguage::Arabic => {
                let f = ArabicStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Bengali => {
                let f = BengaliStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Brazilian => {
                let f = BrazilianStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Bulgarian => {
                let f = BulgarianStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Czech => {
                let f = CzechStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Dutch => {
                let f = DutchStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::English => {
                let f = super::kstem::KStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Finnish => {
                let f = FinnishLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::French => {
                let f = FrenchLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::FrenchMinimal => {
                let f = FrenchMinimalStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Galician => {
                let f = GalicianStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::GalicianMinimal => {
                let f = GalicianMinimalStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::German => {
                let f = GermanLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::GermanMinimal => {
                let f = GermanMinimalStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Greek => {
                let f = GreekStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Hindi => {
                let f = HindiStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Hungarian => {
                let f = HungarianLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Indonesian => {
                let f = IndonesianStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Italian => {
                let f = ItalianLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Kannada => {
                let f = KannadaStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Latvian => {
                let f = LatvianStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Norwegian => {
                let f = NorwegianLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Persian => {
                let f = PersianStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Portuguese => {
                let f = PortugueseLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Russian => {
                let f = RussianLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Spanish => {
                let f = SpanishLightStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Tamil => {
                let f = TamilStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
            StemmerLanguage::Telugu => {
                let f = TeluguStemTokenFilter::new();
                self.filter_to_string(&f, term)
            }
        };
        Cow::Owned(result)
    }

    fn filter_to_string(&self, filter: &dyn TokenFilter, term: &str) -> String {
        let mut token = Token::new(term, 0, term.len() as u32, 0);
        filter.filter(&mut token);
        token.term.into_owned()
    }
}

impl TokenFilter for StemmerTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let stemmed = self.apply_stemming(&token.term);
        token.term = stemmed;
        (false, None)
    }
}
