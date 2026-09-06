//! Pre-composed language analyzers.
//!
//! Each language analyzer follows ES/Lucene conventions:
//! - Standard tokenizer (or language-specific tokenizer)
//! - Language-specific normalizations
//! - Stop words removal
//! - Language-specific stemming
//!
//! Use [`register_all`] to register all built-in components and analyzers into
//! an [`AnalysisFactory`].

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::Analyzer;
use pizza_engine::analysis::Normalizer;
use pizza_engine::analysis::StandardTokenizer;
use pizza_engine::analysis::TokenFilter;
use pizza_engine::analysis::WhitespaceTokenizer;

use crate::token_filters::stopwords;
use crate::token_filters::*;
use crate::*;

/// Register all analysis-core components into an existing [`AnalysisFactory`].
///
/// This registers:
/// - All tokenizers (keyword, letter, pattern, etc.)
/// - All normalizers (html_strip, mapping, etc.)
/// - All token filters (stop, asciifolding, synonym, stemmers, etc.)
/// - All pre-composed language analyzers (arabic, english, french, etc.)
pub fn register_all(factory: &mut AnalysisFactory) {
    register_tokenizers(factory);
    register_normalizers(factory);
    register_token_filters(factory);
    register_language_analyzers(factory);
}

// ─── Tokenizer Registration ───────────────────────────────────────────────

fn register_tokenizers(factory: &mut AnalysisFactory) {
    factory.register_tokenizer("keyword", Box::new(KeywordTokenizer::new()));
    factory.register_tokenizer("letter", Box::new(LetterTokenizer::new()));
    factory.register_tokenizer("lowercase", Box::new(LowercaseTokenizer::new()));
    factory.register_tokenizer("classic", Box::new(ClassicTokenizer::new()));
    factory.register_tokenizer("uax_url_email", Box::new(UaxUrlEmailTokenizer::new()));
    factory.register_tokenizer("thai", Box::new(ThaiTokenizer::new()));
    factory.register_tokenizer("burmese", Box::new(BurmeseTokenizer::new()));
    factory.register_tokenizer(
        "path_hierarchy",
        Box::new(PathHierarchyTokenizer::default()),
    );
    factory.register_tokenizer("ngram", Box::new(NgramTokenizer::new(1, 2)));
    factory.register_tokenizer("edge_ngram", Box::new(EdgeNgramTokenizer::new(1, 2)));
    factory.register_tokenizer("whitespace", Box::new(WhitespaceTokenizer::new()));
    factory.register_tokenizer(
        "char_group",
        Box::new(CharGroupTokenizer::new(vec![' ', '\t', '\n'])),
    );
    factory.register_tokenizer("standard", Box::new(StandardTokenizer::new()));
    factory.register_tokenizer("pattern", Box::new(PatternTokenizer::default()));
    factory.register_tokenizer(
        "simple_pattern",
        Box::new(SimplePatternTokenizer::new(r"\w+").unwrap()),
    );
    factory.register_tokenizer(
        "simple_pattern_split",
        Box::new(SimplePatternSplitTokenizer::new(r"\s+").unwrap()),
    );

    // New tokenizers
    factory.register_tokenizer("sentence", Box::new(SentenceTokenizer::new()));
    factory.register_tokenizer("camel_case", Box::new(CamelCaseTokenizer::new()));
    factory.register_tokenizer("code", Box::new(CodeTokenizer::new()));
    factory.register_tokenizer("url", Box::new(UrlTokenizer::new()));
    factory.register_tokenizer("compound_word", Box::new(CompoundWordTokenizer::default()));
    factory.register_tokenizer("microblog", Box::new(MicroBlogTokenizer::new()));
    factory.register_tokenizer("structured_id", Box::new(StructuredIdTokenizer::new()));
    factory.register_tokenizer("chinese_char", Box::new(ChineseCharTokenizer::new()));
    factory.register_tokenizer("email", Box::new(EmailTokenizer::new()));
    factory.register_tokenizer("punctuation", Box::new(PunctuationTokenizer::new()));
    factory.register_tokenizer("hyphenated", Box::new(HyphenatedTokenizer::new()));
    factory.register_tokenizer("phone_number", Box::new(PhoneNumberTokenizer::new()));
    factory.register_tokenizer("fingerprint", Box::new(FingerprintTokenizer::new()));
    factory.register_tokenizer("tab_separated", Box::new(TabSeparatedTokenizer::new()));
    factory.register_tokenizer("json_field", Box::new(JsonFieldTokenizer::new()));
    factory.register_tokenizer("wildcard", Box::new(WildcardTokenizer::new()));
    factory.register_tokenizer("emoji", Box::new(EmojiTokenizer::new()));
    factory.register_tokenizer("script_boundary", Box::new(ScriptBoundaryTokenizer::new()));
    factory.register_tokenizer("truncate", Box::new(TruncateTokenizer::default()));
    factory.register_tokenizer("reverse", Box::new(ReverseTokenizer::new()));
    factory.register_tokenizer("elision", Box::new(ElisionTokenizer::default()));
    factory.register_tokenizer("log", Box::new(LogTokenizer::new()));
    factory.register_tokenizer("sliding_window", Box::new(SlidingWindowTokenizer::default()));
    factory.register_tokenizer("markdown", Box::new(MarkdownTokenizer::new()));
}

// ─── Normalizer Registration ──────────────────────────────────────────────

