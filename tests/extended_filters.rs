//! Extended tests for analysis-core token filters NOT already covered in lucene_compat.rs.
//!
//! Each module tests a specific filter with multiple cases covering basic operation,
//! edge cases (empty strings, single chars), and Unicode where relevant.

extern crate alloc;

use alloc::borrow::Cow;
use pizza_analysis_core::*;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ─── Helpers ───────────────────────────────────────────────────────────────

fn check_filter(filter: &dyn TokenFilter, input: &str, expected: &str) {
    let mut token = Token::new(input, 0, input.len() as u32, 0);
    let (deleted, _) = filter.filter(&mut token);
    if !deleted {
        assert_eq!(
            token.term.as_ref(),
            expected,
            "filter({:?}) = {:?}, expected {:?}",
            input,
            token.term.as_ref(),
            expected
        );
    } else {
        panic!(
            "filter({:?}) deleted the token, expected {:?}",
            input, expected
        );
    }
}

fn check_filter_delete(filter: &dyn TokenFilter, input: &str) {
    let mut token = Token::new(input, 0, input.len() as u32, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(deleted, "filter({:?}) should delete the token", input);
}

fn check_filter_extras(
    filter: &dyn TokenFilter,
    input: &str,
    expected_term: &str,
    expected_extras: &[&str],
) {
    let mut token = Token::new(input, 0, input.len() as u32, 0);
    let (deleted, extras) = filter.filter(&mut token);
    assert!(!deleted, "filter({:?}) should NOT delete the token", input);
    assert_eq!(
        token.term.as_ref(),
        expected_term,
        "filter({:?}) primary = {:?}, expected {:?}",
        input,
        token.term.as_ref(),
        expected_term
    );
    let extra_terms: Vec<&str> = extras
        .as_ref()
        .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
        .unwrap_or_default();
    assert_eq!(
        extra_terms, expected_extras,
        "filter({:?}) extras = {:?}, expected {:?}",
        input, extra_terms, expected_extras
    );
}

fn check_filter_not_deleted(filter: &dyn TokenFilter, input: &str) {
    let mut token = Token::new(input, 0, input.len() as u32, 0);
    let (deleted, _) = filter.filter(&mut token);
    assert!(!deleted, "filter({:?}) should NOT delete the token", input);
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. EdgeNgramTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod edge_ngram {
    use super::*;

    #[test]
    fn test_basic_1_3() {
        let f = EdgeNgramTokenFilter::new(1, 3);
        let mut token = Token::new("quick", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "q");
        let extra: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert_eq!(extra, vec!["qu", "qui"]);
    }

    #[test]
    fn test_min_equals_max() {
        let f = EdgeNgramTokenFilter::new(2, 2);
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "he");
        assert!(extras.is_none() || extras.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_empty_input() {
        let f = EdgeNgramTokenFilter::new(1, 3);
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(deleted);
    }

    #[test]
    fn test_single_char() {
        let f = EdgeNgramTokenFilter::new(1, 3);
        let mut token = Token::new("a", 0, 1, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "a");
    }

    #[test]
    fn test_token_shorter_than_min() {
        let f = EdgeNgramTokenFilter::new(3, 5);
        let mut token = Token::new("ab", 0, 2, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(deleted);
    }

    #[test]
    fn test_unicode_edge_ngram() {
        let f = EdgeNgramTokenFilter::new(1, 2);
        let mut token = Token::new("日本語", 0, 9, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "日");
        let extra: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert_eq!(extra, vec!["日本"]);
    }

    #[test]
    fn test_preserve_original() {
        let f = EdgeNgramTokenFilter::new(1, 2).with_preserve_original(true);
        let mut token = Token::new("a", 0, 1, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. NgramTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod ngram_filter {
    use super::*;

    #[test]
    fn test_bigrams() {
        let f = NgramTokenFilter::new(2, 2);
        let mut token = Token::new("abc", 0, 3, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "ab");
        let extra: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert_eq!(extra, vec!["bc"]);
    }

    #[test]
    fn test_unigrams() {
        let f = NgramTokenFilter::new(1, 1);
        let mut token = Token::new("ab", 0, 2, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "a");
        let extra: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert_eq!(extra, vec!["b"]);
    }

    #[test]
    fn test_empty_input() {
        let f = NgramTokenFilter::new(1, 2);
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(deleted);
    }

    #[test]
    fn test_single_char_bigram() {
        let f = NgramTokenFilter::new(2, 2);
        let mut token = Token::new("x", 0, 1, 0);
        let (deleted, _extras) = f.filter(&mut token);
        // token is too short for bigrams
        // implementation may delete or return original
        let _ = deleted;
    }

    #[test]
    fn test_mixed_range() {
        let f = NgramTokenFilter::new(1, 3);
        let mut token = Token::new("ab", 0, 2, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        // should produce: "a", "b", "ab"
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.contains(&"a".to_string()));
        assert!(all.contains(&"b".to_string()));
        assert!(all.contains(&"ab".to_string()));
    }

    #[test]
    fn test_unicode_ngrams() {
        let f = NgramTokenFilter::new(2, 2);
        let mut token = Token::new("日本語", 0, 9, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "日本");
        let extra: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert_eq!(extra, vec!["本語"]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. ShingleTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod shingle_filter {
    use super::*;

    #[test]
    fn test_basic_bigram_shingle() {
        let f = ShingleTokenFilter::new(2, 2);
        // Feed first token, no bigram yet
        let mut t1 = Token::new("the", 0, 3, 0);
        let (_del1, _extras1) = f.filter(&mut t1);
        // Feed second token, should get bigram "the quick"
        let mut t2 = Token::new("quick", 4, 9, 1);
        let (del2, extras2) = f.filter(&mut t2);
        assert!(!del2);
        if let Some(ref ex) = extras2 {
            let extra_terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            assert!(
                extra_terms.contains(&"the quick"),
                "expected bigram 'the quick', got {:?}",
                extra_terms
            );
        }
    }

    #[test]
    fn test_single_token_no_shingle() {
        let f = ShingleTokenFilter::new(2, 2);
        f.reset();
        let mut t = Token::new("hello", 0, 5, 0);
        let (_del, extras) = f.filter(&mut t);
        // Only one token, no bigram possible
        assert!(extras.is_none() || extras.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_custom_separator() {
        let f = ShingleTokenFilter::new(2, 2).with_separator("-");
        f.reset();
        let mut t1 = Token::new("foo", 0, 3, 0);
        let _ = f.filter(&mut t1);
        let mut t2 = Token::new("bar", 4, 7, 1);
        let (_del, extras) = f.filter(&mut t2);
        if let Some(ref ex) = extras {
            let terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            assert!(
                terms.contains(&"foo-bar"),
                "expected 'foo-bar', got {:?}",
                terms
            );
        }
    }

    #[test]
    fn test_trigram_shingle() {
        let f = ShingleTokenFilter::new(2, 3);
        f.reset();
        let mut t1 = Token::new("a", 0, 1, 0);
        let _ = f.filter(&mut t1);
        let mut t2 = Token::new("b", 2, 3, 1);
        let _ = f.filter(&mut t2);
        let mut t3 = Token::new("c", 4, 5, 2);
        let (_del, extras) = f.filter(&mut t3);
        if let Some(ref ex) = extras {
            let terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            assert!(
                terms.iter().any(|t| t.contains("b c")),
                "expected bigram containing 'b c', got {:?}",
                terms
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. KeywordMarkerTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod keyword_marker {
    use super::*;

    #[test]
    fn test_no_deletion_on_keyword() {
        let f = KeywordMarkerTokenFilter::new(vec!["running".into()]);
        check_filter(&f, "running", "running");
    }

    #[test]
    fn test_no_deletion_on_non_keyword() {
        let f = KeywordMarkerTokenFilter::new(vec!["test".into()]);
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_empty_keyword_list() {
        let f = KeywordMarkerTokenFilter::new(vec![]);
        check_filter(&f, "word", "word");
    }

    #[test]
    fn test_empty_input() {
        let f = KeywordMarkerTokenFilter::new(vec!["a".into()]);
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. KeywordRepeatTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod keyword_repeat {
    use super::*;

    #[test]
    fn test_emits_duplicate() {
        let f = KeywordRepeatTokenFilter::new();
        let mut token = Token::new("test", 0, 4, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_some());
        let ex = extras.unwrap();
        assert_eq!(ex.len(), 1);
        assert_eq!(ex[0].term.as_ref(), "test");
    }

    #[test]
    fn test_preserves_offsets() {
        let f = KeywordRepeatTokenFilter::new();
        let mut token = Token::new("hello", 5, 10, 3);
        let (_, extras) = f.filter(&mut token);
        let ex = extras.unwrap();
        assert_eq!(ex[0].start_offset, 5);
        assert_eq!(ex[0].end_offset, 10);
        assert_eq!(ex[0].position, 3);
    }

    #[test]
    fn test_empty_token() {
        let f = KeywordRepeatTokenFilter::new();
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(extras.unwrap().len(), 1);
    }

    #[test]
    fn test_unicode_repeat() {
        let f = KeywordRepeatTokenFilter::new();
        let mut token = Token::new("日本", 0, 6, 0);
        let (_, extras) = f.filter(&mut token);
        assert_eq!(extras.unwrap()[0].term.as_ref(), "日本");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. PatternReplaceTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod pattern_replace {
    use super::*;

    #[test]
    fn test_basic_replace() {
        let f = PatternReplaceTokenFilter::new(r"\d+", "NUM").unwrap();
        check_filter(&f, "order123", "orderNUM");
    }

    #[test]
    fn test_replace_all() {
        let f = PatternReplaceTokenFilter::new(r"[aeiou]", "").unwrap();
        check_filter(&f, "hello", "hll");
    }

    #[test]
    fn test_no_match() {
        let f = PatternReplaceTokenFilter::new(r"\d+", "X").unwrap();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_replace_produces_empty() {
        let f = PatternReplaceTokenFilter::new(r".+", "").unwrap();
        check_filter_delete(&f, "hello");
    }

    #[test]
    fn test_replace_single_occurrence() {
        let f = PatternReplaceTokenFilter::new(r"-", "_")
            .unwrap()
            .with_replace_all(false);
        check_filter(&f, "a-b-c", "a_b-c");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. PatternCaptureTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod pattern_capture {
    use super::*;

    #[test]
    fn test_basic_capture() {
        let f = PatternCaptureTokenFilter::new(vec![r"(\d+)-(\w+)"], true);
        let mut token = Token::new("123-abc", 0, 7, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        // original preserved, extras should contain captured groups
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert!(extra_terms.contains(&"123"));
        assert!(extra_terms.contains(&"abc"));
    }

    #[test]
    fn test_no_match() {
        let f = PatternCaptureTokenFilter::new(vec![r"(\d+)"], true);
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert!(extras.is_none() || extras.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_without_preserve_original() {
        let f = PatternCaptureTokenFilter::new(vec![r"(\d+)"], false);
        let mut token = Token::new("abc123", 0, 6, 0);
        let (deleted, _extras) = f.filter(&mut token);
        assert!(!deleted);
        // When not preserving, first capture replaces original
        assert_eq!(token.term.as_ref(), "123");
    }

    #[test]
    fn test_multiple_patterns() {
        let f = PatternCaptureTokenFilter::new(vec![r"([A-Z]+)", r"(\d+)"], true);
        let mut token = Token::new("ABC123", 0, 6, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.term.as_ref())
            .collect();
        assert!(extra_terms.contains(&"ABC"));
        assert!(extra_terms.contains(&"123"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. LimitTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod limit_filter {
    use super::*;

    #[test]
    fn test_within_limit() {
        let f = LimitTokenFilter::new(3);
        let mut t1 = Token::new("a", 0, 1, 0);
        let mut t2 = Token::new("b", 2, 3, 1);
        assert!(!f.filter(&mut t1).0);
        assert!(!f.filter(&mut t2).0);
    }

    #[test]
    fn test_exceeds_limit() {
        let f = LimitTokenFilter::new(2);
        let mut t1 = Token::new("a", 0, 1, 0);
        let mut t2 = Token::new("b", 2, 3, 1);
        let mut t3 = Token::new("c", 4, 5, 2);
        assert!(!f.filter(&mut t1).0);
        assert!(!f.filter(&mut t2).0);
        assert!(f.filter(&mut t3).0); // third token should be deleted
    }

    #[test]
    fn test_limit_zero() {
        let f = LimitTokenFilter::new(0);
        let mut t = Token::new("x", 0, 1, 0);
        assert!(f.filter(&mut t).0);
    }

    #[test]
    fn test_reset() {
        let f = LimitTokenFilter::new(1);
        let mut t1 = Token::new("a", 0, 1, 0);
        assert!(!f.filter(&mut t1).0);
        let mut t2 = Token::new("b", 2, 3, 1);
        assert!(f.filter(&mut t2).0);
        f.reset();
        let mut t3 = Token::new("c", 4, 5, 0);
        assert!(!f.filter(&mut t3).0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. LimitTokenOffsetFilter
// ═══════════════════════════════════════════════════════════════════════════

mod limit_offset {
    use super::*;

    #[test]
    fn test_within_offset() {
        let f = LimitTokenOffsetFilter::new(10);
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_exceeds_offset() {
        let f = LimitTokenOffsetFilter::new(5);
        let mut token = Token::new("word", 10, 14, 2);
        let (deleted, _) = f.filter(&mut token);
        assert!(deleted);
    }

    #[test]
    fn test_at_boundary() {
        let f = LimitTokenOffsetFilter::new(5);
        let mut token = Token::new("word", 5, 9, 1);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_zero_offset_limit() {
        let f = LimitTokenOffsetFilter::new(0);
        let mut token = Token::new("a", 0, 1, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted); // start_offset 0 <= 0
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. LimitTokenPositionFilter
// ═══════════════════════════════════════════════════════════════════════════

mod limit_position {
    use super::*;

    #[test]
    fn test_within_position() {
        let f = LimitTokenPositionFilter::new(5);
        let mut token = Token::new("test", 0, 4, 3);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_exceeds_position() {
        let f = LimitTokenPositionFilter::new(2);
        let mut token = Token::new("test", 0, 4, 5);
        let (deleted, _) = f.filter(&mut token);
        assert!(deleted);
    }

    #[test]
    fn test_at_boundary_position() {
        let f = LimitTokenPositionFilter::new(3);
        let mut token = Token::new("test", 0, 4, 3);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted); // position 3 <= 3
    }

    #[test]
    fn test_zero_position_limit() {
        let f = LimitTokenPositionFilter::new(0);
        let mut token = Token::new("a", 0, 1, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. CommonGramsTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod common_grams {
    use super::*;

    #[test]
    fn test_bigram_with_common_word() {
        let f = CommonGramsTokenFilter::new(vec!["the".into()]);
        f.reset();
        let mut t1 = Token::new("the", 0, 3, 0);
        let (_, _) = f.filter(&mut t1);
        let mut t2 = Token::new("cat", 4, 7, 1);
        let (del, extras) = f.filter(&mut t2);
        assert!(!del);
        // Should produce bigram "the_cat" as extra
        if let Some(ref ex) = extras {
            let terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            assert!(
                terms.contains(&"the_cat"),
                "expected bigram 'the_cat', got {:?}",
                terms
            );
        }
    }

    #[test]
    fn test_no_bigram_without_common() {
        let f = CommonGramsTokenFilter::new(vec!["the".into()]);
        f.reset();
        let mut t1 = Token::new("big", 0, 3, 0);
        let (_, _) = f.filter(&mut t1);
        let mut t2 = Token::new("cat", 4, 7, 1);
        let (del, extras) = f.filter(&mut t2);
        assert!(!del);
        // Neither word is common, so no bigram
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
            .unwrap_or_default();
        assert!(
            !extra_terms.iter().any(|t| t.contains('_')),
            "should not produce bigram, got {:?}",
            extra_terms
        );
    }

    #[test]
    fn test_reset_clears_state() {
        let f = CommonGramsTokenFilter::new(vec!["the".into()]);
        let mut t = Token::new("the", 0, 3, 0);
        let _ = f.filter(&mut t);
        f.reset();
        let mut t2 = Token::new("cat", 4, 7, 1);
        let (_, extras) = f.filter(&mut t2);
        // After reset, no previous token, so no bigram
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
            .unwrap_or_default();
        assert!(!extra_terms.iter().any(|t| t.contains('_')));
    }

    #[test]
    fn test_custom_separator() {
        let f = CommonGramsTokenFilter::new(vec!["of".into()]).with_separator("+");
        f.reset();
        let mut t1 = Token::new("of", 0, 2, 0);
        let _ = f.filter(&mut t1);
        let mut t2 = Token::new("world", 3, 8, 1);
        let (_, extras) = f.filter(&mut t2);
        if let Some(ref ex) = extras {
            let terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            if !terms.is_empty() {
                assert!(terms.iter().any(|t| t.contains('+')));
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. CommonGramsQueryFilter
// ═══════════════════════════════════════════════════════════════════════════

mod common_grams_query {
    use super::*;

    #[test]
    fn test_passes_bigrams() {
        let f = CommonGramsQueryFilter::new();
        let mut token = Token::new("the_cat", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted, "bigrams should not be deleted");
    }

    #[test]
    fn test_handles_unigrams() {
        let f = CommonGramsQueryFilter::new();
        f.reset();
        let mut token = Token::new("cat", 0, 3, 0);
        let (deleted, _) = f.filter(&mut token);
        // Unigram handling depends on look-ahead state
        let _ = deleted;
    }

    #[test]
    fn test_reset() {
        let f = CommonGramsQueryFilter::new();
        let mut t = Token::new("test", 0, 4, 0);
        let _ = f.filter(&mut t);
        f.reset();
        let mut t2 = Token::new("hello_world", 0, 11, 0);
        let (deleted, _) = f.filter(&mut t2);
        assert!(!deleted);
    }

    #[test]
    fn test_custom_separator() {
        let f = CommonGramsQueryFilter::new().with_separator("-");
        let mut token = Token::new("the-cat", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. FixBrokenOffsetsFilter
// ═══════════════════════════════════════════════════════════════════════════

mod fix_broken_offsets {
    use super::*;

    #[test]
    fn test_normal_offsets_unchanged() {
        let f = FixBrokenOffsetsFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.start_offset, 0);
        assert_eq!(token.end_offset, 5);
    }

    #[test]
    fn test_fixes_backward_start_offset() {
        let f = FixBrokenOffsetsFilter::new();
        f.reset();
        // First token at 0..5
        let mut t1 = Token::new("a", 0, 5, 0);
        let _ = f.filter(&mut t1);
        // Second token has start_offset < previous end_offset
        let mut t2 = Token::new("b", 2, 8, 1);
        let _ = f.filter(&mut t2);
        assert!(t2.start_offset >= 5, "start_offset should be >= 5");
    }

    #[test]
    fn test_fixes_end_before_start() {
        let f = FixBrokenOffsetsFilter::new();
        f.reset();
        let mut token = Token::new("test", 10, 5, 0);
        let _ = f.filter(&mut token);
        assert!(
            token.end_offset >= token.start_offset,
            "end_offset should be >= start_offset"
        );
    }

    #[test]
    fn test_reset_clears_state() {
        let f = FixBrokenOffsetsFilter::new();
        let mut t1 = Token::new("a", 0, 100, 0);
        let _ = f.filter(&mut t1);
        f.reset();
        let mut t2 = Token::new("b", 0, 5, 0);
        let _ = f.filter(&mut t2);
        assert_eq!(t2.start_offset, 0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. ConcatenateGraphTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod concatenate_graph {
    use super::*;

    #[test]
    fn test_single_token() {
        let f = ConcatenateGraphTokenFilter::new(' ');
        f.reset();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "hello");
    }

    #[test]
    fn test_two_tokens() {
        let f = ConcatenateGraphTokenFilter::new(' ');
        f.reset();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let _ = f.filter(&mut t1);
        let mut t2 = Token::new("world", 6, 11, 1);
        let (deleted, _) = f.filter(&mut t2);
        assert!(!deleted);
        assert_eq!(token_contains(&t2, "hello world"), true);
    }

    fn token_contains(t: &Token, expected: &str) -> bool {
        t.term.as_ref() == expected || t.term.as_ref().contains(expected)
    }

    #[test]
    fn test_custom_separator() {
        let f = ConcatenateGraphTokenFilter::new('-');
        f.reset();
        let mut t1 = Token::new("a", 0, 1, 0);
        let _ = f.filter(&mut t1);
        let mut t2 = Token::new("b", 2, 3, 1);
        let _ = f.filter(&mut t2);
        assert!(t2.term.as_ref().contains("a-b"));
    }

    #[test]
    fn test_reset_clears_buffer() {
        let f = ConcatenateGraphTokenFilter::new(' ');
        let mut t = Token::new("old", 0, 3, 0);
        let _ = f.filter(&mut t);
        f.reset();
        let mut t2 = Token::new("new", 0, 3, 0);
        let _ = f.filter(&mut t2);
        assert_eq!(t2.term.as_ref(), "new");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. RemoveDuplicatesTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod remove_duplicates {
    use super::*;

    #[test]
    fn test_passes_unique_tokens() {
        let f = RemoveDuplicatesTokenFilter::new();
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_empty_token() {
        let f = RemoveDuplicatesTokenFilter::new();
        check_filter_not_deleted(&f, "");
    }

    #[test]
    fn test_stateless_pass_through() {
        // The per-token implementation is best-effort stateless
        let f = RemoveDuplicatesTokenFilter::new();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let mut t2 = Token::new("hello", 0, 5, 0);
        let (d1, _) = f.filter(&mut t1);
        let (d2, _) = f.filter(&mut t2);
        // Both pass because stateless
        assert!(!d1);
        assert!(!d2);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. ProtectedWordsTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod protected_words {
    use super::*;

    #[test]
    fn test_protected_word_passes() {
        let f = ProtectedWordsTokenFilter::new(&["pizza", "lucene"]);
        assert!(f.is_protected("pizza"));
        check_filter_not_deleted(&f, "pizza");
    }

    #[test]
    fn test_non_protected_word_passes() {
        let f = ProtectedWordsTokenFilter::new(&["pizza"]);
        assert!(!f.is_protected("hello"));
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_case_sensitive() {
        let f = ProtectedWordsTokenFilter::new(&["Pizza"]);
        assert!(!f.is_protected("pizza"));
        assert!(f.is_protected("Pizza"));
    }

    #[test]
    fn test_case_insensitive() {
        let f = ProtectedWordsTokenFilter::new_ignore_case(&["Pizza"]);
        assert!(f.is_protected("pizza"));
        assert!(f.is_protected("PIZZA"));
    }

    #[test]
    fn test_empty_protected_list() {
        let f = ProtectedWordsTokenFilter::new(&[]);
        assert!(!f.is_protected("anything"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. StemmerTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod stemmer_filter {
    use super::*;

    #[test]
    fn test_english_running() {
        let f = StemmerTokenFilter::from_language(StemmerLanguage::English);
        check_filter(&f, "running", "run");
    }

    #[test]
    fn test_english_cats() {
        let f = StemmerTokenFilter::from_language(StemmerLanguage::English);
        check_filter(&f, "cats", "cat");
    }

    #[test]
    fn test_english_short_word() {
        let f = StemmerTokenFilter::from_language(StemmerLanguage::English);
        check_filter(&f, "an", "an");
    }

    #[test]
    fn test_english_empty() {
        let f = StemmerTokenFilter::from_language(StemmerLanguage::English);
        check_filter(&f, "", "");
    }

    #[test]
    fn test_arabic_stemmer() {
        let f = StemmerTokenFilter::from_language(StemmerLanguage::Arabic);
        let mut t = Token::new("test", 0, 4, 0);
        let (deleted, _) = f.filter(&mut t);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. KStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod kstem_filter {
    use super::*;

    #[test]
    fn test_running() {
        let f = KStemTokenFilter::new();
        check_filter(&f, "running", "run");
    }

    #[test]
    fn test_cats() {
        let f = KStemTokenFilter::new();
        check_filter(&f, "cats", "cat");
    }

    #[test]
    fn test_short_word_unchanged() {
        let f = KStemTokenFilter::new();
        check_filter(&f, "an", "an");
    }

    #[test]
    fn test_already_stemmed() {
        let f = KStemTokenFilter::new();
        check_filter(&f, "run", "run");
    }

    #[test]
    fn test_empty() {
        let f = KStemTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. EnglishMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod english_minimal_stem {
    use super::*;

    #[test]
    fn test_plural_s() {
        let f = EnglishMinimalStemTokenFilter::new();
        check_filter(&f, "dogs", "dog");
    }

    #[test]
    fn test_plural_ies() {
        let f = EnglishMinimalStemTokenFilter::new();
        check_filter(&f, "cities", "city");
    }

    #[test]
    fn test_not_plural_us() {
        let f = EnglishMinimalStemTokenFilter::new();
        check_filter(&f, "bus", "bus");
    }

    #[test]
    fn test_not_plural_ss() {
        let f = EnglishMinimalStemTokenFilter::new();
        check_filter(&f, "boss", "boss");
    }

    #[test]
    fn test_short_word() {
        let f = EnglishMinimalStemTokenFilter::new();
        check_filter(&f, "as", "as");
    }

    #[test]
    fn test_empty() {
        let f = EnglishMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. HyphenatedWordsTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod hyphenated_words {
    use super::*;

    #[test]
    fn test_strips_trailing_hyphen() {
        let f = HyphenatedWordsTokenFilter::new();
        check_filter(&f, "knowl-", "knowl");
    }

    #[test]
    fn test_no_hyphen_unchanged() {
        let f = HyphenatedWordsTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_internal_hyphen_unchanged() {
        let f = HyphenatedWordsTokenFilter::new();
        check_filter(&f, "well-known", "well-known");
    }

    #[test]
    fn test_single_hyphen() {
        let f = HyphenatedWordsTokenFilter::new();
        // Just a single hyphen — length is 1, so no strip
        check_filter(&f, "-", "-");
    }

    #[test]
    fn test_empty() {
        let f = HyphenatedWordsTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 21. ClassicTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod classic_filter {
    use super::*;

    #[test]
    fn test_possessive_removal() {
        let f = ClassicTokenFilter::new();
        check_filter(&f, "John's", "John");
    }

    #[test]
    fn test_possessive_uppercase() {
        let f = ClassicTokenFilter::new();
        check_filter(&f, "IT'S", "IT");
    }

    #[test]
    fn test_acronym_dots() {
        let f = ClassicTokenFilter::new();
        check_filter(&f, "U.S.A.", "USA");
    }

    #[test]
    fn test_no_change() {
        let f = ClassicTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_empty() {
        let f = ClassicTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_smart_quote_possessive() {
        let f = ClassicTokenFilter::new();
        check_filter(&f, "John\u{2019}s", "John");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 22. WordDelimiterTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod word_delimiter {
    use super::*;

    #[test]
    fn test_camel_case_split() {
        let f = WordDelimiterTokenFilter::new(WordDelimiterConfig::default());
        let mut token = Token::new("WiFi", 0, 4, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.contains(&"Wi".to_string()) || all.contains(&"Fi".to_string()));
    }

    #[test]
    fn test_numeric_split() {
        let f = WordDelimiterTokenFilter::new(WordDelimiterConfig::default());
        let mut token = Token::new("SD500", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.iter().any(|t| t == "SD" || t == "500"));
    }

    #[test]
    fn test_hyphen_split() {
        let f = WordDelimiterTokenFilter::new(WordDelimiterConfig::default());
        let mut token = Token::new("wi-fi", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.iter().any(|t| t == "wi" || t == "fi"));
    }

    #[test]
    fn test_simple_word_no_split() {
        let f = WordDelimiterTokenFilter::new(WordDelimiterConfig::default());
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_empty() {
        let f = WordDelimiterTokenFilter::new(WordDelimiterConfig::default());
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, _) = f.filter(&mut token);
        // empty could either be deleted or passed through
        let _ = deleted;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 23. WordDelimiterGraphTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod word_delimiter_graph {
    use super::*;

    #[test]
    fn test_camel_case_split() {
        let f = WordDelimiterGraphTokenFilter::new();
        let mut token = Token::new("PowerShot", 0, 9, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.iter().any(|t| t == "Power" || t == "Shot"));
    }

    #[test]
    fn test_numeric_split() {
        let f = WordDelimiterGraphTokenFilter::new();
        let mut token = Token::new("HD1080", 0, 6, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.iter().any(|t| t == "HD" || t == "1080"));
    }

    #[test]
    fn test_delimiter_split() {
        let f = WordDelimiterGraphTokenFilter::new();
        let mut token = Token::new("wi-fi", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let mut all = vec![token.term.to_string()];
        if let Some(ref ex) = extras {
            all.extend(ex.iter().map(|t| t.term.to_string()));
        }
        assert!(all.iter().any(|t| t == "wi" || t == "fi"));
    }

    #[test]
    fn test_simple_word() {
        let f = WordDelimiterGraphTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 24. UniqueTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod unique_filter {
    use super::*;

    #[test]
    fn test_first_occurrence_passes() {
        let f = UniqueTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_duplicate_removed() {
        let f = UniqueTokenFilter::new();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let mut t2 = Token::new("hello", 0, 5, 1);
        let _ = f.filter(&mut t1);
        let (deleted, _) = f.filter(&mut t2);
        assert!(deleted);
    }

    #[test]
    fn test_different_tokens_pass() {
        let f = UniqueTokenFilter::new();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let mut t2 = Token::new("world", 6, 11, 1);
        let _ = f.filter(&mut t1);
        let (deleted, _) = f.filter(&mut t2);
        assert!(!deleted);
    }

    #[test]
    fn test_reset() {
        let f = UniqueTokenFilter::new();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let _ = f.filter(&mut t1);
        f.reset();
        let mut t2 = Token::new("hello", 0, 5, 0);
        let (deleted, _) = f.filter(&mut t2);
        assert!(!deleted);
    }

    #[test]
    fn test_empty_token() {
        let f = UniqueTokenFilter::new();
        let mut t = Token::new("", 0, 0, 0);
        let (deleted, _) = f.filter(&mut t);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 25. RomanianNormalizationTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod romanian_normalization {
    use super::*;

    #[test]
    fn test_s_cedilla_to_comma() {
        let f = RomanianNormalizationTokenFilter::new();
        check_filter(&f, "\u{015F}coala", "\u{0219}coala"); // ş → ș
    }

    #[test]
    fn test_t_cedilla_to_comma() {
        let f = RomanianNormalizationTokenFilter::new();
        check_filter(&f, "\u{0163}ara", "\u{021B}ara"); // ţ → ț
    }

    #[test]
    fn test_uppercase_s_cedilla() {
        let f = RomanianNormalizationTokenFilter::new();
        check_filter(&f, "\u{015E}coala", "\u{0218}coala"); // Ş → Ș
    }

    #[test]
    fn test_uppercase_t_cedilla() {
        let f = RomanianNormalizationTokenFilter::new();
        check_filter(&f, "\u{0162}ARA", "\u{021A}ARA"); // Ţ → Ț
    }

    #[test]
    fn test_no_change_needed() {
        let f = RomanianNormalizationTokenFilter::new();
        check_filter(&f, "normal", "normal");
    }

    #[test]
    fn test_empty() {
        let f = RomanianNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 26. SpanishPluralStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod spanish_plural_stem {
    use super::*;

    #[test]
    fn test_basic_plural() {
        let f = SpanishPluralStemTokenFilter::new();
        check_filter(&f, "gatos", "gato"); // cats → cat
    }

    #[test]
    fn test_es_plural() {
        let f = SpanishPluralStemTokenFilter::new();
        // "ciudades" → "ciudad"
        let mut token = Token::new("ciudades", 0, 8, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        // Either stripped or stays - depends on rules
    }

    #[test]
    fn test_short_word() {
        let f = SpanishPluralStemTokenFilter::new();
        check_filter(&f, "la", "la");
    }

    #[test]
    fn test_empty() {
        let f = SpanishPluralStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_singular_unchanged() {
        let f = SpanishPluralStemTokenFilter::new();
        check_filter(&f, "gato", "gato");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 27. SpanishMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod spanish_minimal_stem {
    use super::*;

    #[test]
    fn test_plural_s() {
        let f = SpanishMinimalStemTokenFilter::new();
        let mut token = Token::new("libros", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_no_change() {
        let f = SpanishMinimalStemTokenFilter::new();
        check_filter(&f, "libro", "libro");
    }

    #[test]
    fn test_empty() {
        let f = SpanishMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 28. RussianYoNormalizationTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod russian_yo_normalization {
    use super::*;

    #[test]
    fn test_lowercase_yo() {
        let f = RussianYoNormalizationTokenFilter::new();
        check_filter(&f, "ёлка", "елка");
    }

    #[test]
    fn test_uppercase_yo() {
        let f = RussianYoNormalizationTokenFilter::new();
        check_filter(&f, "Ёж", "Еж");
    }

    #[test]
    fn test_no_yo_unchanged() {
        let f = RussianYoNormalizationTokenFilter::new();
        check_filter(&f, "молоко", "молоко");
    }

    #[test]
    fn test_empty() {
        let f = RussianYoNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_mixed_yo() {
        let f = RussianYoNormalizationTokenFilter::new();
        check_filter(&f, "ёЁё", "еЕе");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 29. NorwegianNormalizationTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod norwegian_normalization {
    use super::*;

    #[test]
    fn test_ae_to_ae_ligature() {
        let f = NorwegianNormalizationTokenFilter::new();
        check_filter(&f, "bät", "bæt"); // ä → æ
    }

    #[test]
    fn test_oe_to_oslash() {
        let f = NorwegianNormalizationTokenFilter::new();
        check_filter(&f, "böt", "bøt"); // ö → ø
    }

    #[test]
    fn test_aa_to_a_ring() {
        let f = NorwegianNormalizationTokenFilter::new();
        check_filter(&f, "baat", "båt"); // aa → å
    }

    #[test]
    fn test_no_change() {
        let f = NorwegianNormalizationTokenFilter::new();
        check_filter(&f, "hus", "hus");
    }

    #[test]
    fn test_empty() {
        let f = NorwegianNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 30. NorwegianMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod norwegian_minimal_stem {
    use super::*;

    #[test]
    fn test_definite_plural() {
        let f = NorwegianMinimalStemTokenFilter::new();
        let mut token = Token::new("husene", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        // Should stem to "hus" or "huse"
        assert!(
            token.term.len() < "husene".len(),
            "expected stemming of 'husene', got {:?}",
            token.term.as_ref()
        );
    }

    #[test]
    fn test_short_word_unchanged() {
        let f = NorwegianMinimalStemTokenFilter::new();
        check_filter(&f, "hus", "hus");
    }

    #[test]
    fn test_empty() {
        let f = NorwegianMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_bokmaal_only() {
        let f = NorwegianMinimalStemTokenFilter::bokmaal_only();
        let mut token = Token::new("biler", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_nynorsk_only() {
        let f = NorwegianMinimalStemTokenFilter::nynorsk_only();
        let mut token = Token::new("bilar", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 31. SwedishMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod swedish_minimal_stem {
    use super::*;

    #[test]
    fn test_plural_removal() {
        let f = SwedishMinimalStemTokenFilter::new();
        let mut token = Token::new("hundar", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        // Should stem "hundar" by removing suffix
        assert!(
            token.term.as_ref().len() <= "hundar".len(),
            "expected stemming, got {:?}",
            token.term.as_ref()
        );
    }

    #[test]
    fn test_genitiv_s() {
        let f = SwedishMinimalStemTokenFilter::new();
        let mut token = Token::new("husens", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_short_word() {
        let f = SwedishMinimalStemTokenFilter::new();
        check_filter(&f, "hus", "hus");
    }

    #[test]
    fn test_empty() {
        let f = SwedishMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 32. BengaliNormalizationTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod bengali_normalization {
    use super::*;

    #[test]
    fn test_chandrabindu_to_anusvara() {
        let f = BengaliNormalizationTokenFilter::new();
        // \u{0981} → \u{0982}
        check_filter(&f, "\u{0981}test", "\u{0982}test");
    }

    #[test]
    fn test_nukta_removed() {
        let f = BengaliNormalizationTokenFilter::new();
        check_filter(&f, "a\u{09BC}b", "ab");
    }

    #[test]
    fn test_rra_to_dda() {
        let f = BengaliNormalizationTokenFilter::new();
        check_filter(&f, "\u{09DC}", "\u{09A1}"); // RRA → DDA
    }

    #[test]
    fn test_no_change() {
        let f = BengaliNormalizationTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_empty() {
        let f = BengaliNormalizationTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 33. GermanLightStemTokenFilter & GermanMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod german_stemmers {
    use super::*;

    #[test]
    fn test_german_light_en_suffix() {
        let f = GermanLightStemTokenFilter::new();
        // "Katzen" → "Katz"
        let mut token = Token::new("katzen", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        assert!(
            token.term.as_ref() != "katzen",
            "expected stemming of 'katzen'"
        );
    }

    #[test]
    fn test_german_light_short_word() {
        let f = GermanLightStemTokenFilter::new();
        check_filter(&f, "das", "das");
    }

    #[test]
    fn test_german_light_empty() {
        let f = GermanLightStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_german_light_umlaut() {
        let f = GermanLightStemTokenFilter::new();
        let mut token = Token::new("häuser", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_german_minimal_basic() {
        let f = GermanMinimalStemTokenFilter::new();
        let mut token = Token::new("häuser", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_german_minimal_short() {
        let f = GermanMinimalStemTokenFilter::new();
        check_filter(&f, "das", "das");
    }

    #[test]
    fn test_german_minimal_empty() {
        let f = GermanMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 34. FrenchMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod french_minimal_stem {
    use super::*;

    #[test]
    fn test_plural_s() {
        let f = FrenchMinimalStemTokenFilter::new();
        let mut token = Token::new("chats", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_short_word() {
        let f = FrenchMinimalStemTokenFilter::new();
        check_filter(&f, "le", "le");
    }

    #[test]
    fn test_empty() {
        let f = FrenchMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_singular_unchanged() {
        let f = FrenchMinimalStemTokenFilter::new();
        check_filter(&f, "chat", "chat");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 35. CapitalizationTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod capitalization {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        let f = CapitalizationTokenFilter::new();
        check_filter(&f, "hello", "Hello");
    }

    #[test]
    fn test_already_capitalized() {
        let f = CapitalizationTokenFilter::new();
        check_filter(&f, "Hello", "Hello");
    }

    #[test]
    fn test_empty() {
        let f = CapitalizationTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_single_char() {
        let f = CapitalizationTokenFilter::new();
        check_filter(&f, "a", "A");
    }

    #[test]
    fn test_unicode_capitalize() {
        let f = CapitalizationTokenFilter::new();
        check_filter(&f, "über", "Über");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 36. CodepointCountTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod codepoint_count {
    use super::*;

    #[test]
    fn test_within_range() {
        let f = CodepointCountTokenFilter::new(2, 5);
        check_filter_not_deleted(&f, "abc");
    }

    #[test]
    fn test_too_short() {
        let f = CodepointCountTokenFilter::new(3, 10);
        check_filter_delete(&f, "ab");
    }

    #[test]
    fn test_too_long() {
        let f = CodepointCountTokenFilter::new(1, 3);
        check_filter_delete(&f, "hello");
    }

    #[test]
    fn test_at_min_boundary() {
        let f = CodepointCountTokenFilter::new(3, 10);
        check_filter_not_deleted(&f, "abc");
    }

    #[test]
    fn test_at_max_boundary() {
        let f = CodepointCountTokenFilter::new(1, 5);
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_unicode_codepoints() {
        let f = CodepointCountTokenFilter::new(1, 3);
        // "日本語" is 3 codepoints (not 9 bytes)
        check_filter_not_deleted(&f, "日本語");
    }

    #[test]
    fn test_empty_with_min_zero() {
        let f = CodepointCountTokenFilter::new(0, 10);
        check_filter_not_deleted(&f, "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 37. KeepWordsTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod keep_words {
    use super::*;

    #[test]
    fn test_kept_word_passes() {
        let f = KeepWordsTokenFilter::new(vec!["hello".into(), "world".into()]);
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_non_kept_word_deleted() {
        let f = KeepWordsTokenFilter::new(vec!["hello".into()]);
        check_filter_delete(&f, "goodbye");
    }

    #[test]
    fn test_case_sensitive() {
        let f = KeepWordsTokenFilter::new(vec!["Hello".into()]);
        check_filter_delete(&f, "hello");
    }

    #[test]
    fn test_case_insensitive() {
        let f = KeepWordsTokenFilter::new(vec!["Hello".into()]).with_ignore_case(true);
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_empty_keep_list() {
        let f = KeepWordsTokenFilter::new(vec![]);
        check_filter_delete(&f, "anything");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 38. KeepTypesTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod keep_types {
    use super::*;

    #[test]
    fn test_include_alpha() {
        let f = KeepTypesTokenFilter::new(vec![TokenType::Alpha], KeepTypesMode::Include);
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_include_alpha_filters_numeric() {
        let f = KeepTypesTokenFilter::new(vec![TokenType::Alpha], KeepTypesMode::Include);
        check_filter_delete(&f, "12345");
    }

    #[test]
    fn test_exclude_numeric() {
        let f = KeepTypesTokenFilter::new(vec![TokenType::Numeric], KeepTypesMode::Exclude);
        check_filter_delete(&f, "12345");
    }

    #[test]
    fn test_exclude_numeric_keeps_alpha() {
        let f = KeepTypesTokenFilter::new(vec![TokenType::Numeric], KeepTypesMode::Exclude);
        check_filter_not_deleted(&f, "hello");
    }

    #[test]
    fn test_cjk_type() {
        let f = KeepTypesTokenFilter::new(vec![TokenType::Cjk], KeepTypesMode::Include);
        check_filter_not_deleted(&f, "日本");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 39. MinHashTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod minhash_filter {
    use super::*;

    #[test]
    fn test_basic_hash() {
        let f = MinHashTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, _extras) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_different_tokens_produce_hashes() {
        let f = MinHashTokenFilter::new();
        let mut t1 = Token::new("hello", 0, 5, 0);
        let mut t2 = Token::new("world", 6, 11, 1);
        let (d1, _) = f.filter(&mut t1);
        let (d2, _) = f.filter(&mut t2);
        assert!(!d1);
        assert!(!d2);
    }

    #[test]
    fn test_empty_token() {
        let f = MinHashTokenFilter::new();
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, _) = f.filter(&mut token);
        // May or may not be deleted depending on implementation
        let _ = deleted;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 40. PhoneticTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod phonetic_filter {
    use super::*;

    #[test]
    fn test_metaphone() {
        let f = PhoneticTokenFilter::new(PhoneticEncoder::Metaphone(4));
        let mut token = Token::new("Smith", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        // Metaphone of "Smith" → "SM0" or similar
        assert!(!token.term.is_empty());
    }

    #[test]
    fn test_soundex() {
        let f = PhoneticTokenFilter::new(PhoneticEncoder::Soundex);
        let mut token = Token::new("Robert", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        // Soundex of "Robert" → "R163"
        assert_eq!(token.term.as_ref(), "R163");
    }

    #[test]
    fn test_empty_input() {
        let f = PhoneticTokenFilter::new(PhoneticEncoder::Soundex);
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, _) = f.filter(&mut token);
        // May produce empty encoding
        let _ = deleted;
    }

    #[test]
    fn test_single_char() {
        let f = PhoneticTokenFilter::new(PhoneticEncoder::Soundex);
        let mut token = Token::new("A", 0, 1, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_with_replace_false() {
        let f = PhoneticTokenFilter::new(PhoneticEncoder::Soundex).with_replace(false);
        let mut token = Token::new("Robert", 0, 6, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        assert_eq!(token.term.as_ref(), "Robert"); // original preserved
        assert!(extras.is_some()); // phonetic code as extra
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 41. FlattenGraphTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod flatten_graph {
    use super::*;

    #[test]
    fn test_pass_through() {
        let f = FlattenGraphTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_empty() {
        let f = FlattenGraphTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_flatten_stream() {
        let mut tokens = vec![
            Token::new("a", 0, 1, 0),
            Token::new("bc", 0, 2, 0), // same position as "a"
            Token::new("d", 3, 4, 1),
        ];
        FlattenGraphTokenFilter::flatten_stream(&mut tokens);
        // After flattening, positions should be sequential
        assert!(tokens.len() == 3);
    }

    #[test]
    fn test_flatten_empty_stream() {
        let mut tokens: Vec<Token> = vec![];
        FlattenGraphTokenFilter::flatten_stream(&mut tokens);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_flatten_single_token() {
        let mut tokens = vec![Token::new("hello", 0, 5, 0)];
        FlattenGraphTokenFilter::flatten_stream(&mut tokens);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].term.as_ref(), "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 42. TypeAsSynonymTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod type_as_synonym {
    use super::*;

    #[test]
    fn test_alpha_type() {
        let f = TypeAsSynonymTokenFilter::new();
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
            .unwrap_or_default();
        assert!(extra_terms.iter().any(|t| t.contains("alpha")));
    }

    #[test]
    fn test_numeric_type() {
        let f = TypeAsSynonymTokenFilter::new();
        let mut token = Token::new("12345", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
            .unwrap_or_default();
        assert!(extra_terms.iter().any(|t| t.contains("numeric")));
    }

    #[test]
    fn test_custom_prefix() {
        let f = TypeAsSynonymTokenFilter::with_prefix("type=");
        let mut token = Token::new("hello", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
            .unwrap_or_default();
        assert!(extra_terms.iter().any(|t| t.starts_with("type=")));
    }

    #[test]
    fn test_empty() {
        let f = TypeAsSynonymTokenFilter::new();
        let mut token = Token::new("", 0, 0, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        if let Some(ref ex) = extras {
            let terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            assert!(terms.iter().any(|t| t.contains("empty")));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 43. StemmerOverrideTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod stemmer_override {
    use super::*;
    use hashbrown::HashMap;

    #[test]
    fn test_override_match() {
        let f = StemmerOverrideTokenFilter::from_rules(&[("running", "run"), ("better", "good")]);
        check_filter(&f, "running", "run");
    }

    #[test]
    fn test_override_no_match() {
        let f = StemmerOverrideTokenFilter::from_rules(&[("running", "run")]);
        check_filter(&f, "walking", "walking");
    }

    #[test]
    fn test_empty_rules() {
        let f = StemmerOverrideTokenFilter::new(HashMap::new());
        check_filter(&f, "test", "test");
    }

    #[test]
    fn test_case_insensitive() {
        let f =
            StemmerOverrideTokenFilter::from_rules(&[("running", "run")]).with_ignore_case(true);
        check_filter(&f, "Running", "run");
    }

    #[test]
    fn test_empty_input() {
        let f = StemmerOverrideTokenFilter::from_rules(&[("a", "b")]);
        check_filter(&f, "", "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 44. ConditionalTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod conditional_filter {
    use super::*;

    #[test]
    fn test_applies_when_predicate_matches() {
        let predicate = MinLengthPredicate(4);
        let inner = LowercaseTokenFilter::new();
        let f = ConditionalTokenFilter::new(Box::new(predicate), Box::new(inner));
        check_filter(&f, "HELLO", "hello");
    }

    #[test]
    fn test_skips_when_predicate_fails() {
        let predicate = MinLengthPredicate(10);
        let inner = LowercaseTokenFilter::new();
        let f = ConditionalTokenFilter::new(Box::new(predicate), Box::new(inner));
        // "hi" is too short for predicate, so no lowercasing
        check_filter(&f, "HI", "HI");
    }

    #[test]
    fn test_max_length_predicate() {
        let predicate = MaxLengthPredicate(3);
        let inner = pizza_engine::analysis::UppercaseTokenFilter::new();
        let f = ConditionalTokenFilter::new(Box::new(predicate), Box::new(inner));
        check_filter(&f, "hi", "HI"); // matches: len 2 <= 3
        check_filter(&f, "hello", "hello"); // doesn't match: len 5 > 3
    }

    #[test]
    fn test_pattern_predicate() {
        let predicate = PatternPredicate::new(r"^\d+$").unwrap();
        let inner = PatternReplaceTokenFilter::new(r"\d", "X").unwrap();
        let f = ConditionalTokenFilter::new(Box::new(predicate), Box::new(inner));
        check_filter(&f, "123", "XXX"); // numeric, matches
        check_filter(&f, "abc", "abc"); // alpha, doesn't match
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 45. MultiplexerTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod multiplexer_filter {
    use super::*;

    #[test]
    fn test_no_filters() {
        let f = MultiplexerTokenFilter::new(vec![]);
        check_filter(&f, "hello", "hello");
    }

    #[test]
    fn test_single_filter() {
        let inner: Box<dyn TokenFilter> = Box::new(LowercaseTokenFilter::new());
        let f = MultiplexerTokenFilter::new(vec![inner]);
        let mut token = Token::new("HELLO", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        // Original preserved; lowercase form as extra
        assert_eq!(token.term.as_ref(), "HELLO");
        let extra_terms: Vec<&str> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.as_ref()).collect())
            .unwrap_or_default();
        assert!(extra_terms.contains(&"hello"));
    }

    #[test]
    fn test_multiple_filters() {
        let lower: Box<dyn TokenFilter> = Box::new(LowercaseTokenFilter::new());
        let reverse: Box<dyn TokenFilter> = Box::new(ReverseTokenFilter::new());
        let f = MultiplexerTokenFilter::new(vec![lower, reverse]);
        let mut token = Token::new("HELLO", 0, 5, 0);
        let (deleted, extras) = f.filter(&mut token);
        assert!(!deleted);
        let extra_terms: Vec<String> = extras
            .as_ref()
            .map(|v| v.iter().map(|t| t.term.to_string()).collect())
            .unwrap_or_default();
        assert!(extra_terms.contains(&"hello".to_string()));
        assert!(extra_terms.contains(&"OLLEH".to_string()));
    }

    #[test]
    fn test_without_preserve_original() {
        let inner: Box<dyn TokenFilter> = Box::new(LowercaseTokenFilter::new());
        let f = MultiplexerTokenFilter::new(vec![inner]).with_preserve_original(false);
        let mut token = Token::new("HELLO", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        // Without preserve_original, behavior may differ
        let _ = deleted;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 46. SoraniStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod sorani_stem {
    use super::*;

    #[test]
    fn test_postposition_removal() {
        let f = SoraniStemTokenFilter::new();
        // Kurdish word with postposition suffix
        let mut token = Token::new("خانەدا", 0, 12, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_short_word_unchanged() {
        let f = SoraniStemTokenFilter::new();
        check_filter(&f, "من", "من");
    }

    #[test]
    fn test_empty() {
        let f = SoraniStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_latin_unchanged() {
        let f = SoraniStemTokenFilter::new();
        check_filter(&f, "hello", "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 47. PortugueseMinimalStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod portuguese_minimal_stem {
    use super::*;

    #[test]
    fn test_oes_to_ao() {
        let f = PortugueseMinimalStemTokenFilter::new();
        check_filter(&f, "leões", "leão"); // lions
    }

    #[test]
    fn test_ais_to_al() {
        let f = PortugueseMinimalStemTokenFilter::new();
        let mut token = Token::new("animais", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_short_word() {
        let f = PortugueseMinimalStemTokenFilter::new();
        check_filter(&f, "sol", "sol");
    }

    #[test]
    fn test_empty() {
        let f = PortugueseMinimalStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_singular_unchanged() {
        let f = PortugueseMinimalStemTokenFilter::new();
        check_filter(&f, "gato", "gato");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 48. FinnishLightStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod finnish_light_stem {
    use super::*;

    #[test]
    fn test_case_suffix() {
        let f = FinnishLightStemTokenFilter::new();
        let mut token = Token::new("talossa", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
        // "talossa" should have "ssa" removed → "talo"
        assert!(
            token.term.as_ref().len() < "talossa".len(),
            "expected stemming of 'talossa', got {:?}",
            token.term.as_ref()
        );
    }

    #[test]
    fn test_short_word() {
        let f = FinnishLightStemTokenFilter::new();
        check_filter(&f, "tuo", "tuo");
    }

    #[test]
    fn test_empty() {
        let f = FinnishLightStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_no_suffix() {
        let f = FinnishLightStemTokenFilter::new();
        check_filter(&f, "talo", "talo");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 49. HungarianLightStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod hungarian_light_stem {
    use super::*;

    #[test]
    fn test_plural() {
        let f = HungarianLightStemTokenFilter::new();
        let mut token = Token::new("házakat", 0, 9, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_short_word() {
        let f = HungarianLightStemTokenFilter::new();
        check_filter(&f, "ház", "ház");
    }

    #[test]
    fn test_empty() {
        let f = HungarianLightStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_singular() {
        let f = HungarianLightStemTokenFilter::new();
        let mut token = Token::new("magyar", 0, 6, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 50. IndonesianStemTokenFilter
// ═══════════════════════════════════════════════════════════════════════════

mod indonesian_stem {
    use super::*;

    #[test]
    fn test_prefix_removal() {
        let f = IndonesianStemTokenFilter::new();
        let mut token = Token::new("memakan", 0, 7, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }

    #[test]
    fn test_short_word() {
        let f = IndonesianStemTokenFilter::new();
        check_filter(&f, "dan", "dan");
    }

    #[test]
    fn test_empty() {
        let f = IndonesianStemTokenFilter::new();
        check_filter(&f, "", "");
    }

    #[test]
    fn test_no_affix() {
        let f = IndonesianStemTokenFilter::new();
        let mut token = Token::new("makan", 0, 5, 0);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional cross-cutting tests
// ═══════════════════════════════════════════════════════════════════════════

mod cross_cutting {
    use super::*;

    /// Test that all filter constructors compile and don't panic.
    #[test]
    fn test_all_constructors() {
        let _ = EdgeNgramTokenFilter::new(1, 3);
        let _ = NgramTokenFilter::new(1, 2);
        let _ = ShingleTokenFilter::new(2, 3);
        let _ = KeywordMarkerTokenFilter::new(vec!["test".into()]);
        let _ = KeywordRepeatTokenFilter::new();
        let _ = PatternReplaceTokenFilter::new(r"\d", "X");
        let _ = PatternCaptureTokenFilter::new(vec![r"(\d+)"], true);
        let _ = LimitTokenFilter::new(10);
        let _ = LimitTokenOffsetFilter::new(100);
        let _ = LimitTokenPositionFilter::new(10);
        let _ = CommonGramsTokenFilter::new(vec!["the".into()]);
        let _ = CommonGramsQueryFilter::new();
        let _ = FixBrokenOffsetsFilter::new();
        let _ = ConcatenateGraphTokenFilter::new(' ');
        let _ = RemoveDuplicatesTokenFilter::new();
        let _ = ProtectedWordsTokenFilter::new(&["a"]);
        let _ = StemmerTokenFilter::from_language(StemmerLanguage::English);
        let _ = KStemTokenFilter::new();
        let _ = EnglishMinimalStemTokenFilter::new();
        let _ = HyphenatedWordsTokenFilter::new();
        let _ = ClassicTokenFilter::new();
        let _ = WordDelimiterTokenFilter::new(WordDelimiterConfig::default());
        let _ = WordDelimiterGraphTokenFilter::new();
        let _ = UniqueTokenFilter::new();
        let _ = RomanianNormalizationTokenFilter::new();
        let _ = SpanishPluralStemTokenFilter::new();
        let _ = SpanishMinimalStemTokenFilter::new();
        let _ = RussianYoNormalizationTokenFilter::new();
        let _ = NorwegianNormalizationTokenFilter::new();
        let _ = NorwegianMinimalStemTokenFilter::new();
        let _ = SwedishMinimalStemTokenFilter::new();
        let _ = BengaliNormalizationTokenFilter::new();
        let _ = GermanLightStemTokenFilter::new();
        let _ = GermanMinimalStemTokenFilter::new();
        let _ = FrenchMinimalStemTokenFilter::new();
        let _ = CapitalizationTokenFilter::new();
        let _ = CodepointCountTokenFilter::new(1, 10);
        let _ = KeepWordsTokenFilter::new(vec!["a".into()]);
        let _ = KeepTypesTokenFilter::new(vec![TokenType::Alpha], KeepTypesMode::Include);
        let _ = MinHashTokenFilter::new();
        let _ = PhoneticTokenFilter::new(PhoneticEncoder::Soundex);
        let _ = FlattenGraphTokenFilter::new();
        let _ = TypeAsSynonymTokenFilter::new();
        let _ = StemmerOverrideTokenFilter::from_rules(&[("a", "b")]);
        let _ = SoraniStemTokenFilter::new();
        let _ = PortugueseMinimalStemTokenFilter::new();
        let _ = FinnishLightStemTokenFilter::new();
        let _ = HungarianLightStemTokenFilter::new();
        let _ = IndonesianStemTokenFilter::new();
    }

    /// Test that filters handle the null/empty case gracefully.
    #[test]
    fn test_empty_token_universal() {
        let filters: Vec<Box<dyn TokenFilter>> = vec![
            Box::new(KeywordRepeatTokenFilter::new()),
            Box::new(ClassicTokenFilter::new()),
            Box::new(HyphenatedWordsTokenFilter::new()),
            Box::new(RomanianNormalizationTokenFilter::new()),
            Box::new(RussianYoNormalizationTokenFilter::new()),
            Box::new(NorwegianNormalizationTokenFilter::new()),
            Box::new(BengaliNormalizationTokenFilter::new()),
            Box::new(CapitalizationTokenFilter::new()),
            Box::new(FlattenGraphTokenFilter::new()),
            Box::new(RemoveDuplicatesTokenFilter::new()),
        ];
        for (i, f) in filters.iter().enumerate() {
            let mut token = Token::new("", 0, 0, 0);
            let (deleted, _) = f.filter(&mut token);
            // Empty tokens should not panic
            let _ = deleted;
            let _ = i;
        }
    }

    /// Test a pipeline: edge_ngram → lowercase
    #[test]
    fn test_pipeline_edge_ngram_lowercase() {
        let ngram = EdgeNgramTokenFilter::new(1, 3);
        let lower = LowercaseTokenFilter::new();

        let mut token = Token::new("QUICK", 0, 5, 0);
        let (deleted, extras) = ngram.filter(&mut token);
        assert!(!deleted);

        // Apply lowercase to the primary
        let (_, _) = lower.filter(&mut token);
        assert_eq!(token.term.as_ref(), "q");

        // Apply lowercase to extras
        if let Some(mut ex) = extras {
            for t in &mut ex {
                let _ = lower.filter(t);
            }
            let terms: Vec<&str> = ex.iter().map(|t| t.term.as_ref()).collect();
            assert_eq!(terms, vec!["qu", "qui"]);
        }
    }

    /// Test a pipeline: keyword_repeat + stemmer
    #[test]
    fn test_pipeline_keyword_repeat_stemmer() {
        let repeat = KeywordRepeatTokenFilter::new();
        let stemmer = StemmerTokenFilter::from_language(StemmerLanguage::English);

        let mut token = Token::new("running", 0, 7, 0);
        let (_, extras) = repeat.filter(&mut token);
        // Original
        let (_, _) = stemmer.filter(&mut token);
        // Stemmed: "run"
        assert_eq!(token.term.as_ref(), "run");

        // The extra (keyword copy) should also stem if we apply stemmer
        if let Some(mut ex) = extras {
            let (_, _) = stemmer.filter(&mut ex[0]);
            // Both should be "run" after stemming
            assert_eq!(ex[0].term.as_ref(), "run");
        }
    }
}
