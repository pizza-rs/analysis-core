//! Built-in token filters.

mod additional_stemmers;
mod apostrophe;
mod arabic_normalization;
mod arabic_stem;
mod asciifolding;
mod beider_morse;
mod bengali_normalization;
mod bengali_stem;
mod brazilian_stem;
mod bulgarian_stem;
mod capitalization;
mod cjk_bigram;
mod cjk_width;
mod classic;
mod codepoint_count;
mod common_grams;
mod common_grams_query;
mod compound;
mod concatenate_graph;
mod condition;
mod date_recognizer;
mod decimal_digit;
mod delimited;
mod delimited_boost;
mod dictionary_stem;
mod drop_if_flagged;
mod dutch_stem;
mod edge_ngram;
mod elision;
mod english_minimal_stem;
mod european_stemmers;
mod fingerprint;
mod finnish_hungarian_indonesian_stemmers;
mod fix_broken_offsets;
mod fixed_shingle;
mod flatten_graph;
mod french_stem;
mod galician_stem;
mod german_normalization;
mod german_stem;
mod greek_lowercase;
mod greek_stem;
mod hindi_normalization;
mod hindi_stem;
mod hunspell;
mod hyphenated_words;
mod hyphenation_decompounder;
mod indic_normalization;
mod irish;
mod keep_types;
mod keep_words;
mod keyword_marker;
mod keyword_repeat;
mod kstem;
mod latvian_stem;
mod length;
mod limit;
mod limit_offset;
mod limit_position;
mod minhash;
mod misc_stemmers;
mod multiplexer;
mod ngram;
mod norwegian_minimal_stem;
mod norwegian_normalization;
mod pattern_capture;
mod pattern_keyword_marker;
mod pattern_replace;
mod pattern_typing;
mod persian_normalization;
mod persian_stem;
mod phone;
mod phonetic;
mod portuguese_minimal_stem;
mod predicate;
mod protected_words;
mod remove_duplicates;
mod reverse;
mod romanian_normalization;
mod russian_normalization;
mod scandinavian;
mod serbian_normalization;
mod serbian_normalization_regular;
mod shingle;
mod sorani_normalization;
mod sorani_stem;
mod south_asian_stemmers;
mod spanish_plural_stem;
mod stemmer;
mod stemmer_override;
mod stop;
pub mod stopwords;
mod swedish_minimal_stem;
mod synonym;
mod trim;
mod truncate;
mod turkish_lowercase;
mod type_as_synonym;
mod unique;
mod word_delimiter;
mod word_delimiter_graph;

// ═══ Innovative Filters ═══════════════════════════════════════
mod advanced_unicode;
mod code_log_science;
mod emoji;
mod encoding_hash;
mod extraction;
mod nlp_social;
mod number_convert;
mod security_privacy;
mod text_metrics;
mod text_transform;
mod web_network_geo;

// ═══ Minority & Indigenous Language Analysis ═════════════════════════════════
mod cherokee_khmer;
mod hebrew_stem;
mod navajo_nahuatl_aymara;
mod quechua_guarani;
mod tibetan;
mod welsh;
mod yiddish_gaelic;

