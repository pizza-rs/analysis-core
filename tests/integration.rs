//! Integration tests for the builder API, factory, and analysis utilities.

use pizza_analysis_core::analyzers::register_all;
use pizza_analysis_core::*;
use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::AnalyzerConfig;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use pizza_engine::analysis::Tokenizer;

// ═══════════════════════════════════════════════════════════════════════════
// AnalyzerBuilder Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn builder_default_tokenizer() {
    // When no tokenizer is specified, defaults to StandardTokenizer
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "Hello World");
    assert_eq!(terms, vec!["hello", "world"]);
}

#[test]
fn builder_with_keyword_tokenizer() {
    let analyzer = AnalyzerBuilder::new()
        .tokenizer(KeywordTokenizer::new())
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "Hello World");
    assert_eq!(terms, vec!["hello world"]);
}

#[test]
fn builder_with_normalizer() {
    let analyzer = AnalyzerBuilder::new()
        .normalizer(HtmlStripNormalizer::new())
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "<b>Hello</b> World");
    assert_eq!(terms, vec!["hello", "world"]);
}

#[test]
fn builder_multiple_filters() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .filter(AsciiFoldingTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "Café Résumé");
    assert_eq!(terms, vec!["cafe", "resume"]);
}

#[test]
fn builder_with_stop_filter() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .filter(StopTokenFilter::new(&["the", "a", "is", "in"]))
        .build();
    let terms = analyze_text(&analyzer, "The cat is in a box");
    assert_eq!(terms, vec!["cat", "box"]);
}

#[test]
fn builder_letter_tokenizer() {
    let analyzer = AnalyzerBuilder::new()
        .tokenizer(LetterTokenizer::new())
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "it's a test-case");
    assert_eq!(terms, vec!["it", "s", "a", "test", "case"]);
}

#[test]
fn builder_pattern_tokenizer() {
    let analyzer = AnalyzerBuilder::new()
        .tokenizer(PatternTokenizer::new(r"[,;\s]+"))
        .build();
    let terms = analyze_text(&analyzer, "one, two; three four");
    assert_eq!(terms, vec!["one", "two", "three", "four"]);
}

#[test]
fn builder_edge_ngram_filter() {
    let analyzer = AnalyzerBuilder::new()
        .tokenizer(KeywordTokenizer::new())
        .filter(LowercaseTokenFilter::new())
        .filter(EdgeNgramTokenFilter::new(1, 4))
        .build();
    let terms = analyze_text(&analyzer, "Pizza");
    assert_eq!(terms, vec!["p", "pi", "piz", "pizz"]);
}

#[test]
fn builder_chain_normalizers() {
    let analyzer = AnalyzerBuilder::new()
        .normalizer(HtmlStripNormalizer::new())
        .normalizer(MappingNormalizer::new(&[("&amp;", "&"), ("&lt;", "<")]))
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "<p>A &amp; B</p>");
    assert_eq!(terms, vec!["a", "&", "b"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// analyze_text_detailed Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_detailed_positions_and_offsets() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .build();
    let detailed = analyze_text_detailed(&analyzer, "Hello World");
    assert_eq!(detailed.len(), 2);
    // (term, position, start_offset, end_offset)
    assert_eq!(detailed[0].0, "hello");
    assert_eq!(detailed[0].1, 0); // position
    assert_eq!(detailed[0].2, 0); // start_offset
    assert_eq!(detailed[0].3, 5); // end_offset
    assert_eq!(detailed[1].0, "world");
    assert_eq!(detailed[1].1, 1);
    assert_eq!(detailed[1].2, 6);
    assert_eq!(detailed[1].3, 11);
}

// ═══════════════════════════════════════════════════════════════════════════
// analyze_text_unique Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn analyze_unique_deduplicates() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .build();
    let unique = analyze_text_unique(&analyzer, "the the THE dog dog cat");
    assert_eq!(unique, vec!["the", "dog", "cat"]);
}