fn register_normalizers(factory: &mut AnalysisFactory) {
    factory.register_normalizer("html_strip", Box::new(HtmlStripNormalizer::new()));
    factory.register_normalizer("trim", Box::new(TrimNormalizer::new()));
    factory.register_normalizer(
        "collapse_whitespace",
        Box::new(CollapseWhitespaceNormalizer::new()),
    );
    factory.register_normalizer("lowercase", Box::new(LowercaseNormalizer::new()));
    factory.register_normalizer("uppercase", Box::new(UppercaseNormalizer::new()));
    factory.register_normalizer("mapping", Box::new(MappingNormalizer::new()));
    factory.register_normalizer(
        "pattern_replace",
        Box::new(PatternReplaceNormalizer::new(r"[^\w\s]", "")),
    );
    factory.register_normalizer(
        "unicode_nfkc",
        Box::new(UnicodeNormalizer::new(UnicodeNormForm::Nfkc)),
    );
    factory.register_normalizer(
        "unicode_nfc",
        Box::new(UnicodeNormalizer::new(UnicodeNormForm::Nfc)),
    );
    factory.register_normalizer(
        "unicode_nfkd",
        Box::new(UnicodeNormalizer::new(UnicodeNormForm::Nfkd)),
    );
    factory.register_normalizer(
        "unicode_nfd",
        Box::new(UnicodeNormalizer::new(UnicodeNormForm::Nfd)),
    );
}

// ─── Token Filter Registration ────────────────────────────────────────────