pub use additional_stemmers::ArmenianStemTokenFilter;
pub use additional_stemmers::BasqueStemTokenFilter;
pub use additional_stemmers::CatalanStemTokenFilter;
pub use additional_stemmers::CroatianStemTokenFilter;
pub use additional_stemmers::EstonianStemTokenFilter;
pub use additional_stemmers::LithuanianStemTokenFilter;
pub use additional_stemmers::PolishStemTokenFilter;
pub use additional_stemmers::SlovakStemTokenFilter;
pub use additional_stemmers::SlovenianStemTokenFilter;
pub use additional_stemmers::SwedishStemTokenFilter;
pub use additional_stemmers::UkrainianStemTokenFilter;
pub use apostrophe::ApostropheTokenFilter;
pub use arabic_normalization::ArabicNormalizationTokenFilter;
pub use arabic_stem::ArabicStemTokenFilter;
pub use asciifolding::AsciiFoldingTokenFilter;
pub use beider_morse::BeiderMorseFilter;
pub use beider_morse::BmNameType;
pub use beider_morse::BmRuleType;
pub use bengali_normalization::BengaliNormalizationTokenFilter;
pub use bengali_stem::BengaliStemTokenFilter;
pub use brazilian_stem::BrazilianStemTokenFilter;
pub use bulgarian_stem::BulgarianStemTokenFilter;
pub use capitalization::CapitalizationTokenFilter;
pub use cjk_bigram::CjkBigramTokenFilter;
pub use cjk_width::CjkWidthTokenFilter;
pub use classic::ClassicTokenFilter;
pub use codepoint_count::CodepointCountTokenFilter;
pub use common_grams::CommonGramsTokenFilter;
pub use common_grams_query::CommonGramsQueryFilter;
pub use compound::DictionaryDecompounderTokenFilter;
pub use concatenate_graph::concatenate_tokens;
pub use concatenate_graph::ConcatenateGraphTokenFilter;
pub use condition::ConditionalTokenFilter;
pub use condition::MaxLengthPredicate;
pub use condition::MinLengthPredicate;
pub use condition::PatternPredicate;
pub use condition::TokenPredicate;
pub use date_recognizer::DateRecognizerTokenFilter;
pub use decimal_digit::DecimalDigitTokenFilter;
pub use delimited::DelimitedPayloadTokenFilter;
pub use delimited::DelimitedTermFreqTokenFilter;
pub use delimited_boost::DelimitedBoostTokenFilter;
pub use dictionary_stem::DictionaryStemTokenFilter;
pub use drop_if_flagged::DropIfFlaggedTokenFilter;
pub use edge_ngram::EdgeNgramTokenFilter;
pub use elision::ElisionTokenFilter;
pub use english_minimal_stem::EnglishMinimalStemTokenFilter;
pub use european_stemmers::ItalianLightStemTokenFilter;
pub use european_stemmers::PortugueseLightStemTokenFilter;
pub use european_stemmers::RussianLightStemTokenFilter;
pub use european_stemmers::SpanishLightStemTokenFilter;
pub use fingerprint::FingerprintAccumulator;
pub use fingerprint::FingerprintTokenFilter;
pub use finnish_hungarian_indonesian_stemmers::FinnishLightStemTokenFilter;
pub use finnish_hungarian_indonesian_stemmers::HungarianLightStemTokenFilter;
pub use finnish_hungarian_indonesian_stemmers::IndonesianStemTokenFilter;
pub use fix_broken_offsets::FixBrokenOffsetsFilter;
pub use fixed_shingle::build_fixed_shingles;
pub use fixed_shingle::FixedShingleTokenFilter;
pub use flatten_graph::FlattenGraphTokenFilter;
pub use french_stem::FrenchLightStemTokenFilter;
pub use french_stem::FrenchMinimalStemTokenFilter;
pub use galician_stem::GalicianMinimalStemTokenFilter;
pub use galician_stem::GalicianStemTokenFilter;
pub use german_normalization::GermanNormalizationTokenFilter;
pub use german_stem::GermanLightStemTokenFilter;
pub use german_stem::GermanMinimalStemTokenFilter;
pub use greek_lowercase::GreekLowercaseTokenFilter;
pub use greek_stem::GreekStemTokenFilter;
pub use hindi_normalization::HindiNormalizationTokenFilter;
pub use hindi_stem::HindiStemTokenFilter;
pub use hunspell::AffixRule;
pub use hunspell::HunspellStemFilter;
pub use hyphenation_decompounder::HyphenationDecompounderTokenFilter;
pub use indic_normalization::IndicNormalizationTokenFilter;
pub use irish::IrishElisionTokenFilter;
pub use irish::IrishLowercaseTokenFilter;
pub use keep_words::KeepWordsTokenFilter;
pub use keyword_marker::KeywordMarkerTokenFilter;
pub use keyword_repeat::KeywordRepeatTokenFilter;
pub use kstem::KStemTokenFilter;
pub use latvian_stem::LatvianStemTokenFilter;
pub use length::LengthTokenFilter;
pub use limit::LimitTokenFilter;
pub use limit_offset::LimitTokenOffsetFilter;
pub use limit_position::LimitTokenPositionFilter;
// LowercaseTokenFilter is provided by pizza-engine
pub use dutch_stem::DutchStemTokenFilter;
pub use hyphenated_words::HyphenatedWordsTokenFilter;
pub use keep_types::KeepTypesMode;
pub use keep_types::KeepTypesTokenFilter;
pub use keep_types::TokenType;
pub use minhash::MinHashTokenFilter;
pub use misc_stemmers::CzechStemTokenFilter;
pub use misc_stemmers::NorwegianLightStemTokenFilter;
pub use multiplexer::MultiplexerTokenFilter;
pub use ngram::NgramTokenFilter;
pub use norwegian_minimal_stem::NorwegianMinimalStemTokenFilter;
pub use norwegian_normalization::NorwegianNormalizationTokenFilter;
pub use pattern_capture::PatternCaptureTokenFilter;
pub use pattern_keyword_marker::PatternKeywordMarkerTokenFilter;
pub use pattern_replace::PatternReplaceTokenFilter;
pub use pattern_typing::PatternTypingTokenFilter;
pub use persian_normalization::PersianNormalizationTokenFilter;
pub use persian_stem::PersianStemTokenFilter;
pub use phone::PhoneNumberFilter;
pub use phonetic::PhoneticEncoder;
pub use phonetic::PhoneticTokenFilter;
pub use pizza_engine::analysis::LowercaseTokenFilter;
pub use portuguese_minimal_stem::PortugueseMinimalStemTokenFilter;
pub use predicate::detect_script;
pub use predicate::PredicateTokenFilter;
pub use predicate::ScriptType;
pub use predicate::TokenPredicateType;
pub use protected_words::ProtectedWordsTokenFilter;
pub use remove_duplicates::RemoveDuplicatesState;
pub use remove_duplicates::RemoveDuplicatesTokenFilter;
pub use reverse::ReverseTokenFilter;
pub use romanian_normalization::RomanianNormalizationTokenFilter;
pub use russian_normalization::RussianYoNormalizationTokenFilter;
pub use scandinavian::ScandinavianFoldingTokenFilter;
pub use scandinavian::ScandinavianNormalizationTokenFilter;
pub use serbian_normalization::SerbianNormalizationTokenFilter;
pub use serbian_normalization_regular::SerbianNormalizationRegularTokenFilter;
pub use shingle::ShingleTokenFilter;
pub use sorani_normalization::SoraniNormalizationTokenFilter;
pub use sorani_stem::SoraniStemTokenFilter;
pub use south_asian_stemmers::KannadaStemTokenFilter;
pub use south_asian_stemmers::TamilStemTokenFilter;
pub use south_asian_stemmers::TeluguStemTokenFilter;
pub use spanish_plural_stem::SpanishMinimalStemTokenFilter;
pub use spanish_plural_stem::SpanishPluralStemTokenFilter;
pub use stemmer::StemmerLanguage;
pub use stemmer::StemmerTokenFilter;
pub use stemmer_override::StemmerOverrideTokenFilter;
pub use stop::StopTokenFilter;
pub use swedish_minimal_stem::SwedishMinimalStemTokenFilter;
pub use synonym::SynonymMode;
pub use synonym::SynonymTokenFilter;
pub use trim::TrimTokenFilter;
pub use truncate::TruncateTokenFilter;
pub use turkish_lowercase::TurkishLowercaseTokenFilter;
pub use type_as_synonym::TypeAsSynonymTokenFilter;
pub use unique::UniqueTokenFilter;
// UppercaseTokenFilter is provided by pizza-engine
pub use pizza_engine::analysis::UppercaseTokenFilter;
pub use word_delimiter::WordDelimiterConfig;
pub use word_delimiter::WordDelimiterTokenFilter;
pub use word_delimiter_graph::WordDelimiterGraphTokenFilter;