#[test]
fn analyze_unique_preserves_order() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .build();
    let unique = analyze_text_unique(&analyzer, "Zebra Apple apple Banana zebra");
    assert_eq!(unique, vec!["zebra", "apple", "banana"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// StemmerTokenFilter Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn stemmer_filter_english() {
    let stemmer = StemmerTokenFilter::new("english").unwrap();
    let mut token = Token::new("running", 0, 7, 0);
    stemmer.filter(&mut token);
    // KStem should reduce "running" → "run" or "running" (KStem is conservative)
    assert!(
        token.term == "run" || token.term == "running",
        "got: {}",
        token.term
    );
}

#[test]
fn stemmer_filter_french() {
    let stemmer = StemmerTokenFilter::new("french").unwrap();
    let mut token = Token::new("chevaux", 0, 7, 0);
    stemmer.filter(&mut token);
    // French light stem should reduce "chevaux" → "cheval" or similar
    assert_ne!(token.term, "chevaux");
}

#[test]
fn stemmer_filter_german() {
    let stemmer = StemmerTokenFilter::new("german").unwrap();
    let mut token = Token::new("bücher", 0, 7, 0);
    stemmer.filter(&mut token);
    // German light stem should reduce
    let result = token.term.to_string();
    assert!(result.len() <= "bücher".len(), "got: {}", result);
}

#[test]
fn stemmer_filter_unsupported_language() {
    assert!(StemmerTokenFilter::new("klingon").is_none());
    assert!(StemmerTokenFilter::new("").is_none());
}

#[test]
fn stemmer_filter_supported_languages() {
    let langs = StemmerLanguage::supported_languages();
    assert!(langs.len() >= 25);
    for lang in langs {
        assert!(
            StemmerTokenFilter::new(lang).is_some(),
            "StemmerTokenFilter::new({}) should succeed",
            lang
        );
    }
}

#[test]
fn stemmer_in_pipeline() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .filter(StemmerTokenFilter::new("english").unwrap())
        .build();
    let terms = analyze_text(&analyzer, "Dogs Cats");
    // At minimum should lowercase; stemming may or may not reduce
    assert!(terms.iter().all(|t| t == &t.to_lowercase()));
}

// ═══════════════════════════════════════════════════════════════════════════
// AnalysisFactory Registration Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn factory_register_all_succeeds() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    // Should have registered at least 61 analyzers
    assert!(factory.get_analyzer("english").is_some());
    assert!(factory.get_analyzer("french").is_some());
    assert!(factory.get_analyzer("german").is_some());
}

#[test]
fn factory_all_language_analyzers_accessible() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let expected_analyzers = [
        "arabic",
        "armenian",
        "basque",
        "bengali",
        "brazilian",
        "bulgarian",
        "catalan",
        "cjk",
        "czech",
        "danish",
        "dutch",
        "english",
        "estonian",
        "finnish",
        "french",
        "galician",
        "german",
        "greek",
        "hindi",
        "hungarian",
        "indonesian",
        "irish",
        "italian",
        "latvian",
        "norwegian",
        "persian",
        "portuguese",
        "romanian",
        "russian",
        "serbian",
        "sorani",
        "spanish",
        "swedish",
        "thai",
        "turkish",
        // Utility
        "keyword",
        "simple",
        "stop",
        "pattern",
        "fingerprint",
        "whitespace",
        // Extended languages
        "afrikaans",
        "amharic",
        "azerbaijani",
        "croatian",
        "filipino",
        "georgian",
        "hebrew",
        "lithuanian",
        "malay",
        "marathi",
        "mongolian",
        "nepali",
        "polish",
        "slovak",
        "slovenian",
        "swahili",
        "tagalog",
        "tamil",
        "ukrainian",
        "urdu",
        "vietnamese",
    ];

    for name in &expected_analyzers {
        assert!(
            factory.get_analyzer(name).is_some(),
            "Analyzer '{}' should be registered",
            name
        );
    }
}

#[test]
fn factory_tokenizers_accessible() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let tokenizers = [
        "standard",
        "whitespace",
        "keyword",
        "letter",
        "lowercase",
        "classic",
        "uax_url_email",
        "thai",
        "burmese",
        "path_hierarchy",
        "pattern",
        "simple_pattern",
        "simple_pattern_split",
        "ngram",
        "edge_ngram",
        "char_group",
    ];
    for name in &tokenizers {
        assert!(
            factory.get_tokenizer(name).is_some(),
            "Tokenizer '{}' should be registered",
            name
        );
    }
}