fn register_token_filters(factory: &mut AnalysisFactory) {
    // Core manipulation filters
    factory.register_token_filter("lowercase", Box::new(LowercaseTokenFilter::new()));
    factory.register_token_filter("uppercase", Box::new(UppercaseTokenFilter::new()));
    factory.register_token_filter("trim", Box::new(TrimTokenFilter::new()));
    factory.register_token_filter("reverse", Box::new(ReverseTokenFilter::new()));
    factory.register_token_filter("asciifolding", Box::new(AsciiFoldingTokenFilter::new()));
    factory.register_token_filter("apostrophe", Box::new(ApostropheTokenFilter::new()));
    factory.register_token_filter("decimal_digit", Box::new(DecimalDigitTokenFilter::new()));
    factory.register_token_filter("classic", Box::new(ClassicTokenFilter::new()));
    factory.register_token_filter("keyword_repeat", Box::new(KeywordRepeatTokenFilter::new()));
    factory.register_token_filter("kstem", Box::new(KStemTokenFilter::new()));
    factory.register_token_filter("unique", Box::new(UniqueTokenFilter::new()));
    factory.register_token_filter(
        "remove_duplicates",
        Box::new(RemoveDuplicatesTokenFilter::new()),
    );
    factory.register_token_filter("flatten_graph", Box::new(FlattenGraphTokenFilter::new()));

    // Length & limit filters
    factory.register_token_filter("length", Box::new(LengthTokenFilter::new(0, 255)));
    factory.register_token_filter("limit", Box::new(LimitTokenFilter::new(1)));
    factory.register_token_filter("truncate", Box::new(TruncateTokenFilter::new(10)));

    // N-gram & shingle
    factory.register_token_filter("ngram", Box::new(NgramTokenFilter::new(1, 2)));
    factory.register_token_filter("edge_ngram", Box::new(EdgeNgramTokenFilter::new(1, 2)));
    factory.register_token_filter("shingle", Box::new(ShingleTokenFilter::new(2, 2)));

    // Word splitting
    factory.register_token_filter(
        "word_delimiter",
        Box::new(WordDelimiterTokenFilter::new(WordDelimiterConfig::default())),
    );
    factory.register_token_filter(
        "word_delimiter_graph",
        Box::new(WordDelimiterGraphTokenFilter::new()),
    );

    // Pattern-based
    factory.register_token_filter(
        "pattern_replace",
        Box::new(PatternReplaceTokenFilter::new(".*", "$0").unwrap()),
    );

    // Fingerprint
    factory.register_token_filter("fingerprint", Box::new(FingerprintTokenFilter::new()));

    // ES-compatible aliases
    factory.register_token_filter("porter_stem", Box::new(KStemTokenFilter::new()));

    // CJK
    factory.register_token_filter("cjk_bigram", Box::new(CjkBigramTokenFilter::new()));
    factory.register_token_filter("cjk_width", Box::new(CjkWidthTokenFilter::new()));

    // Language normalizations
    factory.register_token_filter(
        "arabic_normalization",
        Box::new(ArabicNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "bengali_normalization",
        Box::new(BengaliNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "german_normalization",
        Box::new(GermanNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "hindi_normalization",
        Box::new(HindiNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "indic_normalization",
        Box::new(IndicNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "persian_normalization",
        Box::new(PersianNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "romanian_normalization",
        Box::new(RomanianNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "scandinavian_normalization",
        Box::new(ScandinavianNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "scandinavian_folding",
        Box::new(ScandinavianFoldingTokenFilter::new()),
    );
    factory.register_token_filter(
        "serbian_normalization",
        Box::new(SerbianNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "sorani_normalization",
        Box::new(SoraniNormalizationTokenFilter::new()),
    );

    // Language-specific lowercase
    factory.register_token_filter(
        "greek_lowercase",
        Box::new(GreekLowercaseTokenFilter::new()),
    );
    factory.register_token_filter(
        "irish_lowercase",
        Box::new(IrishLowercaseTokenFilter::new()),
    );
    factory.register_token_filter(
        "turkish_lowercase",
        Box::new(TurkishLowercaseTokenFilter::new()),
    );

    // Language stemmers
    factory.register_token_filter("arabic_stem", Box::new(ArabicStemTokenFilter::new()));
    factory.register_token_filter("bengali_stem", Box::new(BengaliStemTokenFilter::new()));
    factory.register_token_filter("brazilian_stem", Box::new(BrazilianStemTokenFilter::new()));
    factory.register_token_filter("bulgarian_stem", Box::new(BulgarianStemTokenFilter::new()));
    factory.register_token_filter("czech_stem", Box::new(CzechStemTokenFilter::new()));
    factory.register_token_filter("dutch_stem", Box::new(DutchStemTokenFilter::new()));
    factory.register_token_filter(
        "french_light_stem",
        Box::new(FrenchLightStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "french_minimal_stem",
        Box::new(FrenchMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter("galician_stem", Box::new(GalicianStemTokenFilter::new()));
    factory.register_token_filter(
        "galician_minimal_stem",
        Box::new(GalicianMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "german_light_stem",
        Box::new(GermanLightStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "german_minimal_stem",
        Box::new(GermanMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter("greek_stem", Box::new(GreekStemTokenFilter::new()));
    factory.register_token_filter("hindi_stem", Box::new(HindiStemTokenFilter::new()));
    factory.register_token_filter(
        "italian_light_stem",
        Box::new(ItalianLightStemTokenFilter::new()),
    );
    factory.register_token_filter("kannada_stem", Box::new(KannadaStemTokenFilter::new()));
    factory.register_token_filter("latvian_stem", Box::new(LatvianStemTokenFilter::new()));
    factory.register_token_filter(
        "norwegian_light_stem",
        Box::new(NorwegianLightStemTokenFilter::new()),
    );
    factory.register_token_filter("persian_stem", Box::new(PersianStemTokenFilter::new()));
    factory.register_token_filter(
        "portuguese_light_stem",
        Box::new(PortugueseLightStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "russian_light_stem",
        Box::new(RussianLightStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "russian_yo_normalization",
        Box::new(RussianYoNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "armenian_stem",
        Box::new(ArmenianStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "basque_stem",
        Box::new(BasqueStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "catalan_stem",
        Box::new(CatalanStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "croatian_stem",
        Box::new(CroatianStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "estonian_stem",
        Box::new(EstonianStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "lithuanian_stem",
        Box::new(LithuanianStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "polish_stem",
        Box::new(PolishStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "slovak_stem",
        Box::new(SlovakStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "slovenian_stem",
        Box::new(SlovenianStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "swedish_stem",
        Box::new(SwedishStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "ukrainian_stem",
        Box::new(UkrainianStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "spanish_light_stem",
        Box::new(SpanishLightStemTokenFilter::new()),
    );
    factory.register_token_filter("tamil_stem", Box::new(TamilStemTokenFilter::new()));
    factory.register_token_filter("telugu_stem", Box::new(TeluguStemTokenFilter::new()));
    factory.register_token_filter(
        "finnish_light_stem",
        Box::new(FinnishLightStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "hungarian_light_stem",
        Box::new(HungarianLightStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "indonesian_stem",
        Box::new(IndonesianStemTokenFilter::new()),
    );
    // Generic stemmer (defaults to English/KStem)
    factory.register_token_filter(
        "stemmer",
        Box::new(StemmerTokenFilter::new("english").unwrap()),
    );
    // Hyphenated words
    factory.register_token_filter(
        "hyphenated_words",
        Box::new(HyphenatedWordsTokenFilter::new()),
    );
    // Keep types (alpha-only by default)
    factory.register_token_filter("keep_types", Box::new(KeepTypesTokenFilter::alpha_only()));
    // Protected words (empty by default, user configures)
    factory.register_token_filter(
        "protected_words",
        Box::new(ProtectedWordsTokenFilter::new(&[])),
    );
    // Elision
    factory.register_token_filter(
        "elision",
        Box::new(ElisionTokenFilter::new(&[
            "l", "d", "qu", "m", "n", "s", "t",
        ])),
    );

    // New Lucene-parity filters
    factory.register_token_filter("capitalization", Box::new(CapitalizationTokenFilter::new()));
    factory.register_token_filter(
        "codepoint_count",
        Box::new(CodepointCountTokenFilter::new(0, usize::MAX)),
    );
    factory.register_token_filter(
        "concatenate_graph",
        Box::new(ConcatenateGraphTokenFilter::default()),
    );
    factory.register_token_filter("date_recognizer", Box::new(DateRecognizerTokenFilter::new()));
    factory.register_token_filter(
        "delimited_boost",
        Box::new(DelimitedBoostTokenFilter::default()),
    );
    factory.register_token_filter("drop_if_flagged", Box::new(DropIfFlaggedTokenFilter::new("__DROP__")));
    factory.register_token_filter(
        "english_minimal_stem",
        Box::new(EnglishMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter("fixed_shingle", Box::new(FixedShingleTokenFilter::default()));
    factory.register_token_filter(
        "limit_token_offset",
        Box::new(LimitTokenOffsetFilter::new(usize::MAX)),
    );
    factory.register_token_filter(
        "limit_token_position",
        Box::new(LimitTokenPositionFilter::new(u32::MAX)),
    );
    factory.register_token_filter(
        "norwegian_minimal_stem",
        Box::new(NorwegianMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "norwegian_normalization",
        Box::new(NorwegianNormalizationTokenFilter::new()),
    );
    factory.register_token_filter(
        "pattern_keyword_marker",
        Box::new(PatternKeywordMarkerTokenFilter::new("*")),
    );
    factory.register_token_filter(
        "pattern_typing",
        Box::new(PatternTypingTokenFilter::new(&[("*", "default")])),
    );
    factory.register_token_filter(
        "portuguese_minimal_stem",
        Box::new(PortugueseMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "serbian_normalization_regular",
        Box::new(SerbianNormalizationRegularTokenFilter::new()),
    );
    factory.register_token_filter("sorani_stem", Box::new(SoraniStemTokenFilter::new()));
    factory.register_token_filter(
        "spanish_minimal_stem",
        Box::new(SpanishMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "spanish_plural_stem",
        Box::new(SpanishPluralStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "swedish_minimal_stem",
        Box::new(SwedishMinimalStemTokenFilter::new()),
    );
    factory.register_token_filter(
        "type_as_synonym",
        Box::new(TypeAsSynonymTokenFilter::new()),
    );

    // ═══ Beyond-Lucene Innovative Filters ═══════════════════════════════════

    // Emoji filters
    factory.register_token_filter("emoji_to_text", Box::new(EmojiToTextTokenFilter::new()));
    factory.register_token_filter("emoji_remove", Box::new(EmojiRemoveTokenFilter::new()));
    factory.register_token_filter("emoji_extract", Box::new(EmojiExtractTokenFilter::new()));
    factory.register_token_filter("emoji_sentiment", Box::new(EmojiSentimentTokenFilter::new()));
    factory.register_token_filter("emoticon_to_text", Box::new(EmoticonToTextTokenFilter::new()));

    // Encoding & hash filters
    factory.register_token_filter("base64_encode", Box::new(Base64EncodeTokenFilter::new()));
    factory.register_token_filter("base64_decode", Box::new(Base64DecodeTokenFilter::new()));
    factory.register_token_filter("hex_encode", Box::new(HexEncodeTokenFilter::new()));
    factory.register_token_filter("hex_decode", Box::new(HexDecodeTokenFilter::new()));
    factory.register_token_filter("url_encode", Box::new(UrlEncodeTokenFilter::new()));
    factory.register_token_filter("url_decode", Box::new(UrlDecodeTokenFilter::new()));
    factory.register_token_filter("rot13", Box::new(Rot13TokenFilter::new()));
    factory.register_token_filter("fnv_hash", Box::new(FnvHashTokenFilter::new()));
    factory.register_token_filter("crc32", Box::new(Crc32TokenFilter::new()));
    factory.register_token_filter("murmur3", Box::new(MurmurHash3TokenFilter::new()));
    factory.register_token_filter("hash_synonym", Box::new(HashSynonymTokenFilter::new(HashAlgorithm::Fnv1a)));

    // Extraction filters
    factory.register_token_filter("email_extract", Box::new(EmailExtractTokenFilter::new()));
    factory.register_token_filter("url_extract", Box::new(UrlExtractTokenFilter::new()));
    factory.register_token_filter("ip_extract", Box::new(IpExtractTokenFilter::new()));
    factory.register_token_filter("hashtag_extract", Box::new(HashtagExtractTokenFilter::new()));
    factory.register_token_filter("mention_extract", Box::new(MentionExtractTokenFilter::new()));
    factory.register_token_filter("phone_extract", Box::new(PhoneExtractTokenFilter::new()));
    factory.register_token_filter("currency_extract", Box::new(CurrencyExtractTokenFilter::new()));
    factory.register_token_filter("number_extract", Box::new(NumberExtractTokenFilter::new()));
    factory.register_token_filter("email_domain", Box::new(EmailDomainTokenFilter::new()));

    // Security & privacy filters
    factory.register_token_filter("email_mask", Box::new(EmailMaskTokenFilter::new()));
    factory.register_token_filter("credit_card_mask", Box::new(CreditCardMaskTokenFilter::new()));
    factory.register_token_filter("phone_mask", Box::new(PhoneMaskTokenFilter::new()));
    factory.register_token_filter("ip_mask", Box::new(IpMaskTokenFilter::new()));
    factory.register_token_filter("ssn_mask", Box::new(SsnMaskTokenFilter::new()));
    factory.register_token_filter("redact", Box::new(RedactTokenFilter::default()));
    factory.register_token_filter("sql_injection_detect", Box::new(SqlInjectionDetectTokenFilter::new()));
    factory.register_token_filter("xss_detect", Box::new(XssDetectTokenFilter::new()));
    factory.register_token_filter("path_traversal_detect", Box::new(PathTraversalDetectTokenFilter::new()));

    // Text transform filters
    factory.register_token_filter("camel_case", Box::new(CamelCaseTokenFilter::new()));
    factory.register_token_filter("snake_case", Box::new(SnakeCaseTokenFilter::new()));
    factory.register_token_filter("kebab_case", Box::new(KebabCaseTokenFilter::new()));
    factory.register_token_filter("pascal_case", Box::new(PascalCaseTokenFilter::new()));
    factory.register_token_filter("slugify", Box::new(SlugifyTokenFilter::new()));
    factory.register_token_filter("camel_case_split", Box::new(CamelCaseSplitTokenFilter::new()));
    factory.register_token_filter("word_reverse", Box::new(WordReverseTokenFilter::new()));
    factory.register_token_filter("pig_latin", Box::new(PigLatinTokenFilter::new()));
    factory.register_token_filter("repeat_char", Box::new(RepeatCharTokenFilter::default()));
    factory.register_token_filter("collapse_repeats", Box::new(CollapseRepeatsTokenFilter::new(1)));
    factory.register_token_filter("pad", Box::new(PadTokenFilter::default()));
    factory.register_token_filter("partial_mask", Box::new(PartialMaskTokenFilter::default()));

    // Text metrics filters
    factory.register_token_filter("char_count", Box::new(CharCountTokenFilter::new()));
    factory.register_token_filter("byte_length", Box::new(ByteLengthTokenFilter::new()));
    factory.register_token_filter("length_band", Box::new(LengthBandTokenFilter::new()));
    factory.register_token_filter("syllable_count", Box::new(SyllableCountTokenFilter::new()));
    factory.register_token_filter("entropy", Box::new(EntropyTokenFilter::new()));
    factory.register_token_filter("script_tag", Box::new(ScriptTagTokenFilter::new()));
    factory.register_token_filter("language_tag", Box::new(LanguageTagTokenFilter::new()));
    factory.register_token_filter("uuid_detect", Box::new(UuidDetectTokenFilter::new()));
    factory.register_token_filter("entropy_filter", Box::new(EntropyFilterTokenFilter::default()));

    // Web, network & geo filters
    factory.register_token_filter("domain_extract", Box::new(DomainExtractTokenFilter::new()));
    factory.register_token_filter("tld_extract", Box::new(TldExtractTokenFilter::new()));
    factory.register_token_filter("url_scheme", Box::new(UrlSchemeTokenFilter::new()));
    factory.register_token_filter("url_path", Box::new(UrlPathTokenFilter::new()));
    factory.register_token_filter("ip_normalization", Box::new(IpNormalizationTokenFilter::new()));
    factory.register_token_filter("ip_to_numeric", Box::new(IpToNumericTokenFilter::new()));
    factory.register_token_filter("ip_classify", Box::new(IpClassifyTokenFilter::new()));
    factory.register_token_filter("geohash", Box::new(GeohashTokenFilter::default()));
    factory.register_token_filter("geohash_prefix", Box::new(GeohashPrefixTokenFilter::default()));
    factory.register_token_filter("url_normalization", Box::new(UrlNormalizationTokenFilter::new()));

    // Code, log & science filters
    factory.register_token_filter("identifier_split", Box::new(IdentifierSplitTokenFilter::new()));
    factory.register_token_filter("programming_keyword", Box::new(ProgrammingKeywordTokenFilter::new()));
    factory.register_token_filter("log_level", Box::new(LogLevelTokenFilter::new()));
    factory.register_token_filter("key_value_pair", Box::new(KeyValuePairTokenFilter::new()));
    factory.register_token_filter("semver", Box::new(SemverTokenFilter::new()));
    factory.register_token_filter("http_status", Box::new(HttpStatusTokenFilter::new()));
    factory.register_token_filter("error_code", Box::new(ErrorCodeTokenFilter::new()));
    factory.register_token_filter("ansi_strip", Box::new(AnsiStripTokenFilter::new()));
    factory.register_token_filter("isbn_norm", Box::new(IsbnNormTokenFilter::new()));
    factory.register_token_filter("doi_norm", Box::new(DoiNormTokenFilter::new()));
    factory.register_token_filter("file_extension", Box::new(FileExtensionTokenFilter::new()));
    factory.register_token_filter("path_component", Box::new(PathComponentTokenFilter::new()));

    // NLP & social media filters
    factory.register_token_filter("contraction_expand", Box::new(ContractionExpandTokenFilter::new()));
    factory.register_token_filter("abbreviation_expand", Box::new(AbbreviationExpandTokenFilter::new()));
    factory.register_token_filter("hashtag_split", Box::new(HashtagSplitTokenFilter::new()));
    factory.register_token_filter("slang_norm", Box::new(SlangNormTokenFilter::new()));
    factory.register_token_filter("sentence_case", Box::new(SentenceCaseTokenFilter::new()));
    factory.register_token_filter("mention_tag", Box::new(MentionTagTokenFilter::new()));
    factory.register_token_filter("hashtag_tag", Box::new(HashtagTagTokenFilter::new()));
    factory.register_token_filter("stretched_word_norm", Box::new(StretchedWordNormTokenFilter::new()));
    factory.register_token_filter("sentiment_tag", Box::new(SentimentTagTokenFilter::new()));
    factory.register_token_filter("mention_remove", Box::new(MentionRemoveTokenFilter::new()));
    factory.register_token_filter("hashtag_remove", Box::new(HashtagRemoveTokenFilter::new()));
    factory.register_token_filter("shouting_norm", Box::new(ShoutingNormTokenFilter::new()));

    // Advanced unicode filters
    factory.register_token_filter("invisible_char_remove", Box::new(InvisibleCharRemoveTokenFilter::new()));
    factory.register_token_filter("zero_width_remove", Box::new(ZeroWidthRemoveTokenFilter::new()));
    factory.register_token_filter("bidi_strip", Box::new(BiDiStripTokenFilter::new()));
    factory.register_token_filter("confusable_norm", Box::new(ConfusableNormTokenFilter::new()));
    factory.register_token_filter("homoglyph_norm", Box::new(HomoglyphNormTokenFilter::new()));
    factory.register_token_filter("mixed_script_detect", Box::new(MixedScriptDetectTokenFilter::new()));
    factory.register_token_filter("fullwidth_norm", Box::new(FullwidthNormTokenFilter::new()));
    factory.register_token_filter("diacritic_strip", Box::new(DiacriticStripTokenFilter::new()));
    factory.register_token_filter("emoji_presence", Box::new(EmojiPresenceTokenFilter::new()));

    // Number & conversion filters
    factory.register_token_filter("roman_numeral", Box::new(RomanNumeralTokenFilter::new()));
    factory.register_token_filter("ordinal", Box::new(OrdinalTokenFilter::new()));
    factory.register_token_filter("number_norm", Box::new(NumberNormTokenFilter::new()));
    factory.register_token_filter("number_magnitude", Box::new(NumberMagnitudeTokenFilter::new()));
    factory.register_token_filter("hex_to_decimal", Box::new(HexToDecimalTokenFilter::new()));
    factory.register_token_filter("binary_to_decimal", Box::new(BinaryToDecimalTokenFilter::new()));
    factory.register_token_filter("octal_to_decimal", Box::new(OctalToDecimalTokenFilter::new()));
    factory.register_token_filter("numeric_range", Box::new(NumericRangeTokenFilter::new()));
    factory.register_token_filter("file_size_norm", Box::new(FileSizeNormTokenFilter::new()));
    factory.register_token_filter("duration_norm", Box::new(DurationNormTokenFilter::new()));
    factory.register_token_filter("percent_norm", Box::new(PercentNormTokenFilter::new()));

    // ═══ Minority & Indigenous Language Filters ══════════════════════════════════

    // Hebrew
    factory.register_token_filter("hebrew_niqqud_remove", Box::new(HebrewNiqqudRemoveTokenFilter::new()));
    factory.register_token_filter("hebrew_stem", Box::new(HebrewStemTokenFilter::new()));
    factory.register_token_filter("hebrew_final_form_norm", Box::new(HebrewFinalFormNormTokenFilter::new()));

    // Yiddish
    factory.register_token_filter("yiddish_normalization", Box::new(YiddishNormalizationTokenFilter::new()));
    factory.register_token_filter("yiddish_stem", Box::new(YiddishStemTokenFilter::new()));

    // Scottish Gaelic
    factory.register_token_filter("scottish_gaelic_lenition", Box::new(ScottishGaelicLenitionTokenFilter::new()));
    factory.register_token_filter("scottish_gaelic_stop", Box::new(ScottishGaelicStopTokenFilter::new()));

    // Tibetan
    factory.register_token_filter("tibetan_tsek_segment", Box::new(TibetanTsekSegmentTokenFilter::new()));
    factory.register_token_filter("tibetan_punctuation_remove", Box::new(TibetanPunctuationRemoveTokenFilter::new()));
    factory.register_token_filter("tibetan_stop_syllable", Box::new(TibetanStopSyllableTokenFilter::new()));

    // Welsh
    factory.register_token_filter("welsh_mutation_norm", Box::new(WelshMutationNormTokenFilter::new()));
    factory.register_token_filter("welsh_stop", Box::new(WelshStopTokenFilter::new()));

    // Cherokee & Khmer
    factory.register_token_filter("cherokee_normalization", Box::new(CherokeeNormalizationTokenFilter::new()));
    factory.register_token_filter("cherokee_translit_norm", Box::new(CherokeeTranslitNormTokenFilter::new()));
    factory.register_token_filter("khmer_word_boundary", Box::new(KhmerWordBoundaryTokenFilter::new()));
    factory.register_token_filter("khmer_sign_remove", Box::new(KhmerSignRemoveTokenFilter::new()));

    // Quechua & Guarani
    factory.register_token_filter("quechua_stem", Box::new(QuechuaStemTokenFilter::new()));
    factory.register_token_filter("quechua_stop", Box::new(QuechuaStopTokenFilter::new()));
    factory.register_token_filter("guarani_normalization", Box::new(GuaraniNormalizationTokenFilter::new()));
    factory.register_token_filter("guarani_stem", Box::new(GuaraniStemTokenFilter::new()));
    factory.register_token_filter("guarani_stop", Box::new(GuaraniStopTokenFilter::new()));

    // Navajo, Nahuatl & Aymara
    factory.register_token_filter("navajo_normalization", Box::new(NavajoNormalizationTokenFilter::new()));
    factory.register_token_filter("navajo_tone_remove", Box::new(NavajoToneRemoveTokenFilter::default()));
    factory.register_token_filter("navajo_stem", Box::new(NavajoStemTokenFilter::new()));
    factory.register_token_filter("navajo_stop", Box::new(NavajoStopTokenFilter::new()));
    factory.register_token_filter("nahuatl_stem", Box::new(NahuatlStemTokenFilter::new()));
    factory.register_token_filter("nahuatl_stop", Box::new(NahuatlStopTokenFilter::new()));
    factory.register_token_filter("aymara_stem", Box::new(AymaraStemTokenFilter::new()));
    factory.register_token_filter("aymara_stop", Box::new(AymaraStopTokenFilter::new()));
}

// ─── Language Analyzer Registration ───────────────────────────────────────

fn register_language_analyzers(factory: &mut AnalysisFactory) {
    factory.register_analyzer("afrikaans", build_stop_stem_analyzer("afrikaans"));
    factory.register_analyzer("amharic", build_stop_stem_analyzer("amharic"));
    factory.register_analyzer("arabic", build_arabic_analyzer());
    factory.register_analyzer("armenian", build_armenian_analyzer());
    factory.register_analyzer("azerbaijani", build_stop_stem_analyzer("azerbaijani"));
    factory.register_analyzer("basque", build_basque_analyzer());
    factory.register_analyzer("bengali", build_bengali_analyzer());
    factory.register_analyzer("brazilian", build_brazilian_analyzer());
    factory.register_analyzer("bulgarian", build_bulgarian_analyzer());
    factory.register_analyzer("catalan", build_catalan_analyzer());
    factory.register_analyzer("cjk", build_cjk_analyzer());
    factory.register_analyzer("croatian", build_croatian_analyzer());
    factory.register_analyzer("czech", build_czech_analyzer());
    factory.register_analyzer("danish", build_danish_analyzer());
    factory.register_analyzer("dutch", build_dutch_analyzer());
    factory.register_analyzer("english", build_english_analyzer());
    factory.register_analyzer("estonian", build_estonian_analyzer());
    factory.register_analyzer("filipino", build_stop_stem_analyzer("filipino"));
    factory.register_analyzer("finnish", build_finnish_analyzer());
    factory.register_analyzer("french", build_french_analyzer());
    factory.register_analyzer("galician", build_galician_analyzer());
    factory.register_analyzer("georgian", build_stop_stem_analyzer("georgian"));
    factory.register_analyzer("german", build_german_analyzer());
    factory.register_analyzer("greek", build_greek_analyzer());
    factory.register_analyzer("hindi", build_hindi_analyzer());
    factory.register_analyzer("hungarian", build_hungarian_analyzer());
    factory.register_analyzer("indonesian", build_indonesian_analyzer());
    factory.register_analyzer("irish", build_irish_analyzer());
    factory.register_analyzer("italian", build_italian_analyzer());
    factory.register_analyzer("latvian", build_latvian_analyzer());
    factory.register_analyzer("lithuanian", build_lithuanian_analyzer());
    factory.register_analyzer("malay", build_stop_stem_analyzer("malay"));
    factory.register_analyzer("marathi", build_marathi_analyzer());
    factory.register_analyzer("mongolian", build_stop_stem_analyzer("mongolian"));
    factory.register_analyzer("nepali", build_nepali_analyzer());
    factory.register_analyzer("norwegian", build_norwegian_analyzer());
    factory.register_analyzer("persian", build_persian_analyzer());
    factory.register_analyzer("polish", build_polish_analyzer());
    factory.register_analyzer("portuguese", build_portuguese_analyzer());
    factory.register_analyzer("romanian", build_romanian_analyzer());
    factory.register_analyzer("russian", build_russian_analyzer());
    factory.register_analyzer("serbian", build_serbian_analyzer());
    factory.register_analyzer("slovak", build_slovak_analyzer());
    factory.register_analyzer("slovenian", build_slovenian_analyzer());
    factory.register_analyzer("sorani", build_sorani_analyzer());
    factory.register_analyzer("spanish", build_spanish_analyzer());
    factory.register_analyzer("swahili", build_stop_stem_analyzer("swahili"));
    factory.register_analyzer("swedish", build_swedish_analyzer());
    factory.register_analyzer("tagalog", build_stop_stem_analyzer("tagalog"));
    factory.register_analyzer("tamil", build_tamil_analyzer());
    factory.register_analyzer("thai", build_thai_analyzer());
    factory.register_analyzer("turkish", build_turkish_analyzer());
    factory.register_analyzer("ukrainian", build_ukrainian_analyzer());
    factory.register_analyzer("urdu", build_urdu_analyzer());
    factory.register_analyzer("vietnamese", build_stop_stem_analyzer("vietnamese"));
    factory.register_analyzer("fingerprint", build_fingerprint_analyzer());
    factory.register_analyzer("simple", build_simple_analyzer());
    factory.register_analyzer("stop", build_stop_analyzer());
    factory.register_analyzer("keyword", build_keyword_analyzer());
    factory.register_analyzer("pattern", build_pattern_analyzer());
    factory.register_analyzer("whitespace", build_whitespace_analyzer());

    // ═══ Minority & Indigenous Language Analyzers ════════════════════════════════
    factory.register_analyzer("hebrew", build_hebrew_analyzer());
    factory.register_analyzer("yiddish", build_yiddish_analyzer());
    factory.register_analyzer("scottish_gaelic", build_scottish_gaelic_analyzer());
    factory.register_analyzer("tibetan", build_tibetan_analyzer());
    factory.register_analyzer("welsh", build_welsh_analyzer());
    factory.register_analyzer("cherokee", build_cherokee_analyzer());
    factory.register_analyzer("khmer", build_khmer_analyzer());
    factory.register_analyzer("quechua", build_quechua_analyzer());
    factory.register_analyzer("guarani", build_guarani_analyzer());
    factory.register_analyzer("navajo", build_navajo_analyzer());
    factory.register_analyzer("nahuatl", build_nahuatl_analyzer());
    factory.register_analyzer("aymara", build_aymara_analyzer());
}

// ─── Analyzer Builders ────────────────────────────────────────────────────

/// Helper: build a basic analyzer with standard tokenizer + stop words + a named stemmer.
/// Used for languages that only need stop + snowball stem (no special normalization).
fn build_stop_stem_analyzer(lang: &str) -> Analyzer {
    let stop_words = stopwords::get_stop_words(lang).unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop_words)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_armenian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("armenian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(ArmenianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_basque_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("basque").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(BasqueStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_croatian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("croatian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(CroatianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_estonian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("estonian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(EstonianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_lithuanian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("lithuanian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(LithuanianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_polish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("polish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(PolishStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_slovak_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("slovak").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(SlovakStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_slovenian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("slovenian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(SlovenianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_ukrainian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("ukrainian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(UkrainianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_arabic_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("arabic").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(ArabicNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(ArabicStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_bengali_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("bengali").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(IndicNormalizationTokenFilter::new()),
        Box::new(BengaliNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(BengaliStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_brazilian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("brazilian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(BrazilianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_bulgarian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("bulgarian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(BulgarianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_catalan_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("catalan").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(ElisionTokenFilter::new(&["l", "d", "qu", "m", "n", "s"])),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(CatalanStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_cjk_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("cjk").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(CjkWidthTokenFilter::new()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(CjkBigramTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_czech_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("czech").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(CzechStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_danish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("danish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(ScandinavianNormalizationTokenFilter::new()),
        Box::new(ScandinavianFoldingTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_dutch_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("dutch").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(DutchStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_english_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("english").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(ApostropheTokenFilter::new()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(KStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_finnish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("finnish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(FinnishLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_french_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("french").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(ElisionTokenFilter::french()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(FrenchLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_galician_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("galician").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(GalicianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_german_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("german").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(GermanNormalizationTokenFilter::new()),
        Box::new(GermanLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_greek_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("greek").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(GreekLowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(GreekStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_hindi_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("hindi").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(IndicNormalizationTokenFilter::new()),
        Box::new(HindiNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(HindiStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_hungarian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("hungarian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(HungarianLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_indonesian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("indonesian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(IndonesianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_irish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("irish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(IrishElisionTokenFilter::new()),
        Box::new(IrishLowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_italian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("italian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(ElisionTokenFilter::new(&[
            "l", "dell", "all", "dall", "sull", "nell", "un", "quest", "quell",
        ])),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(ItalianLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_latvian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("latvian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(LatvianStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_marathi_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("marathi").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(IndicNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_nepali_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("nepali").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(IndicNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_norwegian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("norwegian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(NorwegianLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_persian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("persian").unwrap_or(&[]);
    let normalizers: Vec<Box<dyn Normalizer>> =
        vec![Box::new(PatternReplaceNormalizer::new(r"\x{200C}", " "))];
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(ArabicNormalizationTokenFilter::new()),
        Box::new(PersianNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(normalizers, Box::new(StandardTokenizer::new()), filters)
}

fn build_portuguese_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("portuguese").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(PortugueseLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_romanian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("romanian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(RomanianNormalizationTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_russian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("russian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(RussianYoNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(RussianLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_serbian_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("serbian").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(SerbianNormalizationTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_sorani_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("sorani").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(SoraniNormalizationTokenFilter::new()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_spanish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("spanish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(SpanishLightStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_swedish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("swedish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(SwedishStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_tamil_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("tamil").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(IndicNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(TamilStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_urdu_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("urdu").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(IndicNormalizationTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_thai_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("thai").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(DecimalDigitTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(ThaiTokenizer::new()), filters)
}

fn build_turkish_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("turkish").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(ApostropheTokenFilter::new()),
        Box::new(TurkishLowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_fingerprint_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("english").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(AsciiFoldingTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(FingerprintTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_simple_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![Box::new(LowercaseTokenFilter::new())];
    Analyzer::new(vec![], Box::new(LetterTokenizer::new()), filters)
}

fn build_stop_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("english").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
    ];
    Analyzer::new(vec![], Box::new(LetterTokenizer::new()), filters)
}

fn build_keyword_analyzer() -> Analyzer {
    Analyzer::new(vec![], Box::new(KeywordTokenizer::new()), vec![])
}

fn build_pattern_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![Box::new(LowercaseTokenFilter::new())];
    Analyzer::new(vec![], Box::new(PatternTokenizer::default()), filters)
}

fn build_whitespace_analyzer() -> Analyzer {
    Analyzer::new(vec![], Box::new(WhitespaceTokenizer::new()), vec![])
}

// ═══ Minority & Indigenous Language Analyzer Builders ════════════════════════

fn build_hebrew_analyzer() -> Analyzer {
    let stop = stopwords::get_stop_words("hebrew").unwrap_or(&[]);
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(HebrewNiqqudRemoveTokenFilter::new()),
        Box::new(HebrewFinalFormNormTokenFilter::new()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(StopTokenFilter::new(stop)),
        Box::new(HebrewStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_yiddish_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(YiddishNormalizationTokenFilter::new()),
        Box::new(YiddishStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_scottish_gaelic_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(ScottishGaelicLenitionTokenFilter::new()),
        Box::new(ScottishGaelicStopTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_tibetan_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(TibetanTsekSegmentTokenFilter::new()),
        Box::new(TibetanPunctuationRemoveTokenFilter::new()),
        Box::new(TibetanStopSyllableTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_welsh_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(WelshMutationNormTokenFilter::new()),
        Box::new(WelshStopTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_cherokee_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(CherokeeNormalizationTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_khmer_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(KhmerWordBoundaryTokenFilter::new()),
        Box::new(KhmerSignRemoveTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_quechua_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(QuechuaStopTokenFilter::new()),
        Box::new(QuechuaStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_guarani_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(GuaraniNormalizationTokenFilter::new()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(GuaraniStopTokenFilter::new()),
        Box::new(GuaraniStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_navajo_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(NavajoNormalizationTokenFilter::new()),
        Box::new(LowercaseTokenFilter::new()),
        Box::new(NavajoStopTokenFilter::new()),
        Box::new(NavajoStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_nahuatl_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(NahuatlStopTokenFilter::new()),
        Box::new(NahuatlStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}

fn build_aymara_analyzer() -> Analyzer {
    let filters: Vec<Box<dyn TokenFilter>> = vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(AymaraStopTokenFilter::new()),
        Box::new(AymaraStemTokenFilter::new()),
    ];
    Analyzer::new(vec![], Box::new(StandardTokenizer::new()), filters)
}