// ═══ Beyond-Lucene Innovative Filters ═══════════════════════════════════════

// Advanced Unicode
pub use advanced_unicode::BiDiStripTokenFilter;
pub use advanced_unicode::ConfusableNormTokenFilter;
pub use advanced_unicode::DiacriticStripTokenFilter;
pub use advanced_unicode::EmojiPresenceTokenFilter;
pub use advanced_unicode::FullwidthNormTokenFilter;
pub use advanced_unicode::HomoglyphNormTokenFilter;
pub use advanced_unicode::InvisibleCharRemoveTokenFilter;
pub use advanced_unicode::MixedScriptDetectTokenFilter;
pub use advanced_unicode::ZeroWidthRemoveTokenFilter;

// Code, Log & Science
pub use code_log_science::AnsiStripTokenFilter;
pub use code_log_science::DoiNormTokenFilter;
pub use code_log_science::ErrorCodeTokenFilter;
pub use code_log_science::FileExtensionTokenFilter;
pub use code_log_science::HttpStatusTokenFilter;
pub use code_log_science::IdentifierSplitTokenFilter;
pub use code_log_science::IsbnNormTokenFilter;
pub use code_log_science::KeyValuePairTokenFilter;
pub use code_log_science::LogLevelTokenFilter;
pub use code_log_science::PathComponentTokenFilter;
pub use code_log_science::ProgrammingKeywordTokenFilter;
pub use code_log_science::SemverTokenFilter;