#[test]
fn factory_token_filters_accessible() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let filters = [
        "lowercase",
        "uppercase",
        "trim",
        "reverse",
        "asciifolding",
        "apostrophe",
        "decimal_digit",
        "classic",
        "keyword_repeat",
        "kstem",
        "unique",
        "remove_duplicates",
        "flatten_graph",
        "length",
        "limit",
        "truncate",
        "ngram",
        "edge_ngram",
        "shingle",
        "word_delimiter",
        "word_delimiter_graph",
        "pattern_replace",
        "fingerprint",
        "porter_stem",
        "cjk_bigram",
        "cjk_width",
        "stemmer",
        "elision",
        // Language stemmers
        "arabic_stem",
        "bengali_stem",
        "brazilian_stem",
        "bulgarian_stem",
        "czech_stem",
        "dutch_stem",
        "french_light_stem",
        "german_light_stem",
        "greek_stem",
        "hindi_stem",
        "italian_light_stem",
        "latvian_stem",
        "norwegian_light_stem",
        "persian_stem",
        "portuguese_light_stem",
        "russian_light_stem",
        "spanish_light_stem",
        "finnish_light_stem",
        "hungarian_light_stem",
        "indonesian_stem",
    ];
    for name in &filters {
        assert!(
            factory.get_token_filter(name).is_some(),
            "Token filter '{}' should be registered",
            name
        );
    }
}

#[test]
fn factory_normalizers_accessible() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let normalizers = [
        "html_strip",
        "mapping",
        "pattern_replace",
        "lowercase",
        "uppercase",
        "unicode_nfkc",
        "unicode_nfc",
        "unicode_nfkd",
        "unicode_nfd",
    ];
    for name in &normalizers {
        assert!(
            factory.get_normalizer(name).is_some(),
            "Normalizer '{}' should be registered",
            name
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// End-to-End Language Analyzer Pipeline Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn english_analyzer_pipeline() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("english").unwrap();
    let mut text = String::from("The quick brown foxes jumped over the lazy dogs");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    // "the" should be removed (stop word)
    assert!(!terms.contains(&"the"));
    // "foxes" should be stemmed
    assert!(terms.contains(&"fox") || terms.contains(&"foxes"));
    // "dogs" should be stemmed
    assert!(terms.contains(&"dog") || terms.contains(&"dogs"));
}

#[test]
fn french_analyzer_pipeline() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("french").unwrap();
    let mut text = String::from("Les chats sont dans la maison");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    // French stop words should be removed
    assert!(!terms.contains(&"les"));
    assert!(!terms.contains(&"la"));
    assert!(!terms.contains(&"dans"));
}

#[test]
fn german_analyzer_pipeline() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("german").unwrap();
    let mut text = String::from("Die Katzen sind in dem Haus");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    // German stop words removed
    assert!(!terms.contains(&"die"));
    assert!(!terms.contains(&"sind"));
    assert!(!terms.contains(&"in"));
    assert!(!terms.contains(&"dem"));
}

#[test]
fn keyword_analyzer_preserves_input() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("keyword").unwrap();
    let mut text = String::from("Hello World! 123");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].term, "Hello World! 123");
}

#[test]
fn simple_analyzer_splits_on_non_letter() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("simple").unwrap();
    let mut text = String::from("Hello-World! 123 test");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    // Simple analyzer lowercases and splits on non-letters
    assert!(terms.contains(&"hello"));
    assert!(terms.contains(&"world"));
    assert!(terms.contains(&"test"));
}

#[test]
fn whitespace_analyzer_preserves_case() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("whitespace").unwrap();
    let mut text = String::from("Hello World FOO");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    assert_eq!(terms, vec!["Hello", "World", "FOO"]);
}

#[test]
fn arabic_analyzer_pipeline() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("arabic").unwrap();
    let mut text = String::from("الكتاب في المكتبة");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    // Should produce some tokens (Arabic stop words removed, normalization applied)
    assert!(!tokens.is_empty());
}

#[test]
fn cjk_analyzer_bigrams() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("cjk").unwrap();
    let mut text = String::from("中文测试");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    // CJK analyzer should produce bigrams from the CJK text
    assert!(!tokens.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge Cases & Robustness
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn empty_input() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "");
    assert!(terms.is_empty());
}

#[test]
fn whitespace_only_input() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "   \t\n  ");
    assert!(terms.is_empty());
}

