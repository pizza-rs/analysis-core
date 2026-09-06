//! Pizza Analysis Core — Built-in analysis components
//!
//! This crate provides all generic built-in analysis components for the Pizza
//! search engine, providing common analysis components for text processing.
//!
//! # Architecture
//!
//! Pizza uses a three-stage pipeline: **Normalize → Tokenize → Filter**
//!
//! - **Normalizers:** Pre-tokenization string transforms
//!   `html_strip`, `mapping`, `pattern_replace`
//! - **Tokenizers:** `keyword`, `letter`, `lowercase`, `ngram`, `edge_ngram`,
//!   `char_group`, `path_hierarchy`, `pattern`, `classic`, `uax_url_email`,
//!   `simple_pattern`, `simple_pattern_split`, `thai`
//! - **Token Filters:** `stop`, `asciifolding`, `trim`, `truncate`, `length`,
//!   `limit`, `reverse`, `unique`, `remove_duplicates`, `keyword_marker`,
//!   `keyword_repeat`, `apostrophe`, `elision`, `ngram`, `edge_ngram`,
//!   `shingle`, `common_grams`, `keep_words`, `fingerprint`, `classic`,
//!   `pattern_capture`, `pattern_replace`, `cjk_bigram`, `cjk_width`,
//!   `decimal_digit`, `stemmer_override`, `delimited_payload`,
//!   `delimited_term_freq`, language normalizations & stemmers
//!
//! # `std` dependency
//!
//! This crate currently requires `std`. Several token filters use
//! `std::sync::Mutex` for stateful buffering across calls, and the
//! `regex` dependency (used by pattern-based components) also pulls in
//! `std`. An earlier version of this file declared `#![no_std]` behind
//! a `std` cargo feature; that declaration was inconsistent with the
//! actual dependency graph and has been removed.

extern crate alloc;

pub mod analyzers;
pub mod builder;
pub mod normalizers;
pub mod registry;
pub mod stream;
pub mod token_filters;
pub mod tokenizers;

pub use analyzers::register_all;
pub use builder::analyze_text;
pub use builder::analyze_text_detailed;
pub use builder::analyze_text_unique;
pub use builder::AnalyzerBuilder;
pub use registry::AnalysisRegistry;
pub use stream::TokenStream;
pub use stream::TokenStreamExt;

// ─── Normalizers ───────────────────────────────────────────────────────────
pub use normalizers::CollapseWhitespaceNormalizer;
pub use normalizers::HtmlStripNormalizer;
pub use normalizers::LowercaseNormalizer;
pub use normalizers::MappingNormalizer;
pub use normalizers::PatternReplaceNormalizer;
pub use normalizers::TrimNormalizer;
pub use normalizers::UnicodeNormForm;
pub use normalizers::UnicodeNormalizer;
pub use normalizers::UppercaseNormalizer;

// ─── Tokenizers ────────────────────────────────────────────────────────────
pub use tokenizers::BurmeseTokenizer;
pub use tokenizers::CamelCaseTokenizer;
pub use tokenizers::CharGroupTokenizer;
pub use tokenizers::ChineseCharTokenizer;
pub use tokenizers::ClassicTokenizer;
pub use tokenizers::CodeTokenizer;
pub use tokenizers::CompoundWordTokenizer;
pub use tokenizers::EdgeNgramTokenizer;
pub use tokenizers::ElisionTokenizer;
pub use tokenizers::EmailTokenizer;
pub use tokenizers::EmojiTokenizer;
pub use tokenizers::FingerprintTokenizer;
pub use tokenizers::HyphenatedTokenizer;
pub use tokenizers::JsonFieldTokenizer;
pub use tokenizers::KeywordTokenizer;
pub use tokenizers::LetterTokenizer;
pub use tokenizers::LogTokenizer;
pub use tokenizers::LowercaseTokenizer;
pub use tokenizers::MarkdownTokenizer;
pub use tokenizers::MicroBlogTokenizer;
pub use tokenizers::NgramTokenizer;
pub use tokenizers::PathHierarchyTokenizer;
pub use tokenizers::PatternTokenizer;
pub use tokenizers::PhoneNumberTokenizer;
pub use tokenizers::PunctuationTokenizer;
pub use tokenizers::ReverseTokenizer;
pub use tokenizers::ScriptBoundaryTokenizer;
pub use tokenizers::SentenceTokenizer;
pub use tokenizers::SimplePatternSplitTokenizer;
pub use tokenizers::SimplePatternTokenizer;
pub use tokenizers::SlidingWindowTokenizer;
pub use tokenizers::StandardTokenizer;
pub use tokenizers::StructuredIdTokenizer;
pub use tokenizers::TabSeparatedTokenizer;
pub use tokenizers::ThaiTokenizer;
pub use tokenizers::TruncateTokenizer;
pub use tokenizers::UaxUrlEmailTokenizer;
pub use tokenizers::UrlTokenizer;
pub use tokenizers::WhitespaceTokenizer;
pub use tokenizers::WildcardTokenizer;