// Emoji
pub use emoji::EmojiExtractTokenFilter;
pub use emoji::EmojiRemoveTokenFilter;
pub use emoji::EmojiSentimentTokenFilter;
pub use emoji::EmojiToTextTokenFilter;
pub use emoji::EmoticonToTextTokenFilter;

// Encoding & Hash
pub use encoding_hash::Base64DecodeTokenFilter;
pub use encoding_hash::Base64EncodeTokenFilter;
pub use encoding_hash::Crc32TokenFilter;
pub use encoding_hash::FnvHashTokenFilter;
pub use encoding_hash::HashAlgorithm;
pub use encoding_hash::HashSynonymTokenFilter;
pub use encoding_hash::HexDecodeTokenFilter;
pub use encoding_hash::HexEncodeTokenFilter;
pub use encoding_hash::MurmurHash3TokenFilter;
pub use encoding_hash::Rot13TokenFilter;
pub use encoding_hash::UrlDecodeTokenFilter;
pub use encoding_hash::UrlEncodeTokenFilter;

// Extraction
pub use extraction::CurrencyExtractTokenFilter;
pub use extraction::EmailDomainTokenFilter;
pub use extraction::EmailExtractTokenFilter;
pub use extraction::HashtagExtractTokenFilter;
pub use extraction::IpExtractTokenFilter;
pub use extraction::LookupTokenFilter;
pub use extraction::MentionExtractTokenFilter;
pub use extraction::NumberExtractTokenFilter;
pub use extraction::PhoneExtractTokenFilter;
pub use extraction::UrlExtractTokenFilter;

// NLP & Social
pub use nlp_social::AbbreviationExpandTokenFilter;
pub use nlp_social::ContractionExpandTokenFilter;
pub use nlp_social::HashtagRemoveTokenFilter;
pub use nlp_social::HashtagSplitTokenFilter;
pub use nlp_social::HashtagTagTokenFilter;
pub use nlp_social::MentionRemoveTokenFilter;
pub use nlp_social::MentionTagTokenFilter;
pub use nlp_social::SentenceCaseTokenFilter;
pub use nlp_social::SentimentTagTokenFilter;
pub use nlp_social::ShoutingNormTokenFilter;
pub use nlp_social::SlangNormTokenFilter;
pub use nlp_social::StretchedWordNormTokenFilter;

// Number & Conversion
pub use number_convert::BinaryToDecimalTokenFilter;
pub use number_convert::DurationNormTokenFilter;
pub use number_convert::FileSizeNormTokenFilter;
pub use number_convert::HexToDecimalTokenFilter;
pub use number_convert::NumberMagnitudeTokenFilter;
pub use number_convert::NumberNormTokenFilter;
pub use number_convert::NumericRangeTokenFilter;
pub use number_convert::OctalToDecimalTokenFilter;
pub use number_convert::OrdinalTokenFilter;
pub use number_convert::PercentNormTokenFilter;
pub use number_convert::RomanNumeralTokenFilter;

// Security & Privacy
pub use security_privacy::CreditCardMaskTokenFilter;
pub use security_privacy::EmailMaskTokenFilter;
pub use security_privacy::IpMaskTokenFilter;
pub use security_privacy::PathTraversalDetectTokenFilter;
pub use security_privacy::PhoneMaskTokenFilter;
pub use security_privacy::RedactTokenFilter;
pub use security_privacy::SqlInjectionDetectTokenFilter;
pub use security_privacy::SsnMaskTokenFilter;
pub use security_privacy::XssDetectTokenFilter;