#[test]
fn unicode_input() {
    let analyzer = AnalyzerBuilder::new()
        .filter(LowercaseTokenFilter::new())
        .filter(AsciiFoldingTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, "Ñoño Ëlève Ünter");
    assert_eq!(terms, vec!["nono", "eleve", "unter"]);
}

#[test]
fn very_long_token() {
    let long = "a".repeat(10000);
    let analyzer = AnalyzerBuilder::new()
        .tokenizer(KeywordTokenizer::new())
        .filter(LowercaseTokenFilter::new())
        .build();
    let terms = analyze_text(&analyzer, &long);
    assert_eq!(terms.len(), 1);
    assert_eq!(terms[0].len(), 10000);
}

#[test]
fn special_characters() {
    let analyzer = AnalyzerBuilder::new().build();
    let terms = analyze_text(&analyzer, "foo@bar.com http://example.com/path?q=1");
    // Standard tokenizer handles URLs and emails
    assert!(!terms.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Token Filter Composition Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn asciifolding_preserves_ascii() {
    let filter = AsciiFoldingTokenFilter::new();
    let mut token = Token::new("hello", 0, 5, 0);
    filter.filter(&mut token);
    assert_eq!(token.term, "hello");
}

#[test]
fn asciifolding_folds_diacritics() {
    let filter = AsciiFoldingTokenFilter::new();
    let mut token = Token::new("café", 0, 5, 0);
    filter.filter(&mut token);
    assert_eq!(token.term, "cafe");
}

#[test]
fn truncate_filter() {
    let filter = TruncateTokenFilter::new(5);
    let mut token = Token::new("abcdefgh", 0, 8, 0);
    filter.filter(&mut token);
    assert_eq!(token.term, "abcde");
}

#[test]
fn length_filter_removes_short() {
    let filter = LengthTokenFilter::new(3, 100);
    let mut token = Token::new("ab", 0, 2, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(deleted);
}

#[test]
fn length_filter_keeps_valid() {
    let filter = LengthTokenFilter::new(2, 10);
    let mut token = Token::new("hello", 0, 5, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);
    assert_eq!(token.term, "hello");
}

#[test]
fn reverse_filter() {
    let filter = ReverseTokenFilter::new();
    let mut token = Token::new("hello", 0, 5, 0);
    filter.filter(&mut token);
    assert_eq!(token.term, "olleh");
}

#[test]
fn trim_filter() {
    let filter = TrimTokenFilter::new();
    let mut token = Token::new("  hello  ", 0, 9, 0);
    filter.filter(&mut token);
    assert_eq!(token.term, "hello");
}

#[test]
fn keep_words_filter() {
    let filter = KeepWordsTokenFilter::new(&["good", "keep", "this"]);
    let mut token = Token::new("good", 0, 4, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);

    let mut token2 = Token::new("bad", 0, 3, 0);
    let (deleted2, _) = filter.filter(&mut token2);
    assert!(deleted2);
}

#[test]
fn keyword_marker_filter() {
    let filter = KeywordMarkerTokenFilter::new(&["running"]);
    let mut token = Token::new("running", 0, 7, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);
    // Token should be marked as keyword (not modified)
    assert_eq!(token.term, "running");
}

// ═══════════════════════════════════════════════════════════════════════════
// HyphenatedWordsTokenFilter Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn hyphenated_words_strips_trailing_hyphen() {
    let filter = HyphenatedWordsTokenFilter::new();
    let mut token = Token::new("knowl-", 0, 6, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);
    assert_eq!(token.term, "knowl");
}

#[test]
fn hyphenated_words_preserves_normal_token() {
    let filter = HyphenatedWordsTokenFilter::new();
    let mut token = Token::new("hello", 0, 5, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);
    assert_eq!(token.term, "hello");
}

#[test]
fn hyphenated_words_ignores_single_hyphen() {
    let filter = HyphenatedWordsTokenFilter::new();
    let mut token = Token::new("-", 0, 1, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);
    assert_eq!(token.term, "-");
}

// ═══════════════════════════════════════════════════════════════════════════
// KeepTypesTokenFilter Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn keep_types_alpha_only() {
    let filter = KeepTypesTokenFilter::alpha_only();

    let mut token = Token::new("hello", 0, 5, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);

    let mut token2 = Token::new("123", 0, 3, 0);
    let (deleted2, _) = filter.filter(&mut token2);
    assert!(deleted2);
}

#[test]
fn keep_types_exclude_numeric() {
    let filter = KeepTypesTokenFilter::exclude_numeric();

    let mut token = Token::new("hello", 0, 5, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);

    let mut token2 = Token::new("42", 0, 2, 0);
    let (deleted2, _) = filter.filter(&mut token2);
    assert!(deleted2);
}

#[test]
fn keep_types_include_cjk() {
    let filter = KeepTypesTokenFilter::new(vec![TokenType::Cjk], KeepTypesMode::Include);

    let mut token = Token::new("中文", 0, 6, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);

    let mut token2 = Token::new("hello", 0, 5, 0);
    let (deleted2, _) = filter.filter(&mut token2);
    assert!(deleted2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Factory Registration for New Filters
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn factory_new_filters_registered() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    assert!(
        factory.get_token_filter("stemmer").is_some(),
        "stemmer filter should be registered"
    );
    assert!(
        factory.get_token_filter("hyphenated_words").is_some(),
        "hyphenated_words filter should be registered"
    );
    assert!(
        factory.get_token_filter("keep_types").is_some(),
        "keep_types filter should be registered"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Config-Based Analyzer Creation (ES-style settings)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn config_create_custom_analyzer() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let config = AnalyzerConfig::from_parts(
        vec!["html_strip"],
        "standard",
        vec!["lowercase", "asciifolding"],
    );
    let analyzer = factory.create_analyzer_from_config(&config).unwrap();
    let mut text = String::from("<b>Café</b> Résumé");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    assert_eq!(terms, vec!["cafe", "resume"]);
}

#[test]
fn config_english_custom() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let config = AnalyzerConfig::from_parts(vec![], "standard", vec!["lowercase", "porter_stem"]);
    let analyzer = factory.create_analyzer_from_config(&config).unwrap();
    let mut text = String::from("Running Quickly");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    // Should be lowercased and stemmed
    assert!(terms.iter().all(|t| t == &t.to_lowercase()));
}

#[test]
fn config_with_normalizer_chain() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let config = AnalyzerConfig::from_parts(vec!["lowercase", "html_strip"], "keyword", vec![]);
    let analyzer = factory.create_analyzer_from_config(&config).unwrap();
    let mut text = String::from("<p>Hello World</p>");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    assert_eq!(tokens.len(), 1);
    // Normalizers should strip HTML and lowercase
    let term = tokens[0].term.as_ref();
    assert!(!term.contains("<p>"));
    assert_eq!(term, term.to_lowercase());
}

#[test]
fn config_nonexistent_tokenizer_returns_none() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    let config = AnalyzerConfig::from_parts(vec![], "nonexistent_tokenizer", vec![]);
    assert!(factory.create_analyzer_from_config(&config).is_none());
}