// ─── Token Filters: Core ───────────────────────────────────────────────────
pub use token_filters::ApostropheTokenFilter;
pub use token_filters::AsciiFoldingTokenFilter;
pub use token_filters::ClassicTokenFilter;
pub use token_filters::DecimalDigitTokenFilter;
pub use token_filters::ElisionTokenFilter;
pub use token_filters::KeepWordsTokenFilter;
pub use token_filters::KeywordMarkerTokenFilter;
pub use token_filters::KeywordRepeatTokenFilter;
pub use token_filters::LengthTokenFilter;
pub use token_filters::LimitTokenFilter;
pub use token_filters::LowercaseTokenFilter;
pub use token_filters::ReverseTokenFilter;
pub use token_filters::StopTokenFilter;
pub use token_filters::TrimTokenFilter;
pub use token_filters::TruncateTokenFilter;
pub use token_filters::UniqueTokenFilter;
pub use token_filters::UppercaseTokenFilter;

// ─── Token Filters: N-gram & Shingle ──────────────────────────────────────
pub use token_filters::CommonGramsTokenFilter;
pub use token_filters::CommonGramsQueryFilter;
pub use token_filters::FixBrokenOffsetsFilter;
pub use token_filters::RussianYoNormalizationTokenFilter;
pub use token_filters::EdgeNgramTokenFilter;
pub use token_filters::NgramTokenFilter;
pub use token_filters::ShingleTokenFilter;

// ─── Token Filters: Pattern ────────────────────────────────────────────────
pub use token_filters::PatternCaptureTokenFilter;
pub use token_filters::PatternReplaceTokenFilter;

// ─── Token Filters: Phonetic ───────────────────────────────────────────────
pub use token_filters::BeiderMorseFilter;
pub use token_filters::BmNameType;
pub use token_filters::BmRuleType;
pub use token_filters::PhoneticEncoder;
pub use token_filters::PhoneticTokenFilter;

// ─── Token Filters: Hunspell ───────────────────────────────────────────────
pub use token_filters::AffixRule;
pub use token_filters::HunspellStemFilter;

// ─── Token Filters: Phone ──────────────────────────────────────────────────
pub use token_filters::PhoneNumberFilter;

// ─── Token Filters: Synonyms & Word Processing ────────────────────────────
pub use token_filters::DelimitedPayloadTokenFilter;
pub use token_filters::DelimitedTermFreqTokenFilter;
pub use token_filters::DictionaryStemTokenFilter;
pub use token_filters::FingerprintAccumulator;
pub use token_filters::FingerprintTokenFilter;
pub use token_filters::FlattenGraphTokenFilter;
pub use token_filters::ProtectedWordsTokenFilter;
pub use token_filters::RemoveDuplicatesState;
pub use token_filters::RemoveDuplicatesTokenFilter;
pub use token_filters::StemmerLanguage;
pub use token_filters::StemmerOverrideTokenFilter;
pub use token_filters::StemmerTokenFilter;
pub use token_filters::SynonymMode;
pub use token_filters::SynonymTokenFilter;
pub use token_filters::WordDelimiterConfig;
pub use token_filters::WordDelimiterGraphTokenFilter;
pub use token_filters::WordDelimiterTokenFilter;

// ─── Token Filters: Compound Decomposition ─────────────────────────────────
pub use token_filters::DictionaryDecompounderTokenFilter;
pub use token_filters::HyphenatedWordsTokenFilter;
pub use token_filters::HyphenationDecompounderTokenFilter;

// ─── Token Filters: Type Filtering ────────────────────────────────────────
pub use token_filters::KeepTypesMode;
pub use token_filters::KeepTypesTokenFilter;
pub use token_filters::TokenType;