// Text Metrics
pub use text_metrics::ByteLengthTokenFilter;
pub use text_metrics::CharCountTokenFilter;
pub use text_metrics::EntropyFilterTokenFilter;
pub use text_metrics::EntropyTokenFilter;
pub use text_metrics::LanguageTagTokenFilter;
pub use text_metrics::LengthBandTokenFilter;
pub use text_metrics::ScriptTagTokenFilter;
pub use text_metrics::SyllableCountTokenFilter;
pub use text_metrics::UuidDetectTokenFilter;

// Text Transform
pub use text_transform::CamelCaseSplitTokenFilter;
pub use text_transform::CamelCaseTokenFilter;
pub use text_transform::CollapseRepeatsTokenFilter;
pub use text_transform::KebabCaseTokenFilter;
pub use text_transform::PadTokenFilter;
pub use text_transform::PartialMaskTokenFilter;
pub use text_transform::PascalCaseTokenFilter;
pub use text_transform::PigLatinTokenFilter;
pub use text_transform::RepeatCharTokenFilter;
pub use text_transform::SlugifyTokenFilter;
pub use text_transform::SnakeCaseTokenFilter;
pub use text_transform::WordReverseTokenFilter;

// Web, Network & Geo
pub use web_network_geo::DomainExtractTokenFilter;
pub use web_network_geo::GeohashPrefixTokenFilter;
pub use web_network_geo::GeohashTokenFilter;
pub use web_network_geo::IpClassifyTokenFilter;
pub use web_network_geo::IpNormalizationTokenFilter;
pub use web_network_geo::IpToNumericTokenFilter;
pub use web_network_geo::TldExtractTokenFilter;
pub use web_network_geo::UrlNormalizationTokenFilter;
pub use web_network_geo::UrlPathTokenFilter;
pub use web_network_geo::UrlSchemeTokenFilter;

// ═══ Minority & Indigenous Language Filters ══════════════════════════════════

// Hebrew
pub use hebrew_stem::HebrewFinalFormNormTokenFilter;
pub use hebrew_stem::HebrewNiqqudRemoveTokenFilter;
pub use hebrew_stem::HebrewStemTokenFilter;

// Yiddish & Scottish Gaelic
pub use yiddish_gaelic::ScottishGaelicLenitionTokenFilter;
pub use yiddish_gaelic::ScottishGaelicStopTokenFilter;
pub use yiddish_gaelic::YiddishNormalizationTokenFilter;
pub use yiddish_gaelic::YiddishStemTokenFilter;

// Tibetan
pub use tibetan::TibetanPunctuationRemoveTokenFilter;
pub use tibetan::TibetanStopSyllableTokenFilter;
pub use tibetan::TibetanTsekSegmentTokenFilter;

// Welsh
pub use welsh::WelshMutationNormTokenFilter;
pub use welsh::WelshStopTokenFilter;

// Cherokee & Khmer
pub use cherokee_khmer::CherokeeNormalizationTokenFilter;
pub use cherokee_khmer::CherokeeTranslitNormTokenFilter;
pub use cherokee_khmer::KhmerSignRemoveTokenFilter;
pub use cherokee_khmer::KhmerWordBoundaryTokenFilter;

// Quechua & Guarani
pub use quechua_guarani::GuaraniNormalizationTokenFilter;
pub use quechua_guarani::GuaraniStemTokenFilter;
pub use quechua_guarani::GuaraniStopTokenFilter;
pub use quechua_guarani::QuechuaStemTokenFilter;
pub use quechua_guarani::QuechuaStopTokenFilter;

// Navajo, Nahuatl & Aymara
pub use navajo_nahuatl_aymara::AymaraStemTokenFilter;
pub use navajo_nahuatl_aymara::AymaraStopTokenFilter;
pub use navajo_nahuatl_aymara::NahuatlStemTokenFilter;
pub use navajo_nahuatl_aymara::NahuatlStopTokenFilter;
pub use navajo_nahuatl_aymara::NavajoNormalizationTokenFilter;
pub use navajo_nahuatl_aymara::NavajoStemTokenFilter;
pub use navajo_nahuatl_aymara::NavajoStopTokenFilter;
pub use navajo_nahuatl_aymara::NavajoToneRemoveTokenFilter;