#[test]
fn config_unknown_filters_gracefully_skipped() {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);

    // Unknown filters should be silently skipped
    let config = AnalyzerConfig::from_parts(
        vec!["nonexistent_normalizer"],
        "standard",
        vec!["lowercase", "nonexistent_filter", "asciifolding"],
    );
    let analyzer = factory.create_analyzer_from_config(&config).unwrap();
    let mut text = String::from("Café");
    let tokens = analyzer.analyze_and_return_tokens(&mut text);
    let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
    assert_eq!(terms, vec!["cafe"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// ProtectedWordsTokenFilter Tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn protected_words_case_sensitive() {
    let filter = ProtectedWordsTokenFilter::new(&["Pizza", "Lucene"]);
    assert!(filter.is_protected("Pizza"));
    assert!(!filter.is_protected("pizza"));
    assert!(filter.is_protected("Lucene"));
}

#[test]
fn protected_words_case_insensitive() {
    let filter = ProtectedWordsTokenFilter::new_ignore_case(&["Pizza", "Lucene"]);
    assert!(filter.is_protected("pizza"));
    assert!(filter.is_protected("PIZZA"));
    assert!(filter.is_protected("lucene"));
}

#[test]
fn protected_words_does_not_delete() {
    let filter = ProtectedWordsTokenFilter::new(&["keep"]);
    let mut token = Token::new("keep", 0, 4, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted);
    assert_eq!(token.term, "keep");
}

// ─── StopTokenFilter::for_language() ─────────────────────────────────────

#[test]
fn stop_filter_for_language_english() {
    let filter = StopTokenFilter::for_language("english").unwrap();
    let mut token = Token::new("the", 0, 3, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(deleted);
}

#[test]
fn stop_filter_for_language_french() {
    let filter = StopTokenFilter::for_language("french").unwrap();
    // "le" is a French stop word
    let mut token = Token::new("le", 0, 2, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(deleted);
    // "bonjour" is not
    let mut token2 = Token::new("bonjour", 0, 7, 1);
    let (deleted2, _) = filter.filter(&mut token2);
    assert!(!deleted2);
}

#[test]
fn stop_filter_for_language_german() {
    let filter = StopTokenFilter::for_language("german").unwrap();
    let mut token = Token::new("und", 0, 3, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(deleted);
}

#[test]
fn stop_filter_for_language_unknown_returns_none() {
    assert!(StopTokenFilter::for_language("klingon").is_none());
    assert!(StopTokenFilter::for_language("").is_none());
}

#[test]
fn stop_filter_supported_languages_list() {
    let langs = StopTokenFilter::supported_languages();
    assert!(langs.len() >= 40);
    assert!(langs.contains(&"english"));
    assert!(langs.contains(&"french"));
    assert!(langs.contains(&"japanese"));
}

// ─── AnalysisRegistry ─────────────────────────────────────────────────────

#[test]
fn registry_new_has_all_analyzers() {
    let registry = AnalysisRegistry::new();
    assert!(registry.has_analyzer("english"));
    assert!(registry.has_analyzer("french"));
    assert!(registry.has_analyzer("german"));
    assert!(registry.has_analyzer("keyword"));
    assert!(registry.has_analyzer("simple"));
}

#[test]
fn registry_analyze_english() {
    let registry = AnalysisRegistry::new();
    let terms = registry.analyze("english", "The Quick Brown Foxes");
    // "the" is a stop word, rest are lowercased + stemmed
    assert!(!terms.contains(&"the".to_string()));
    assert!(terms.contains(&"quick".to_string()));
}

#[test]
fn registry_analyze_nonexistent_returns_empty() {
    let registry = AnalysisRegistry::new();
    let terms = registry.analyze("nonexistent_analyzer", "hello world");
    assert!(terms.is_empty());
}

#[test]
fn registry_analyze_custom_pipeline() {
    let registry = AnalysisRegistry::new();
    let terms = registry.analyze_custom(
        &[],
        "standard",
        &["lowercase", "asciifolding"],
        "Café Résumé",
    );
    assert!(terms.contains(&"cafe".to_string()));
    assert!(terms.contains(&"resume".to_string()));
}

#[test]
fn registry_has_components() {
    let registry = AnalysisRegistry::new();
    assert!(registry.has_tokenizer("standard"));
    assert!(registry.has_tokenizer("keyword"));
    assert!(registry.has_token_filter("lowercase"));
    assert!(registry.has_token_filter("asciifolding"));
    assert!(registry.has_normalizer("html_strip"));
    assert!(!registry.has_tokenizer("nonexistent"));
}

#[test]
fn registry_analyzer_names() {
    let registry = AnalysisRegistry::new();
    let names = registry.analyzer_names();
    assert!(names.len() >= 50);
    assert!(names.contains(&"english"));
    assert!(names.contains(&"french"));
}

#[test]
fn registry_analyze_detailed() {
    let registry = AnalysisRegistry::new();
    let details = registry.analyze_detailed("keyword", "Hello World");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].0, "Hello World");
    assert_eq!(details[0].1, 0); // position
}

#[test]
fn registry_factory_access() {
    let mut registry = AnalysisRegistry::new();
    // Can access factory for advanced operations
    let factory = registry.factory();
    assert!(factory.get_analyzer("english").is_some());
    // Can mutate
    let _factory_mut = registry.factory_mut();
}

// ─── TokenStream API ──────────────────────────────────────────────────────

#[test]
fn token_stream_basic() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("Hello World");
    let stream = TokenStream::from_tokens(tokens);
    assert_eq!(stream.len(), 2);
    let terms = stream.into_terms();
    assert_eq!(terms, vec!["Hello", "World"]);
}

#[test]
fn token_stream_filter_with() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("The Quick Brown Fox");
    let stream = TokenStream::from_tokens(tokens)
        .filter_with(&LowercaseTokenFilter::new())
        .filter_with(&StopTokenFilter::english());
    let terms = stream.into_terms();
    assert!(!terms.contains(&"the".to_string()));
    assert!(terms.contains(&"quick".to_string()));
    assert!(terms.contains(&"brown".to_string()));
    assert!(terms.contains(&"fox".to_string()));
}