// ─── Token Filters: Conditional & Multiplexer ──────────────────────────────
pub use token_filters::ConditionalTokenFilter;
pub use token_filters::MaxLengthPredicate;
pub use token_filters::MinLengthPredicate;
pub use token_filters::MultiplexerTokenFilter;
pub use token_filters::PatternPredicate;
pub use token_filters::TokenPredicate;

// ─── Token Filters: MinHash & KStem ────────────────────────────────────────
pub use token_filters::KStemTokenFilter;
pub use token_filters::MinHashTokenFilter;

// ─── Token Filters: Predicate & Script ─────────────────────────────────────
pub use token_filters::detect_script;
pub use token_filters::PredicateTokenFilter;
pub use token_filters::ScriptType;
pub use token_filters::TokenPredicateType;

// ─── Token Filters: CJK ───────────────────────────────────────────────────
pub use token_filters::CjkBigramTokenFilter;
pub use token_filters::CjkWidthTokenFilter;

// ─── Token Filters: Language Normalizations ────────────────────────────────
pub use token_filters::ArabicNormalizationTokenFilter;
pub use token_filters::BengaliNormalizationTokenFilter;
pub use token_filters::GermanNormalizationTokenFilter;
pub use token_filters::GreekLowercaseTokenFilter;
pub use token_filters::HindiNormalizationTokenFilter;
pub use token_filters::IndicNormalizationTokenFilter;
pub use token_filters::IrishElisionTokenFilter;
pub use token_filters::IrishLowercaseTokenFilter;
pub use token_filters::PersianNormalizationTokenFilter;
pub use token_filters::RomanianNormalizationTokenFilter;
pub use token_filters::ScandinavianFoldingTokenFilter;
pub use token_filters::ScandinavianNormalizationTokenFilter;
pub use token_filters::SerbianNormalizationTokenFilter;
pub use token_filters::SoraniNormalizationTokenFilter;
pub use token_filters::TurkishLowercaseTokenFilter;

// ─── Token Filters: Language Stemmers ──────────────────────────────────────
pub use token_filters::ArabicStemTokenFilter;
pub use token_filters::BengaliStemTokenFilter;
pub use token_filters::BrazilianStemTokenFilter;
pub use token_filters::BulgarianStemTokenFilter;
pub use token_filters::CzechStemTokenFilter;
pub use token_filters::DutchStemTokenFilter;
pub use token_filters::FinnishLightStemTokenFilter;
pub use token_filters::FrenchLightStemTokenFilter;
pub use token_filters::FrenchMinimalStemTokenFilter;
pub use token_filters::GalicianMinimalStemTokenFilter;
pub use token_filters::GalicianStemTokenFilter;
pub use token_filters::GermanLightStemTokenFilter;
pub use token_filters::GermanMinimalStemTokenFilter;
pub use token_filters::GreekStemTokenFilter;
pub use token_filters::HindiStemTokenFilter;
pub use token_filters::HungarianLightStemTokenFilter;
pub use token_filters::IndonesianStemTokenFilter;
pub use token_filters::ItalianLightStemTokenFilter;
pub use token_filters::KannadaStemTokenFilter;
pub use token_filters::LatvianStemTokenFilter;
pub use token_filters::NorwegianLightStemTokenFilter;
pub use token_filters::PersianStemTokenFilter;
pub use token_filters::PortugueseLightStemTokenFilter;
pub use token_filters::RussianLightStemTokenFilter;
pub use token_filters::SpanishLightStemTokenFilter;
pub use token_filters::TamilStemTokenFilter;
pub use token_filters::TeluguStemTokenFilter;

// ─── Token Filters: Additional Lucene-Parity Filters ───────────────────────
pub use token_filters::CapitalizationTokenFilter;
pub use token_filters::CodepointCountTokenFilter;
pub use token_filters::ConcatenateGraphTokenFilter;
pub use token_filters::DateRecognizerTokenFilter;
pub use token_filters::DelimitedBoostTokenFilter;
pub use token_filters::DropIfFlaggedTokenFilter;
pub use token_filters::EnglishMinimalStemTokenFilter;
pub use token_filters::FixedShingleTokenFilter;
pub use token_filters::LimitTokenOffsetFilter;
pub use token_filters::LimitTokenPositionFilter;
pub use token_filters::NorwegianMinimalStemTokenFilter;
pub use token_filters::NorwegianNormalizationTokenFilter;
pub use token_filters::PatternKeywordMarkerTokenFilter;
pub use token_filters::PatternTypingTokenFilter;
pub use token_filters::PortugueseMinimalStemTokenFilter;
pub use token_filters::SerbianNormalizationRegularTokenFilter;
pub use token_filters::SoraniStemTokenFilter;
pub use token_filters::SpanishMinimalStemTokenFilter;
pub use token_filters::SpanishPluralStemTokenFilter;
pub use token_filters::SwedishMinimalStemTokenFilter;
pub use token_filters::TypeAsSynonymTokenFilter;
pub use token_filters::concatenate_tokens;
pub use token_filters::build_fixed_shingles;

// ─── Token Filters: Beyond-Lucene Innovative Filters ───────────────────────

// Advanced Unicode
pub use token_filters::BiDiStripTokenFilter;
pub use token_filters::ConfusableNormTokenFilter;
pub use token_filters::DiacriticStripTokenFilter;
pub use token_filters::EmojiPresenceTokenFilter;
pub use token_filters::FullwidthNormTokenFilter;
pub use token_filters::HomoglyphNormTokenFilter;
pub use token_filters::InvisibleCharRemoveTokenFilter;
pub use token_filters::MixedScriptDetectTokenFilter;
pub use token_filters::ZeroWidthRemoveTokenFilter;

// Code, Log & Science
pub use token_filters::AnsiStripTokenFilter;
pub use token_filters::DoiNormTokenFilter;
pub use token_filters::ErrorCodeTokenFilter;
pub use token_filters::FileExtensionTokenFilter;
pub use token_filters::HttpStatusTokenFilter;
pub use token_filters::IdentifierSplitTokenFilter;
pub use token_filters::IsbnNormTokenFilter;
pub use token_filters::KeyValuePairTokenFilter;
pub use token_filters::LogLevelTokenFilter;
pub use token_filters::PathComponentTokenFilter;
pub use token_filters::ProgrammingKeywordTokenFilter;
pub use token_filters::SemverTokenFilter;

// Emoji
pub use token_filters::EmojiExtractTokenFilter;
pub use token_filters::EmojiRemoveTokenFilter;
pub use token_filters::EmojiSentimentTokenFilter;
pub use token_filters::EmojiToTextTokenFilter;
pub use token_filters::EmoticonToTextTokenFilter;

// Encoding & Hash
pub use token_filters::Base64DecodeTokenFilter;
pub use token_filters::Base64EncodeTokenFilter;
pub use token_filters::Crc32TokenFilter;
pub use token_filters::FnvHashTokenFilter;
pub use token_filters::HashAlgorithm;
pub use token_filters::HashSynonymTokenFilter;
pub use token_filters::HexDecodeTokenFilter;
pub use token_filters::HexEncodeTokenFilter;
pub use token_filters::MurmurHash3TokenFilter;
pub use token_filters::Rot13TokenFilter;
pub use token_filters::UrlDecodeTokenFilter;
pub use token_filters::UrlEncodeTokenFilter;

// Extraction
pub use token_filters::CurrencyExtractTokenFilter;
pub use token_filters::EmailDomainTokenFilter;
pub use token_filters::EmailExtractTokenFilter;
pub use token_filters::HashtagExtractTokenFilter;
pub use token_filters::IpExtractTokenFilter;
pub use token_filters::LookupTokenFilter;
pub use token_filters::MentionExtractTokenFilter;
pub use token_filters::NumberExtractTokenFilter;
pub use token_filters::PhoneExtractTokenFilter;
pub use token_filters::UrlExtractTokenFilter;

// NLP & Social
pub use token_filters::AbbreviationExpandTokenFilter;
pub use token_filters::ContractionExpandTokenFilter;
pub use token_filters::HashtagRemoveTokenFilter;
pub use token_filters::HashtagSplitTokenFilter;
pub use token_filters::HashtagTagTokenFilter;
pub use token_filters::MentionRemoveTokenFilter;
pub use token_filters::MentionTagTokenFilter;
pub use token_filters::SentenceCaseTokenFilter;
pub use token_filters::SentimentTagTokenFilter;
pub use token_filters::ShoutingNormTokenFilter;
pub use token_filters::SlangNormTokenFilter;
pub use token_filters::StretchedWordNormTokenFilter;

// Number & Conversion
pub use token_filters::BinaryToDecimalTokenFilter;
pub use token_filters::DurationNormTokenFilter;
pub use token_filters::FileSizeNormTokenFilter;
pub use token_filters::HexToDecimalTokenFilter;
pub use token_filters::NumberMagnitudeTokenFilter;
pub use token_filters::NumberNormTokenFilter;
pub use token_filters::NumericRangeTokenFilter;
pub use token_filters::OctalToDecimalTokenFilter;
pub use token_filters::OrdinalTokenFilter;
pub use token_filters::PercentNormTokenFilter;
pub use token_filters::RomanNumeralTokenFilter;