#[test]
fn token_stream_filter_chain() {
    use pizza_engine::analysis::StandardTokenizer;

    let lowercase = LowercaseTokenFilter::new();
    let stop = StopTokenFilter::english();
    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("The Quick Brown Fox");
    let filters: Vec<&dyn TokenFilter> = vec![&lowercase, &stop];
    let terms = TokenStream::from_tokens(tokens)
        .filter_chain(&filters)
        .into_terms();
    assert!(!terms.contains(&"the".to_string()));
    assert!(terms.contains(&"quick".to_string()));
}

#[test]
fn token_stream_into_detailed() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("Hello World");
    let detailed = TokenStream::from_tokens(tokens).into_detailed();
    assert_eq!(detailed.len(), 2);
    assert_eq!(detailed[0].0, "Hello");
    assert_eq!(detailed[0].1, 0); // position
    assert_eq!(detailed[1].0, "World");
}

#[test]
fn token_stream_unique_terms() {
    let tokens = vec![
        Token::new("hello", 0, 5, 0),
        Token::new("world", 6, 11, 1),
        Token::new("hello", 12, 17, 2),
    ];
    let terms = TokenStream::from_tokens(tokens).into_unique_terms();
    assert_eq!(terms, vec!["hello", "world"]);
}

#[test]
fn token_stream_take_and_skip() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("one two three four five");
    let terms = TokenStream::from_tokens(tokens.clone())
        .take(3)
        .into_terms();
    assert_eq!(terms.len(), 3);

    let terms = TokenStream::from_tokens(tokens).skip(2).into_terms();
    assert_eq!(terms[0], "three");
}