// Security & Privacy
pub use token_filters::CreditCardMaskTokenFilter;
pub use token_filters::EmailMaskTokenFilter;
pub use token_filters::IpMaskTokenFilter;
pub use token_filters::PathTraversalDetectTokenFilter;
pub use token_filters::PhoneMaskTokenFilter;
pub use token_filters::RedactTokenFilter;
pub use token_filters::SqlInjectionDetectTokenFilter;
pub use token_filters::SsnMaskTokenFilter;
pub use token_filters::XssDetectTokenFilter;

// Text Metrics
pub use token_filters::ByteLengthTokenFilter;
pub use token_filters::CharCountTokenFilter;
pub use token_filters::EntropyFilterTokenFilter;
pub use token_filters::EntropyTokenFilter;
pub use token_filters::LanguageTagTokenFilter;
pub use token_filters::LengthBandTokenFilter;
pub use token_filters::ScriptTagTokenFilter;
pub use token_filters::SyllableCountTokenFilter;
pub use token_filters::UuidDetectTokenFilter;

// Text Transform
pub use token_filters::CamelCaseTokenFilter;
pub use token_filters::CamelCaseSplitTokenFilter;
pub use token_filters::CollapseRepeatsTokenFilter;
pub use token_filters::KebabCaseTokenFilter;
pub use token_filters::PadTokenFilter;
pub use token_filters::PartialMaskTokenFilter;
pub use token_filters::PascalCaseTokenFilter;
pub use token_filters::PigLatinTokenFilter;
pub use token_filters::RepeatCharTokenFilter;
pub use token_filters::SlugifyTokenFilter;
pub use token_filters::SnakeCaseTokenFilter;
pub use token_filters::WordReverseTokenFilter;

// Web, Network & Geo
pub use token_filters::DomainExtractTokenFilter;
pub use token_filters::GeohashPrefixTokenFilter;
pub use token_filters::GeohashTokenFilter;
pub use token_filters::IpClassifyTokenFilter;
pub use token_filters::IpNormalizationTokenFilter;
pub use token_filters::IpToNumericTokenFilter;
pub use token_filters::TldExtractTokenFilter;
pub use token_filters::UrlNormalizationTokenFilter;
pub use token_filters::UrlPathTokenFilter;
pub use token_filters::UrlSchemeTokenFilter;

// ═══ Minority & Indigenous Language Filters ══════════════════════════════════
pub use token_filters::HebrewFinalFormNormTokenFilter;
pub use token_filters::HebrewNiqqudRemoveTokenFilter;
pub use token_filters::HebrewStemTokenFilter;
pub use token_filters::YiddishNormalizationTokenFilter;
pub use token_filters::YiddishStemTokenFilter;
pub use token_filters::ScottishGaelicLenitionTokenFilter;
pub use token_filters::ScottishGaelicStopTokenFilter;
pub use token_filters::TibetanTsekSegmentTokenFilter;
pub use token_filters::TibetanPunctuationRemoveTokenFilter;
pub use token_filters::TibetanStopSyllableTokenFilter;
pub use token_filters::WelshMutationNormTokenFilter;
pub use token_filters::WelshStopTokenFilter;
pub use token_filters::CherokeeNormalizationTokenFilter;
pub use token_filters::CherokeeTranslitNormTokenFilter;
pub use token_filters::KhmerWordBoundaryTokenFilter;
pub use token_filters::KhmerSignRemoveTokenFilter;
pub use token_filters::QuechuaStemTokenFilter;
pub use token_filters::QuechuaStopTokenFilter;
pub use token_filters::GuaraniNormalizationTokenFilter;
pub use token_filters::GuaraniStemTokenFilter;
pub use token_filters::GuaraniStopTokenFilter;
pub use token_filters::NavajoNormalizationTokenFilter;
pub use token_filters::NavajoToneRemoveTokenFilter;
pub use token_filters::NavajoStemTokenFilter;
pub use token_filters::NavajoStopTokenFilter;
pub use token_filters::NahuatlStemTokenFilter;
pub use token_filters::NahuatlStopTokenFilter;
pub use token_filters::AymaraStemTokenFilter;
pub use token_filters::AymaraStopTokenFilter;