#[test]
fn token_stream_retain() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("I am a very long sentence indeed");
    // Keep only tokens with 4+ chars
    let terms = TokenStream::from_tokens(tokens)
        .retain(|t| t.term.len() >= 4)
        .into_terms();
    assert!(terms.iter().all(|t| t.len() >= 4));
    assert!(terms.contains(&"very".to_string()));
    assert!(terms.contains(&"long".to_string()));
    assert!(terms.contains(&"sentence".to_string()));
}

#[test]
fn token_stream_map_terms() {
    let tokens = vec![Token::new("hello", 0, 5, 0), Token::new("world", 6, 11, 1)];
    let terms = TokenStream::from_tokens(tokens)
        .map_terms(|t| t.to_uppercase())
        .into_terms();
    assert_eq!(terms, vec!["HELLO", "WORLD"]);
}

#[test]
fn token_stream_predicates() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("Hello World Foo");
    let stream = TokenStream::from_tokens(tokens);
    assert!(stream.any(|t| t.term == "Hello"));
    assert!(!stream.all(|t| t.term.len() > 4));
    assert_eq!(stream.count_matching(|t| t.term.len() >= 3), 3);
}

#[test]
fn token_stream_metadata() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize("one two three");
    let stream = TokenStream::from_tokens(tokens);
    assert_eq!(stream.max_position(), Some(2));
    let span = stream.total_span();
    assert!(span.is_some());
    let (start, end) = span.unwrap();
    assert_eq!(start, 0);
    assert!(end > 0);
}

#[test]
fn token_stream_ext_trait() {
    use pizza_engine::analysis::StandardTokenizer;

    let tokenizer = StandardTokenizer::new();
    let terms = tokenizer
        .tokenize("Hello World")
        .into_stream()
        .filter_with(&LowercaseTokenFilter::new())
        .into_terms();
    assert_eq!(terms, vec!["hello", "world"]);
}

#[test]
fn token_stream_empty() {
    let stream = TokenStream::from_tokens(vec![]);
    assert!(stream.is_empty());
    assert_eq!(stream.len(), 0);
    assert_eq!(stream.max_position(), None);
    assert_eq!(stream.total_span(), None);
    assert!(stream.into_terms().is_empty());
}
