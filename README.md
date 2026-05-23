<div align="center">

# 🧩 pizza-analysis-core

**Core text analysis components for [INFINI Pizza](https://pizza.rs)**

[![Crate](https://img.shields.io/badge/crate-pizza--analysis--core-blue)](https://github.com/pizza-rs/analysis-core)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

*16 tokenizers · 60+ token filters · 13 normalizers · 65 built-in language analyzers*

</div>

---

Provides the comprehensive foundation of normalizers, tokenizers, token filters, and pre-composed language analyzers for the [INFINI Pizza](https://pizza.rs) search engine.

## Table of Contents

- [Architecture](#architecture)
- [Normalizers](#normalizers)
- [Tokenizers](#tokenizers)
- [Token Filters](#token-filters)
- [Language Analyzers](#language-analyzers)
- [Stop Word Lists](#stop-word-lists)
- [Full Pipeline Examples](#full-pipeline-examples)
- [Features](#features)
- [Related Crates](#related-crates)

---

## Architecture

Pizza uses a three-stage analysis pipeline:

```
Input Text → [Normalizer(s)] → [Tokenizer] → [Token Filter(s)] → Indexed Tokens
```

| Stage | Role | Trait | Invocation |
|-------|------|-------|------------|
| **Normalize** | Pre-tokenization string transforms (mutate full input in-place) | `Normalizer` | `normalize(&mut String)` |
| **Tokenize** | Split text into a stream of tokens | `Tokenizer` | `tokenize(&str) -> Vec<Token>` |
| **Filter** | Transform, remove, or inject tokens post-tokenization | `TokenFilter` | `filter(&mut Token) -> (bool, Option<Vec<Token>>)` |

The `TokenFilter::filter()` return value:
- `(true, _)` → **remove** the token from the stream
- `(false, None)` → keep the (possibly modified) token
- `(false, Some(extras))` → keep the token AND inject additional tokens at the same position

All components are registered into an `AnalysisFactory` via `register_all()`:

```rust
use pizza_analysis_core::analyzers::register_all;
use pizza_engine::analysis::AnalysisFactory;

let mut factory = AnalysisFactory::new();
register_all(&mut factory);

// Now use any registered analyzer by name
let analyzer = factory.get_analyzer("english").unwrap();
```

---

## Normalizers

Normalizers operate on the raw input string **before** tokenization. They modify the entire text in-place.

### HtmlStripNormalizer

Strips HTML/XML tags and decodes HTML entities. Block-level tags are replaced with a space to preserve word boundaries.

```rust
use pizza_analysis_core::HtmlStripNormalizer;
use pizza_engine::analysis::Normalizer;

// Basic usage
let normalizer = HtmlStripNormalizer::new();
let mut text = String::from("<h1>Hello</h1><p>World &amp; Pizza</p>");
normalizer.normalize(&mut text);
assert_eq!(text, " Hello  World & Pizza ");

// Preserve specific tags (escaped tags are NOT stripped)
let normalizer = HtmlStripNormalizer::new()
    .with_escaped_tags(vec!["b".to_string(), "i".to_string()]);
let mut text = String::from("<b>bold</b> and <script>evil</script>");
normalizer.normalize(&mut text);
assert_eq!(text, "<b>bold</b> and  evil ");
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `escaped_tags` | `Vec<String>` | `[]` | HTML tags to preserve (case-insensitive) |

**Handles:** `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#NNN;`, `&#xHHHH;` entities.

---

### MappingNormalizer

Character/string mapping normalizer. Replaces source strings with target strings in a single pass.

```rust
use pizza_analysis_core::MappingNormalizer;
use pizza_engine::analysis::Normalizer;

// From a list of mappings
let normalizer = MappingNormalizer::from_mappings(&[
    ("α", "a"),
    ("β", "b"),
    (":)", "happy"),
    (":(", "sad"),
]);
let mut text = String::from("α and β :) :(");
normalizer.normalize(&mut text);
assert_eq!(text, "a and b happy sad");

// Build incrementally
let mut normalizer = MappingNormalizer::new();
normalizer.add_mapping("ö", "oe");
normalizer.add_mapping("ü", "ue");
let mut text = String::from("über öl");
normalizer.normalize(&mut text);
assert_eq!(text, "ueber oel");
```

| Method | Description |
|--------|-------------|
| `new()` | Empty mapping normalizer |
| `from_mappings(&[(&str, &str)])` | Create with initial mapping pairs |
| `add_mapping(&mut self, from, to)` | Add a single mapping |

---

### PatternReplaceNormalizer

Regex-based find & replace on the entire input string before tokenization.

```rust
use pizza_analysis_core::PatternReplaceNormalizer;
use pizza_engine::analysis::Normalizer;

// Replace digits with placeholder
let normalizer = PatternReplaceNormalizer::new(r"\d+", "NUM");
let mut text = String::from("order 12345 shipped on 2024-01-15");
normalizer.normalize(&mut text);
assert_eq!(text, "order NUM shipped on NUM-NUM-NUM");

// Use capture groups ($1, $2, etc.)
let normalizer = PatternReplaceNormalizer::new(r"(\w+)@(\w+)", "$1 at $2");
let mut text = String::from("user@host");
normalizer.normalize(&mut text);
assert_eq!(text, "user at host");

// Persian zero-width non-joiner → space (used in Persian analyzer)
let normalizer = PatternReplaceNormalizer::new(r"\x{200C}", " ");
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `pattern` | `&str` | Regex pattern (panics if invalid) |
| `replacement` | `&str` | Replacement string; supports `$1`, `$2` capture groups |

---

### LowercaseNormalizer / UppercaseNormalizer

Simple case conversion for the entire input text.

```rust
use pizza_engine::analysis::Normalizer;
// These are provided by pizza-engine
// LowercaseNormalizer::new() - converts all text to lowercase
// UppercaseNormalizer::new() - converts all text to uppercase
```

---

### TrimNormalizer

Strips leading and trailing whitespace from the input text.

```rust
use pizza_analysis_core::TrimNormalizer;
use pizza_engine::analysis::Normalizer;

let normalizer = TrimNormalizer::new();
let mut text = String::from("  hello world  \n");
normalizer.normalize(&mut text);
assert_eq!(text, "hello world");
```

---

### CollapseWhitespaceNormalizer

Collapses consecutive whitespace characters (spaces, tabs, newlines) into a single space.

```rust
use pizza_analysis_core::CollapseWhitespaceNormalizer;
use pizza_engine::analysis::Normalizer;

let normalizer = CollapseWhitespaceNormalizer::new();
let mut text = String::from("hello   world\t\tfoo\n\nbar");
normalizer.normalize(&mut text);
assert_eq!(text, "hello world foo bar");
```

---

### UnicodeNormalizer

Applies Unicode normalization (NFC, NFD, NFKC, NFKD) for handling composed vs. decomposed characters.

```rust
use pizza_analysis_core::UnicodeNormalizer;
use pizza_engine::analysis::Normalizer;

// NFC: Canonical Composition (most common)
let normalizer = UnicodeNormalizer::nfc();
let mut text = String::from("e\u{0301}"); // e + combining accent
normalizer.normalize(&mut text);
assert_eq!(text, "é"); // single composed character

// NFKC: Compatibility Composition (folds typographic variants)
let normalizer = UnicodeNormalizer::nfkc();
let mut text = String::from("ﬁ"); // fi ligature
normalizer.normalize(&mut text);
assert_eq!(text, "fi");
```

| Constructor | Form | Use Case |
|-------------|------|----------|
| `UnicodeNormalizer::nfc()` | NFC | Default composition; safe for most text |
| `UnicodeNormalizer::nfd()` | NFD | Decomposition; useful before diacritic stripping |
| `UnicodeNormalizer::nfkc()` | NFKC | Compatibility; normalizes ligatures, superscripts |
| `UnicodeNormalizer::nfkd()` | NFKD | Compat decomposition; most aggressive |

---

## Tokenizers

Tokenizers split the (normalized) text into individual tokens. Each token carries:
- `term: Cow<str>` — the token text
- `start_offset: u32` — byte offset of start in original text
- `end_offset: u32` — byte offset of end in original text
- `position: u32` — positional index in token stream

### StandardTokenizer

UAX#29 Unicode word break rules. Provided by `pizza-engine`.

```rust
use pizza_engine::analysis::{StandardTokenizer, Tokenizer};

let tokenizer = StandardTokenizer::new();
let tokens = tokenizer.tokenize("The quick brown fox jumps!");
// ["The", "quick", "brown", "fox", "jumps"]
```

Handles: Unicode letters/digits, keeps contractions (don't), splits on punctuation/whitespace.

---

### KeywordTokenizer

Emits the **entire input** as a single token. Useful for exact-match fields (IDs, tags, SKUs).

```rust
use pizza_analysis_core::KeywordTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = KeywordTokenizer::new();
let tokens = tokenizer.tokenize("New York City");
assert_eq!(tokens.len(), 1);
assert_eq!(tokens[0].term, "New York City");
```

---

### LetterTokenizer

Splits text at any character that is **not** a Unicode letter. Non-letter characters are discarded.

```rust
use pizza_analysis_core::LetterTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = LetterTokenizer::new();
let tokens = tokenizer.tokenize("hello-world! foo123bar");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["hello", "world", "foo", "bar"]);
```

---

### LowercaseTokenizer

Equivalent to `LetterTokenizer` + immediate lowercasing. Slightly more efficient than chaining.

```rust
use pizza_analysis_core::LowercaseTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = LowercaseTokenizer::new();
let tokens = tokenizer.tokenize("Hello WORLD Foo-Bar");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["hello", "world", "foo", "bar"]);
```

---

### NgramTokenizer

Generates character n-grams of configurable length from text.

```rust
use pizza_analysis_core::NgramTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = NgramTokenizer::new(2, 3);
let tokens = tokenizer.tokenize("pizza");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
// ["pi", "piz", "iz", "izz", "zz", "zza", "za"]
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `min_gram` | `usize` | required | Minimum n-gram size (inclusive) |
| `max_gram` | `usize` | required | Maximum n-gram size (inclusive) |

**Builder method:**
- `.with_token_chars(Vec<TokenCharKind>)` — Character classes to include: `Letter`, `Digit`, `Whitespace`, `Punctuation`, `Symbol`. Empty = all characters.

---

### EdgeNgramTokenizer

Generates **prefix-anchored** (edge) n-grams. Only produces n-grams starting from the beginning of each token/word.

```rust
use pizza_analysis_core::EdgeNgramTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = EdgeNgramTokenizer::new(1, 5);
let tokens = tokenizer.tokenize("pizza");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["p", "pi", "piz", "pizz", "pizza"]);
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `min_gram` | `usize` | Starting n-gram size |
| `max_gram` | `usize` | Maximum n-gram size |

**Use case:** Autocomplete / search-as-you-type fields.

---

### CharGroupTokenizer

Splits text on configurable character sets — either specific characters or entire character classes.

```rust
use pizza_analysis_core::CharGroupTokenizer;
use pizza_engine::analysis::Tokenizer;

// Split on specific characters
let tokenizer = CharGroupTokenizer::new(vec!['-', '_', '.']);
let tokens = tokenizer.tokenize("hello-world_foo.bar");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["hello", "world", "foo", "bar"]);

// Split on whitespace + punctuation character classes
let tokenizer = CharGroupTokenizer::new(vec![])
    .split_on_whitespace()
    .split_on_punctuation();
```

| Method | Description |
|--------|-------------|
| `new(chars: Vec<char>)` | Split on specific characters |
| `.split_on_whitespace()` | Also split on whitespace class |
| `.split_on_letter()` | Also split on letter class |
| `.split_on_digit()` | Also split on digit class |
| `.split_on_punctuation()` | Also split on punctuation class |
| `.split_on_symbol()` | Also split on symbol class |

---

### PathHierarchyTokenizer

Tokenizes filesystem-like paths into hierarchical segments for faceted navigation.

```rust
use pizza_analysis_core::PathHierarchyTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = PathHierarchyTokenizer::default();
let tokens = tokenizer.tokenize("/usr/local/bin");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["/usr", "/usr/local", "/usr/local/bin"]);

// Custom separator with skip
let tokenizer = PathHierarchyTokenizer::new()
    .with_separator('.')
    .with_skip(1);  // skip first segment
let tokens = tokenizer.tokenize("com.example.app.Main");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["example", "example.app", "example.app.Main"]);
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_separator(char)` | `'/'` | Path separator character |
| `.with_replacement(char)` | `'/'` | Character used in output |
| `.with_skip(usize)` | `0` | Skip first N path segments |
| `.reversed()` | `false` | Output in reverse hierarchy order |

---

### PatternTokenizer

Regex-based tokenizer with two operating modes:

1. **Split mode** (default, group = -1): Pattern is the delimiter; text between matches becomes tokens
2. **Match mode** (group ≥ 0): Pattern matches become tokens; capture groups extracted

```rust
use pizza_analysis_core::PatternTokenizer;
use pizza_engine::analysis::Tokenizer;

// Split mode: split on non-word characters (default)
let tokenizer = PatternTokenizer::default(); // pattern = r"\W+"
let tokens = tokenizer.tokenize("hello, world! foo-bar");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["hello", "world", "foo", "bar"]);

// Split on custom delimiter
let tokenizer = PatternTokenizer::new(r"[,;]\s*");
let tokens = tokenizer.tokenize("one, two; three");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["one", "two", "three"]);

// Match mode: extract emails
let tokenizer = PatternTokenizer::with_group(r"\b[\w.]+@[\w.]+\b", 0);
let tokens = tokenizer.tokenize("contact user@example.com or admin@site.org");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["user@example.com", "admin@site.org"]);
```

| Constructor | Description |
|-------------|-------------|
| `PatternTokenizer::default()` | Split on `\W+` |
| `PatternTokenizer::new(pattern)` | Split on custom regex |
| `PatternTokenizer::with_group(pattern, group)` | Match mode; group=0 for full match, 1+ for capture groups |

---

### ClassicTokenizer

Legacy tokenizer that recognizes English grammar patterns:
- Preserves acronyms (U.S.A.)
- Preserves company names with apostrophes (O'Reilly)
- Keeps email addresses and hostnames intact
- Splits on most punctuation

```rust
use pizza_analysis_core::ClassicTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = ClassicTokenizer::new();
let tokens = tokenizer.tokenize("U.S.A. email: test@example.com");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
// Keeps "U.S.A." and "test@example.com" as single tokens
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_max_token_length(usize)` | `255` | Maximum characters per token |

---

### UaxUrlEmailTokenizer

UAX#29-based tokenizer that additionally recognizes URLs and email addresses as single tokens.

```rust
use pizza_analysis_core::UaxUrlEmailTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = UaxUrlEmailTokenizer::new();
let tokens = tokenizer.tokenize("Visit https://pizza.dev or email hello@pizza.dev today");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["Visit", "https://pizza.dev", "or", "email", "hello@pizza.dev", "today"]);
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_max_token_length(usize)` | `255` | Maximum characters per token |

---

### SimplePatternTokenizer / SimplePatternSplitTokenizer

Lightweight regex tokenizers:
- **SimplePatternTokenizer**: Text matching the pattern becomes tokens
- **SimplePatternSplitTokenizer**: Pattern is a delimiter; text between matches becomes tokens

```rust
use pizza_analysis_core::SimplePatternTokenizer;
use pizza_engine::analysis::Tokenizer;

// Extract sequences of digits
let tokenizer = SimplePatternTokenizer::new(r"\d+").unwrap();
let tokens = tokenizer.tokenize("order 123 has 4 items");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["123", "4"]);
```

```rust
use pizza_analysis_core::SimplePatternSplitTokenizer;
use pizza_engine::analysis::Tokenizer;

// Split on underscores
let tokenizer = SimplePatternSplitTokenizer::new(r"_+").unwrap();
let tokens = tokenizer.tokenize("foo__bar_baz");
let terms: Vec<&str> = tokens.iter().map(|t| t.term.as_ref()).collect();
assert_eq!(terms, vec!["foo", "bar", "baz"]);
```

---

### ThaiTokenizer

Segments Thai text at script boundaries (Thai/non-Thai transitions). Handles Thai-specific whitespace and punctuation rules.

```rust
use pizza_analysis_core::ThaiTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = ThaiTokenizer::new();
let tokens = tokenizer.tokenize("การทดสอบ test");
// Splits at Thai/Latin script boundary
```

> **Note:** For full dictionary-based Thai word segmentation, use an external ICU-based tokenizer.

---

### BurmeseTokenizer

Segments Myanmar/Burmese script text at syllable boundaries using Unicode code points and the virama (killer) character `\u{1039}`.

```rust
use pizza_analysis_core::BurmeseTokenizer;
use pizza_engine::analysis::Tokenizer;

let tokenizer = BurmeseTokenizer::new();
let tokens = tokenizer.tokenize("မြန်မာစာ");
// Segments at Myanmar syllable boundaries
```

Non-Myanmar text is split at whitespace/punctuation as usual.

---

## Token Filters

Token filters transform, remove, or inject tokens after tokenization. They are applied sequentially in the order configured.

### Core Token Manipulation

#### LowercaseTokenFilter

Converts all token text to lowercase using Unicode-aware lowercasing.

```rust
use pizza_analysis_core::LowercaseTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = LowercaseTokenFilter::new();
let mut token = Token::new("HELLO World", 0, 11, 0);
filter.filter(&mut token);
assert_eq!(token.term, "hello world");
```

---

#### UppercaseTokenFilter

Converts all token text to uppercase.

```rust
use pizza_analysis_core::UppercaseTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = UppercaseTokenFilter::new();
let mut token = Token::new("hello", 0, 5, 0);
filter.filter(&mut token);
assert_eq!(token.term, "HELLO");
```

---

#### TrimTokenFilter

Removes leading and trailing whitespace from each token.

```rust
use pizza_analysis_core::TrimTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = TrimTokenFilter::new();
let mut token = Token::new("  hello  ", 0, 9, 0);
filter.filter(&mut token);
assert_eq!(token.term, "hello");
```

---

#### ReverseTokenFilter

Reverses the character order of each token. Useful for leading-wildcard search simulation.

```rust
use pizza_analysis_core::ReverseTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = ReverseTokenFilter::new();
let mut token = Token::new("hello", 0, 5, 0);
filter.filter(&mut token);
assert_eq!(token.term, "olleh");
```

---

#### TruncateTokenFilter

Truncates tokens to a maximum number of characters.

```rust
use pizza_analysis_core::TruncateTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = TruncateTokenFilter::new(5);
let mut token = Token::new("university", 0, 10, 0);
filter.filter(&mut token);
assert_eq!(token.term, "unive");
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `length` | `usize` | Maximum characters to keep |

---

#### LengthTokenFilter

**Removes** tokens that fall outside the specified character length range.

```rust
use pizza_analysis_core::LengthTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = LengthTokenFilter::new(3, 10);

let mut short = Token::new("ab", 0, 2, 0);
let (remove, _) = filter.filter(&mut short);
assert!(remove); // "ab" is too short (< 3)

let mut ok = Token::new("hello", 0, 5, 1);
let (remove, _) = filter.filter(&mut ok);
assert!(!remove); // "hello" is 5 chars, within [3, 10]
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `min` | `usize` | Minimum token length (chars); shorter tokens removed |
| `max` | `usize` | Maximum token length (chars); longer tokens removed |

---

#### LimitTokenFilter

Limits the total number of tokens emitted. Stateful — call `reset()` between documents.

```rust
use pizza_analysis_core::LimitTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = LimitTokenFilter::new(3);
// First 3 tokens pass through, subsequent ones are removed
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `max_token_count` | `u32` | Maximum tokens to emit per document |

---

#### UniqueTokenFilter

Removes duplicate tokens from the stream. Only keeps the first occurrence.

```rust
use pizza_analysis_core::UniqueTokenFilter;
// If stream is ["the", "quick", "the", "fox"], removes second "the"
```

---

#### RemoveDuplicatesTokenFilter

Removes duplicate tokens at the **same position** (e.g., from synonym expansion). Uses `RemoveDuplicatesState` for tracking.

---

### Stop Words & Filtering

#### StopTokenFilter

Removes common stop words from the token stream.

```rust
use pizza_analysis_core::StopTokenFilter;
use pizza_analysis_core::token_filters::stopwords;
use pizza_engine::analysis::{Token, TokenFilter};

// Using built-in language stop words
let words = stopwords::get_stop_words("english").unwrap();
let filter = StopTokenFilter::new(words);

let mut token = Token::new("the", 0, 3, 0);
let (remove, _) = filter.filter(&mut token);
assert!(remove); // "the" is a stop word

let mut token = Token::new("pizza", 0, 5, 1);
let (remove, _) = filter.filter(&mut token);
assert!(!remove); // "pizza" is NOT a stop word

// Case-insensitive mode
let filter = StopTokenFilter::new(&["the", "a", "an"])
    .with_ignore_case(true);
let mut token = Token::new("THE", 0, 3, 0);
let (remove, _) = filter.filter(&mut token);
assert!(remove); // matches "the" case-insensitively
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `words` | `&[&str]` | required | Stop word list |
| `ignore_case` | `bool` | `false` | Case-insensitive matching |

---

#### KeepWordsTokenFilter

The **inverse** of stop filter: only keeps tokens in the whitelist; removes everything else.

```rust
use pizza_analysis_core::KeepWordsTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = KeepWordsTokenFilter::new(
    vec!["pizza".to_string(), "pasta".to_string(), "salad".to_string()]
);

let mut token = Token::new("pizza", 0, 5, 0);
let (remove, _) = filter.filter(&mut token);
assert!(!remove); // "pizza" is in keep list

let mut token = Token::new("burger", 0, 6, 1);
let (remove, _) = filter.filter(&mut token);
assert!(remove); // "burger" is NOT in keep list
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `words` | `Vec<String>` | required | Whitelist of words to keep |
| `ignore_case` | `bool` | `false` | Case-insensitive matching |

---

### Character Normalization

#### AsciiFoldingTokenFilter

Folds Unicode characters to their ASCII equivalents: ü→u, é→e, ñ→n, ß→ss, ø→o, and hundreds more including Greek/Cyrillic transliterations.

```rust
use pizza_analysis_core::AsciiFoldingTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = AsciiFoldingTokenFilter::new();
let mut token = Token::new("résumé", 0, 8, 0);
filter.filter(&mut token);
assert_eq!(token.term, "resume");

// Preserve original + emit folded version
let filter = AsciiFoldingTokenFilter::preserving_original();
let mut token = Token::new("über", 0, 5, 0);
let (_, extra) = filter.filter(&mut token);
assert_eq!(token.term, "uber");  // modified to ASCII
// extra contains original "über" at same position
```

---

#### DecimalDigitTokenFilter

Converts Unicode decimal digits from any script (Arabic-Indic, Devanagari, Thai, etc.) to ASCII 0-9.

```rust
use pizza_analysis_core::DecimalDigitTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = DecimalDigitTokenFilter::new();
let mut token = Token::new("٣٢١", 0, 6, 0); // Arabic-Indic digits
filter.filter(&mut token);
assert_eq!(token.term, "321");
```

---

#### CjkWidthTokenFilter

Normalizes fullwidth/halfwidth CJK character variants:
- Fullwidth ASCII (Ａ-Ｚ, ０-９) → normal ASCII (A-Z, 0-9)
- Halfwidth Katakana → fullwidth Katakana

```rust
use pizza_analysis_core::CjkWidthTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = CjkWidthTokenFilter::new();
let mut token = Token::new("Ｔｅｓｔ", 0, 12, 0); // fullwidth
filter.filter(&mut token);
assert_eq!(token.term, "Test");
```

---

### Morphology & Stemming

#### ApostropheTokenFilter

Strips everything after (and including) the first apostrophe. Useful for Turkish, Italian.

```rust
use pizza_analysis_core::ApostropheTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = ApostropheTokenFilter::new();
let mut token = Token::new("Istanbul'un", 0, 11, 0);
filter.filter(&mut token);
assert_eq!(token.term, "Istanbul");
```

---

#### ElisionTokenFilter

Removes leading articles/elisions in Romance languages (text before an apostrophe when it matches a known article).

```rust
use pizza_analysis_core::ElisionTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

// French elisions
let filter = ElisionTokenFilter::french();
let mut token = Token::new("l'avion", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "avion");

let mut token = Token::new("qu'est", 0, 6, 0);
filter.filter(&mut token);
assert_eq!(token.term, "est");

// Custom articles
let filter = ElisionTokenFilter::new(&["l", "d", "n", "qu"]);

// Pre-built language sets
let french = ElisionTokenFilter::french();   // l', m', t', qu', n', s', j', d'
let italian = ElisionTokenFilter::italian(); // l', all', dall', dell', nell', ...
let catalan = ElisionTokenFilter::catalan(); // d', l', m', n', s', qu'
```

| Constructor | Articles |
|-------------|----------|
| `ElisionTokenFilter::new(&[&str])` | Custom article list |
| `ElisionTokenFilter::french()` | l, m, t, qu, n, s, j, d |
| `ElisionTokenFilter::italian()` | l, all, dall, dell, nell, sull, un, quest, quell |
| `ElisionTokenFilter::catalan()` | d, l, m, n, s, qu |

---

#### KStemTokenFilter

K-stem algorithm for English. Combines algorithmic suffix stripping with a dictionary for high-quality English stemming.

```rust
use pizza_analysis_core::KStemTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = KStemTokenFilter::new();
let mut token = Token::new("running", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "run");
```

Less aggressive than Porter stemmer. Does not over-stem: "university" stays "university" (not "univers").

---

#### ClassicTokenFilter

Post-processing for `ClassicTokenizer`: removes trailing possessives (`'s`) and dots from acronyms.

```rust
use pizza_analysis_core::ClassicTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = ClassicTokenFilter::new();
let mut token = Token::new("U.S.A.", 0, 6, 0);
filter.filter(&mut token);
assert_eq!(token.term, "USA");

let mut token = Token::new("children's", 0, 10, 0);
filter.filter(&mut token);
assert_eq!(token.term, "children");
```

---

#### StemmerOverrideTokenFilter

Dictionary-based stem override. Apply **before** algorithmic stemming to handle exceptions and irregular words.

```rust
use pizza_analysis_core::StemmerOverrideTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = StemmerOverrideTokenFilter::from_rules(&[
    ("running", "run"),
    ("better", "good"),
    ("mice", "mouse"),
]);

let mut token = Token::new("mice", 0, 4, 0);
filter.filter(&mut token);
assert_eq!(token.term, "mouse");
```

| Constructor | Format |
|-------------|--------|
| `from_rules(&[(&str, &str)])` | (word, stem) pairs |
| `new(HashMap<String, String>)` | Pre-built HashMap |
| `.with_ignore_case(bool)` | Case-insensitive lookup (default: false) |

---

#### DictionaryStemTokenFilter

Dictionary-based stemming using loaded word→stem mappings. Alternative to algorithmic stemmers for domain-specific vocabularies.

```rust
use pizza_analysis_core::DictionaryStemTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

// From tab-separated file format
let filter = DictionaryStemTokenFilter::from_tab_separated(
    "running\trun\nswimming\tswim\nchildren\tchild"
);

// From arrow-separated format
let filter = DictionaryStemTokenFilter::from_arrow_separated(
    "running => run\nswimming => swim"
);

let mut token = Token::new("running", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "run");
```

| Method | Description |
|--------|-------------|
| `new(entries: Vec<(String, String)>)` | From (word, stem) pairs |
| `from_tab_separated(content: &str)` | Parse `"word\tstem"` lines |
| `from_arrow_separated(content: &str)` | Parse `"word => stem"` lines |
| `.with_case_insensitive(bool)` | Case-insensitive (default: true) |

---

#### HunspellStemFilter

Morphological stemming using Hunspell-style affix rules. Supports prefix/suffix stripping with conditions.

```rust
use pizza_analysis_core::{HunspellStemFilter, AffixRule};
use pizza_engine::analysis::{Token, TokenFilter};

let mut filter = HunspellStemFilter::new();
// Add suffix rule: strip "ing", add "", condition "." (any)
filter.add_suffix_rule("ing", "", ".");
// Add suffix rule: strip "s", add "", condition "." (any)
filter.add_suffix_rule("s", "", ".");

let mut token = Token::new("running", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "runn"); // strips "ing"
```

| Method | Parameters | Description |
|--------|------------|-------------|
| `add_suffix_rule(strip, affix, condition)` | All `&str` | Add a suffix stripping rule |
| `add_prefix_rule(strip, affix, condition)` | All `&str` | Add a prefix stripping rule |

Config fields: `dedup: bool` (default: true), `longest_only: bool` (default: false).

---

### Language-Specific Stemmer Filters

All language stemmers take no parameters (`::new()`) and implement lightweight suffix-stripping algorithms.

| Filter | Language | Algorithm |
|--------|----------|-----------|
| `ArabicStemTokenFilter` | Arabic | Root extraction (prefix/suffix/pattern removal) |
| `BengaliStemTokenFilter` | Bengali | Common Bengali suffix removal |
| `BrazilianStemTokenFilter` | Portuguese (BR) | Plural, gender, verb, and noun stemming |
| `BulgarianStemTokenFilter` | Bulgarian | Light suffix stripping |
| `CzechStemTokenFilter` | Czech | Dolamic/Savoy light stemmer + palatalization |
| `DutchStemTokenFilter` | Dutch | Kraaij-Pohlmann suffix algorithm |
| `FinnishLightStemTokenFilter` | Finnish | Case/number suffix removal (-ssa, -lla, -lta, etc.) |
| `FrenchLightStemTokenFilter` | French | ~70 rules; gender/plural/verb endings |
| `FrenchMinimalStemTokenFilter` | French | Minimal: plural + feminine only |
| `GalicianStemTokenFilter` | Galician | Full: plural + derivational suffixes |
| `GalicianMinimalStemTokenFilter` | Galician | Minimal: plural only |
| `GermanLightStemTokenFilter` | German | Light compound-aware stemmer |
| `GermanMinimalStemTokenFilter` | German | Minimal: plurals only |
| `GreekStemTokenFilter` | Greek | Greek suffix rule set |
| `HindiStemTokenFilter` | Hindi | Hindi suffix removal |
| `HungarianLightStemTokenFilter` | Hungarian | Case suffix removal (-ban, -nak, -ból, etc.) |
| `IndonesianStemTokenFilter` | Indonesian | Prefix (me-, ber-, di-) + suffix (-kan, -an, -i) |
| `ItalianLightStemTokenFilter` | Italian | Light plurals/gender |
| `KannadaStemTokenFilter` | Kannada | Vibhakti (case marker) removal |
| `LatvianStemTokenFilter` | Latvian | Noun/adjective/verb endings |
| `NorwegianLightStemTokenFilter` | Norwegian | Light (Bokmål + Nynorsk) |
| `PersianStemTokenFilter` | Persian | Persian suffix stemmer |
| `PortugueseLightStemTokenFilter` | Portuguese | Light plural/gender removal |
| `RussianLightStemTokenFilter` | Russian | Lightweight suffix stripping |
| `SpanishLightStemTokenFilter` | Spanish | Light plural/gender |
| `TamilStemTokenFilter` | Tamil | Case/plural suffix stripping |
| `TeluguStemTokenFilter` | Telugu | Case marker suffix stripping |

**Example:**

```rust
use pizza_analysis_core::FrenchLightStemTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = FrenchLightStemTokenFilter::new();
let mut token = Token::new("chevaux", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "cheval");

let mut token = Token::new("nationale", 0, 9, 0);
filter.filter(&mut token);
assert_eq!(token.term, "national");
```

---

### Language-Specific Normalization Filters

These normalize language-specific character variations. All take no parameters (`::new()`).

#### ArabicNormalizationTokenFilter

Normalizes Arabic orthographic variations:
- Alef variants (أ إ آ) → Alef (ا)
- Teh Marbuta (ة) → Heh (ه)
- Yeh variants (ى) → Yeh (ي)
- Removes diacritics (Fatha, Kasra, Damma, Shadda, Sukun)

```rust
use pizza_analysis_core::ArabicNormalizationTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = ArabicNormalizationTokenFilter::new();
// Normalizes Arabic character variants for consistent indexing
```

---

#### GermanNormalizationTokenFilter

Normalizes German umlaut characters and sharp-s:
- ä → a, ö → o, ü → u
- ß → ss
- ae → a, oe → o, ue → u (digraph normalization)

```rust
use pizza_analysis_core::GermanNormalizationTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = GermanNormalizationTokenFilter::new();
let mut token = Token::new("über", 0, 5, 0);
filter.filter(&mut token);
assert_eq!(token.term, "uber");

let mut token = Token::new("straße", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "strasse");
```

---

#### IndicNormalizationTokenFilter

Shared normalization across all Indic scripts (Devanagari, Bengali, Gurmukhi, Gujarati, Oriya, Telugu, Kannada, Malayalam). Normalizes nukta, canonical equivalents, and visarga.

---

#### HindiNormalizationTokenFilter

Hindi-specific normalization (applied after IndicNormalization):
- Chandrabindu → Anunasika
- Nukta removal
- Final halant removal

---

#### BengaliNormalizationTokenFilter

Bengali-specific character normalizations on top of the generic Indic normalization.

---

#### PersianNormalizationTokenFilter

Normalizes Persian character variants:
- Arabic Yeh (ي) → Persian Yeh (ی)
- Arabic Keh (ك) → Persian Keh (ک)

---

#### RomanianNormalizationTokenFilter

Romanian diacritic normalization (handles both old and new standard):
- ş (cedilla) → ș (comma below)
- ţ (cedilla) → ț (comma below)

---

#### ScandinavianNormalizationTokenFilter

Normalizes interchangeable Scandinavian vowels:
- ä, æ → a
- ö, ø → o
- å → o (for Swedish/Norwegian equivalence)

```rust
use pizza_analysis_core::ScandinavianNormalizationTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = ScandinavianNormalizationTokenFilter::new();
let mut token = Token::new("räksmörgås", 0, 12, 0);
filter.filter(&mut token);
// Normalizes Scandinavian vowels for cross-language matching
```

---

#### ScandinavianFoldingTokenFilter

More aggressive Scandinavian folding than normalization:
- å → a, ä → a, æ → a
- ö → o, ø → o
- ü → u

---

#### SerbianNormalizationTokenFilter

Transliterates Serbian Cyrillic to Latin equivalent for unified indexing.

```rust
use pizza_analysis_core::SerbianNormalizationTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = SerbianNormalizationTokenFilter::new();
let mut token = Token::new("Београд", 0, 14, 0);
filter.filter(&mut token);
assert_eq!(token.term, "Beograd");
```

---

#### SoraniNormalizationTokenFilter

Sorani Kurdish normalization: handles Yeh/Alef Maksura equivalence, Heh/Ae variations.

---

#### GreekLowercaseTokenFilter

Greek-aware lowercasing that handles:
- Tonos (accent) removal
- Final sigma (ς → σ after lowercasing)
- Dialytika preservation

```rust
use pizza_analysis_core::GreekLowercaseTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = GreekLowercaseTokenFilter::new();
let mut token = Token::new("ΑΘΉΝΑ", 0, 10, 0);
filter.filter(&mut token);
assert_eq!(token.term, "αθηνα"); // tonos removed, lowercased
```

---

#### TurkishLowercaseTokenFilter

Turkish-specific lowercasing with dotted/dotless I handling:
- İ (U+0130) → i
- I → ı (U+0131, dotless i)

```rust
use pizza_analysis_core::TurkishLowercaseTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = TurkishLowercaseTokenFilter::new();
let mut token = Token::new("İSTANBUL", 0, 9, 0);
filter.filter(&mut token);
assert_eq!(token.term, "istanbul");

let mut token = Token::new("ISPARTA", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "ısparta"); // I → ı (dotless)
```

---

#### IrishLowercaseTokenFilter / IrishElisionTokenFilter

- **IrishLowercaseTokenFilter**: Handles Irish eclipsis mutations (nDún → dún when lowercasing)
- **IrishElisionTokenFilter**: Strips Irish elisions: d', n-, t-

---

### N-gram & Shingle Filters

#### NgramTokenFilter

Generates character-level n-grams from each token.

```rust
use pizza_analysis_core::NgramTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = NgramTokenFilter::new(2, 3);
let mut token = Token::new("hello", 0, 5, 0);
let (_, extra) = filter.filter(&mut token);
// token becomes "he" (first 2-gram)
// extra contains: "hel", "el", "ell", "ll", "llo", "lo"
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `min_gram` | `usize` | required | Minimum n-gram size |
| `max_gram` | `usize` | required | Maximum n-gram size |
| `preserve_original` | `bool` | `false` | Keep original token |

---

#### EdgeNgramTokenFilter

Generates prefix n-grams from each token (useful for autocomplete at index time).

```rust
use pizza_analysis_core::EdgeNgramTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = EdgeNgramTokenFilter::new(1, 4)
    .with_preserve_original(true);
let mut token = Token::new("pizza", 0, 5, 0);
let (_, extra) = filter.filter(&mut token);
// token becomes "p" (min edge-gram)
// extra contains: "pi", "piz", "pizz", and original "pizza"
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `min_gram` | `usize` | required | Starting prefix length |
| `max_gram` | `usize` | required | Maximum prefix length |
| `preserve_original` | `bool` | `false` | Keep original token |

---

#### ShingleTokenFilter

Creates word-level n-grams (shingles) for phrase search optimization. **Stateful** — uses `add_token()` API.

```rust
use pizza_analysis_core::ShingleTokenFilter;

let mut filter = ShingleTokenFilter::new(2, 3)
    .with_separator(" ")
    .with_output_unigrams(false);

// Feed tokens one at a time
filter.reset();
let shingles1 = filter.add_token("the");    // []
let shingles2 = filter.add_token("quick");  // ["the quick"]
let shingles3 = filter.add_token("fox");    // ["quick fox", "the quick fox"]
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `min_size` | `usize` | `2` | Minimum shingle size (words) |
| `max_size` | `usize` | required | Maximum shingle size (words) |
| `separator` | `String` | `" "` | Word separator in output |
| `output_unigrams` | `bool` | `true` | Output individual tokens too |
| `filler_token` | `String` | `"_"` | Placeholder for position gaps |

---

#### CommonGramsTokenFilter

Creates bigrams pairing adjacent common words to preserve phrase-query capability while reducing stop-word index impact.

```rust
use pizza_analysis_core::CommonGramsTokenFilter;

let mut filter = CommonGramsTokenFilter::new(
    vec!["the".to_string(), "is".to_string(), "a".to_string()]
);

filter.reset();
let r1 = filter.process_token("the");     // None (buffered)
let r2 = filter.process_token("quick");   // Some("the_quick") bigram
let r3 = filter.process_token("fox");     // None (not common)
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `words` | `Vec<String>` | required | Common/frequent words |
| `ignore_case` | `bool` | `false` | Case-insensitive |
| `separator` | `String` | `"_"` | Bigram separator |

---

#### CjkBigramTokenFilter

Creates bigrams from consecutive CJK characters (Han, Hiragana, Katakana, Hangul).

```rust
use pizza_analysis_core::CjkBigramTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = CjkBigramTokenFilter::new();
let mut token = Token::new("東京都", 0, 9, 0);
let (_, extra) = filter.filter(&mut token);
// Produces bigrams: "東京", "京都"
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_output_unigrams(bool)` | `false` | Also output individual CJK chars |
| `.with_han(bool)` | `true` | Include Han (Chinese) characters |
| `.with_hiragana(bool)` | `true` | Include Hiragana |
| `.with_katakana(bool)` | `true` | Include Katakana |
| `.with_hangul(bool)` | `true` | Include Hangul (Korean) |

---

### Synonyms & Expansion

#### SynonymTokenFilter

Expands or contracts synonyms. Supports two modes:
- **Expand**: All synonyms emitted at the same position (for recall)
- **Contract**: Map multiple forms to a single canonical form (for precision)

```rust
use pizza_analysis_core::{SynonymTokenFilter, SynonymMode};
use pizza_engine::analysis::{Token, TokenFilter};

let mut filter = SynonymTokenFilter::new(true); // case-insensitive

// Equivalence group: all terms are interchangeable
filter.add_equivalence(&["fast", "quick", "speedy"]);

// Explicit mapping: "big" → replace with "large"
filter.add_mapping("big", &["large"], SynonymMode::Contract);

// Parse Solr/ES format
filter.parse_rules("
    happy, glad, joyful
    sad => unhappy
");

let mut token = Token::new("fast", 0, 4, 0);
let (_, extra) = filter.filter(&mut token);
// token = "fast", extra = ["quick", "speedy"] at same position
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `ignore_case` | `bool` | required | Case-insensitive matching |

| Mode | Format | Behavior |
|------|--------|----------|
| Expand | `"a, b, c"` | Any of a/b/c → emits all three |
| Contract | `"a => b"` | "a" → replaced with "b" |

---

#### KeywordRepeatTokenFilter

Emits each token **twice**: once as a keyword (protected from stemming) and once for normal processing. Used with `RemoveDuplicatesTokenFilter` downstream.

```rust
use pizza_analysis_core::KeywordRepeatTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = KeywordRepeatTokenFilter::new();
let mut token = Token::new("running", 0, 7, 0);
let (_, extra) = filter.filter(&mut token);
// token = "running" (will be stemmed)
// extra = ["running"] at same position (keyword, skip stemming)
```

---

#### KeywordMarkerTokenFilter

Marks specific tokens as keywords to prevent downstream stemming.

```rust
use pizza_analysis_core::KeywordMarkerTokenFilter;

let filter = KeywordMarkerTokenFilter::new(
    vec!["iPhone".to_string(), "PlayStation".to_string()]
);
```

---

### Pattern-Based Filters

#### PatternCaptureTokenFilter

Extracts regex capture groups as additional tokens. Useful for splitting compound patterns.

```rust
use pizza_analysis_core::PatternCaptureTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = PatternCaptureTokenFilter::new(
    vec![r"(\d+)-(\w+)"],
    true  // preserve original
);

let mut token = Token::new("123-abc", 0, 7, 0);
let (_, extra) = filter.filter(&mut token);
// extra contains "123" and "abc" (capture groups)
// original "123-abc" preserved
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `patterns` | `Vec<&str>` | List of regex patterns with capture groups |
| `preserve_original` | `bool` | Keep original token in stream |

---

#### PatternReplaceTokenFilter

Regex-based find/replace within individual token text.

```rust
use pizza_analysis_core::PatternReplaceTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = PatternReplaceTokenFilter::new(r"[_-]", " ").unwrap();
let mut token = Token::new("hello_world-test", 0, 16, 0);
filter.filter(&mut token);
assert_eq!(token.term, "hello world test");

// Replace only first occurrence
let filter = PatternReplaceTokenFilter::new(r"\d+", "N")
    .unwrap()
    .with_replace_all(false);
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `pattern` | `&str` | required | Regex pattern |
| `replacement` | `&str` | required | Replacement string |
| `replace_all` | `bool` | `true` | Replace all vs. first only |

**Note:** Tokens are **removed** if replacement produces an empty string.

---

### Word Splitting

#### WordDelimiterTokenFilter

Splits tokens at case transitions, letter/digit boundaries, and delimiter characters.

```rust
use pizza_analysis_core::{WordDelimiterTokenFilter, WordDelimiterConfig};
use pizza_engine::analysis::{Token, TokenFilter};

let config = WordDelimiterConfig {
    split_on_case_change: true,
    split_on_numerics: true,
    generate_word_parts: true,
    generate_number_parts: true,
    catenate_words: false,
    catenate_numbers: false,
    preserve_original: false,
};
let filter = WordDelimiterTokenFilter::new(config);

let mut token = Token::new("camelCase", 0, 9, 0);
let (_, extra) = filter.filter(&mut token);
assert_eq!(token.term, "camel");
assert_eq!(extra.unwrap()[0].term, "Case");

let mut token = Token::new("Wi-Fi", 0, 5, 0);
let (_, extra) = filter.filter(&mut token);
// "Wi", "Fi"
```

| Config Field | Type | Default | Description |
|--------------|------|---------|-------------|
| `split_on_case_change` | `bool` | `true` | Split camelCase → camel + Case |
| `split_on_numerics` | `bool` | `true` | Split letter-digit boundaries |
| `generate_word_parts` | `bool` | `true` | Output alphabetic sub-parts |
| `generate_number_parts` | `bool` | `true` | Output numeric sub-parts |
| `catenate_words` | `bool` | `false` | Also emit concatenated word parts |
| `catenate_numbers` | `bool` | `false` | Also emit concatenated number parts |
| `preserve_original` | `bool` | `false` | Keep original token |

---

#### WordDelimiterGraphTokenFilter

Graph-aware version with correct position tracking for phrase queries. Additional options:

| Config Field | Type | Default | Description |
|--------------|------|---------|-------------|
| `concatenate_all` | `bool` | `false` | Emit concatenation of all parts |
| `stem_english_possessive` | `bool` | `true` | Remove trailing 's |

---

### Compound Word Decomposition

#### DictionaryDecompounderTokenFilter

Splits compound words (common in Germanic languages) using a dictionary of known word parts.

```rust
use pizza_analysis_core::DictionaryDecompounderTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let dict = vec![
    "donner".to_string(), "wetter".to_string(),
    "butter".to_string(), "brot".to_string(),
    "schule".to_string(), "kind".to_string(),
];
let filter = DictionaryDecompounderTokenFilter::new(dict)
    .with_min_word_size(5)
    .with_min_subword_size(3);

let mut token = Token::new("donnerwetter", 0, 12, 0);
let (_, extra) = filter.filter(&mut token);
// token = "donnerwetter" (preserved)
// extra = ["donner", "wetter"]
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_min_word_size(usize)` | `5` | Minimum input word length to attempt decomposition |
| `.with_min_subword_size(usize)` | `2` | Minimum component length |
| `.with_max_subword_size(usize)` | `15` | Maximum component length |
| `.with_only_longest_match(bool)` | `false` | Only emit longest decomposition |

---

#### HyphenationDecompounderTokenFilter

Same as DictionaryDecompounder but uses hyphenation patterns to find possible split points before dictionary lookup.

```rust
use pizza_analysis_core::HyphenationDecompounderTokenFilter;

let filter = HyphenationDecompounderTokenFilter::new(
    vec!["butter".to_string(), "brot".to_string()]
);
```

Same configuration options as `DictionaryDecompounderTokenFilter`.

---

### Phonetic Encoding

#### PhoneticTokenFilter

Encodes tokens using phonetic algorithms for sound-based matching ("sounds like" search).

```rust
use pizza_analysis_core::{PhoneticTokenFilter, PhoneticEncoder};
use pizza_engine::analysis::{Token, TokenFilter};

// Metaphone encoding
let filter = PhoneticTokenFilter::new(PhoneticEncoder::Metaphone(6));
let mut token = Token::new("smith", 0, 5, 0);
filter.filter(&mut token);
assert_eq!(token.term, "SM0"); // phonetic code

// Keep original + emit phonetic as extra token
let filter = PhoneticTokenFilter::new(PhoneticEncoder::Soundex)
    .with_replace(false);
let mut token = Token::new("robert", 0, 6, 0);
let (_, extra) = filter.filter(&mut token);
// token = "robert" (original preserved)
// extra = ["R163"] (Soundex code)
```

| Encoder | Description | Example |
|---------|-------------|---------|
| `Metaphone(max_len)` | Standard Metaphone | "smith" → "SM0" |
| `DoubleMetaphone(max_len)` | Two encodings per word | "smith" → "SM0"/"XMT" |
| `Soundex` | Classic 4-char code | "robert" → "R163" |
| `RefinedSoundex` | More granular Soundex | More distinctions |
| `Caverphone1` | NZ English optimized | |
| `Caverphone2` | Updated Caverphone | |
| `ColognePhonetic` | German phonetic | "müller" → "657" |
| `Nysiis` | NY state algorithm | |
| `DaitchMokotoff` | Eastern European names | |

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `encoder` | `PhoneticEncoder` | required | Algorithm to use |
| `replace` | `bool` | `true` | Replace original (true) or emit alongside (false) |

---

#### BeiderMorseFilter

Beider-Morse Phonetic Matching for multi-language surname matching. Generates phonetic representations considering multiple possible language origins.

```rust
use pizza_analysis_core::{BeiderMorseFilter, BmNameType, BmRuleType};
use pizza_engine::analysis::{Token, TokenFilter};

let filter = BeiderMorseFilter::new()
    .with_name_type(BmNameType::Generic)
    .with_rule_type(BmRuleType::Approx)
    .with_max_phonemes(10);

let mut token = Token::new("Schmidt", 0, 7, 0);
let (_, extra) = filter.filter(&mut token);
// Multiple phonetic variants for different language origins
```

| Method | Options | Default | Description |
|--------|---------|---------|-------------|
| `.with_name_type()` | `Generic`, `Ashkenazi`, `Sephardic` | `Generic` | Name origin type |
| `.with_rule_type()` | `Approx`, `Exact` | `Approx` | Matching strictness |
| `.with_replace(bool)` | | `true` | Replace or emit alongside |
| `.with_max_phonemes(usize)` | | `20` | Max phonetic variants |

---

#### PhoneNumberFilter

Parses and normalizes phone numbers for consistent indexing.

```rust
use pizza_analysis_core::PhoneNumberFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = PhoneNumberFilter::new();
// Normalizes various phone formats
// +1 (555) 123-4567 → 15551234567
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `generate_variants` | `bool` | `true` | Generate format variants for matching |

---

### Fingerprinting & Deduplication

#### FingerprintTokenFilter

Creates a "fingerprint" of a document by sorting unique tokens and joining them. Useful for near-duplicate detection.

```rust
use pizza_analysis_core::FingerprintAccumulator;

// Use FingerprintAccumulator for stream-level fingerprinting
let mut acc = FingerprintAccumulator::new(" ", 1024);
acc.add_token("quick");
acc.add_token("the");
acc.add_token("brown");
acc.add_token("the"); // duplicate, ignored
let fingerprint = acc.finish();
assert_eq!(fingerprint, "brown quick the"); // sorted, deduped
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_max_output_size(usize)` | `1024` | Maximum fingerprint length |
| `.with_separator(&str)` | `" "` | Token separator |

---

#### MinHashTokenFilter

Generates MinHash signatures for locality-sensitive hashing (document similarity / near-duplicate detection).

```rust
use pizza_analysis_core::MinHashTokenFilter;

let filter = MinHashTokenFilter::new()
    .with_hash_count(1)
    .with_bucket_count(512)
    .with_hash_set_size(1)
    .with_rotation(true);
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_hash_count(usize)` | `1` | Number of hash functions |
| `.with_bucket_count(usize)` | `512` | Buckets per hash |
| `.with_hash_set_size(usize)` | `1` | Minimum hashes per bucket |
| `.with_rotation(bool)` | `true` | Fill empty buckets from neighbors |

---

### Advanced Token Graph Filters

#### MultiplexerTokenFilter

Runs tokens through multiple sub-filter chains independently, emitting all variants.

```rust
use pizza_analysis_core::{MultiplexerTokenFilter, AsciiFoldingTokenFilter, LowercaseTokenFilter};
use pizza_engine::analysis::TokenFilter;

let filter = MultiplexerTokenFilter::new(vec![
    Box::new(LowercaseTokenFilter::new()),
    Box::new(AsciiFoldingTokenFilter::new()),
]).with_preserve_original(true);

// Token "Résumé" → emits "résumé" (lowercased) + "Resume" (folded) + "Résumé" (original)
```

| Method | Default | Description |
|--------|---------|-------------|
| `.with_preserve_original(bool)` | `true` | Keep original token |

---

#### ConditionalTokenFilter

Applies a sub-filter only to tokens matching a predicate.

```rust
use pizza_analysis_core::{ConditionalTokenFilter, MinLengthPredicate, LowercaseTokenFilter};
use pizza_engine::analysis::TokenFilter;

// Only lowercase tokens >= 4 characters
let filter = ConditionalTokenFilter::new(
    Box::new(MinLengthPredicate(4)),
    Box::new(LowercaseTokenFilter::new()),
);
```

**Built-in predicates:**
| Predicate | Description |
|-----------|-------------|
| `MinLengthPredicate(usize)` | Token length ≥ N |
| `MaxLengthPredicate(usize)` | Token length ≤ N |
| `PatternPredicate::new(regex)` | Token matches regex pattern |

---

#### PredicateTokenFilter

Removes tokens based on script type or custom predicate.

```rust
use pizza_analysis_core::{PredicateTokenFilter, ScriptType, TokenPredicateType};

// Keep only Latin script tokens
let filter = PredicateTokenFilter::new(TokenPredicateType::ScriptIs(ScriptType::Latin));
```

**Script types:** `Latin`, `Cyrillic`, `Arabic`, `Devanagari`, `Han`, `Hangul`, `Hiragana`, `Katakana`, `Thai`, `Greek`, `Hebrew`, `Other`

---

#### FlattenGraphTokenFilter

Flattens a token graph (produced by synonym graph or word delimiter graph filters) into a linear stream suitable for indexing.

```rust
use pizza_analysis_core::FlattenGraphTokenFilter;
let filter = FlattenGraphTokenFilter::new();
```

---

### Payload & Metadata

#### DelimitedPayloadTokenFilter

Extracts payload data from tokens in format `term|payload`.

```rust
use pizza_analysis_core::DelimitedPayloadTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = DelimitedPayloadTokenFilter::new('|');
let mut token = Token::new("pizza|0.95", 0, 10, 0);
filter.filter(&mut token);
assert_eq!(token.term, "pizza");
// Payload "0.95" extracted (stored separately)
```

---

#### DelimitedTermFreqTokenFilter

Extracts term frequency from tokens in format `term|freq`.

```rust
use pizza_analysis_core::DelimitedTermFreqTokenFilter;
use pizza_engine::analysis::{Token, TokenFilter};

let filter = DelimitedTermFreqTokenFilter::new('|');
let mut token = Token::new("pizza|5", 0, 7, 0);
filter.filter(&mut token);
assert_eq!(token.term, "pizza");
// Term frequency 5 extracted
```

---

## Language Analyzers

Pre-composed language analyzers follow Elasticsearch/Lucene conventions. Each is registered by name and can be retrieved from `AnalysisFactory`.

> **Snowball stemmers:** Some languages (e.g. `polish`, `swedish`, `turkish`,
> `armenian`, `basque`, `catalan`, `estonian`, `lithuanian`) are registered
> here as `lowercase + stop` only. To enable full Snowball stemming for these
> languages, also call
> `pizza_analysis_stemmers::register_all(&mut factory)` after
> `pizza_analysis_core::analyzers::register_all`. See
> [`pizza-analysis-stemmers`](https://github.com/pizza-rs/analysis-stemmers).

### Utility Analyzers

| Analyzer | Pipeline | Use Case |
|----------|----------|----------|
| `keyword` | KeywordTokenizer (no filters) | Exact-match fields |
| `simple` | LetterTokenizer → Lowercase | Basic word splitting |
| `stop` | LetterTokenizer → Lowercase → English Stop | English with stop removal |
| `pattern` | PatternTokenizer (default `\W+`) → Lowercase | Regex-based splitting |
| `fingerprint` | StandardTokenizer → Lowercase → AsciiFolding → Stop → Fingerprint | Deduplication |

### Language Analyzer Pipelines

Each language analyzer is tuned for its language with appropriate normalization, stemming, and stop word removal:

| Analyzer | Pipeline |
|----------|----------|
| **arabic** | Standard → Lowercase → DecimalDigit → ArabicNorm → Stop → ArabicStem |
| **bengali** | Standard → Lowercase → DecimalDigit → IndicNorm → BengaliNorm → Stop → BengaliStem |
| **brazilian** | Standard → Lowercase → Stop → BrazilianStem |
| **bulgarian** | Standard → Lowercase → Stop → BulgarianStem |
| **catalan** | Standard → Elision(l,d,qu,m,n,s) → Lowercase → Stop |
| **cjk** | Standard → CjkWidth → Lowercase → CjkBigram → Stop |
| **czech** | Standard → Lowercase → Stop → CzechStem |
| **danish** | Standard → Lowercase → Stop → ScandinavianNorm → ScandinavianFolding |
| **dutch** | Standard → Lowercase → Stop → DutchStem |
| **english** | Standard → Lowercase → Stop |
| **finnish** | Standard → Lowercase → Stop → FinnishLightStem |
| **french** | Standard → Elision(french) → Lowercase → Stop → FrenchLightStem |
| **galician** | Standard → Lowercase → Stop → GalicianStem |
| **german** | Standard → Lowercase → Stop → GermanNorm → GermanLightStem |
| **greek** | Standard → GreekLowercase → Stop → GreekStem |
| **hindi** | Standard → Lowercase → DecimalDigit → IndicNorm → HindiNorm → Stop → HindiStem |
| **hungarian** | Standard → Lowercase → Stop → HungarianLightStem |
| **indonesian** | Standard → Lowercase → Stop → IndonesianStem |
| **irish** | Standard → IrishElision → IrishLowercase → Stop |
| **italian** | Standard → Elision(italian) → Lowercase → Stop → ItalianLightStem |
| **latvian** | Standard → Lowercase → Stop → LatvianStem |
| **marathi** | Standard → Lowercase → DecimalDigit → IndicNorm → Stop |
| **nepali** | Standard → Lowercase → DecimalDigit → IndicNorm → Stop |
| **norwegian** | Standard → Lowercase → Stop → NorwegianLightStem |
| **persian** | PatternReplace(ZWNJ→space) + Standard → Lowercase → DecimalDigit → ArabicNorm → PersianNorm → Stop |
| **portuguese** | Standard → Lowercase → Stop → PortugueseLightStem |
| **romanian** | Standard → Lowercase → Stop → RomanianNorm |
| **russian** | Standard → Lowercase → Stop → RussianLightStem |
| **serbian** | Standard → Lowercase → Stop → SerbianNorm |
| **sorani** | Standard → SoraniNorm → Lowercase → DecimalDigit → Stop |
| **spanish** | Standard → Lowercase → Stop → SpanishLightStem |
| **swedish** | Standard → Lowercase → Stop → ScandinavianNorm → ScandinavianFolding |
| **tamil** | Standard → Lowercase → DecimalDigit → IndicNorm → Stop → TamilStem |
| **thai** | Thai → Lowercase → DecimalDigit → Stop |
| **turkish** | Standard → Apostrophe → TurkishLowercase → Stop |
| **urdu** | Standard → Lowercase → DecimalDigit → IndicNorm → Stop |

### Stop-Only Analyzers

These languages have stop word removal but no specialized stemmer available in this crate:

`afrikaans`, `amharic`, `armenian`, `azerbaijani`, `basque`, `croatian`, `estonian`, `filipino`, `georgian`, `hebrew`, `lithuanian`, `malay`, `mongolian`, `polish`, `slovak`, `slovenian`, `swahili`, `tagalog`, `ukrainian`, `vietnamese`

Pipeline: Standard → Lowercase → Stop

> **Tip:** For Snowball-based stemming on these languages, add `pizza-analysis-stemmers` which provides 33 algorithmic stemmer algorithms.

### Custom Analyzer Composition

Build your own analyzer by combining any normalizers, tokenizer, and filters:

```rust
use pizza_analysis_core::*;
use pizza_engine::analysis::{Analyzer, Normalizer, StandardTokenizer, TokenFilter};

// Custom e-commerce analyzer
let normalizers: Vec<Box<dyn Normalizer>> = vec![
    Box::new(HtmlStripNormalizer::new()),
    Box::new(PatternReplaceNormalizer::new(r"\b(SKU|sku):\s*", "")),
];

let filters: Vec<Box<dyn TokenFilter>> = vec![
    Box::new(LowercaseTokenFilter::new()),
    Box::new(AsciiFoldingTokenFilter::new()),
    Box::new(StopTokenFilter::new(
        token_filters::stopwords::get_stop_words("english").unwrap()
    )),
    Box::new(KStemTokenFilter::new()),
    Box::new(LengthTokenFilter::new(2, 50)),
];

let analyzer = Analyzer::new(
    normalizers,
    Box::new(StandardTokenizer::new()),
    filters,
);
```

---

## Stop Word Lists

Pre-built stop word lists for **57 languages**, accessible via the `token_filters::stopwords` module.

### Supported Languages

| Language | Constant | Language | Constant |
|----------|----------|----------|----------|
| Afrikaans | `AFRIKAANS_STOP_WORDS` | Latvian | `LATVIAN_STOP_WORDS` |
| Amharic | `AMHARIC_STOP_WORDS` | Lithuanian | `LITHUANIAN_STOP_WORDS` |
| Arabic | `ARABIC_STOP_WORDS` | Malay | `MALAY_STOP_WORDS` |
| Armenian | `ARMENIAN_STOP_WORDS` | Marathi | `MARATHI_STOP_WORDS` |
| Azerbaijani | `AZERBAIJANI_STOP_WORDS` | Mongolian | `MONGOLIAN_STOP_WORDS` |
| Basque | `BASQUE_STOP_WORDS` | Nepali | `NEPALI_STOP_WORDS` |
| Bengali | `BENGALI_STOP_WORDS` | Norwegian | `NORWEGIAN_STOP_WORDS` |
| Brazilian Portuguese | `BRAZILIAN_STOP_WORDS` | Persian | `PERSIAN_STOP_WORDS` |
| Bulgarian | `BULGARIAN_STOP_WORDS` | Polish | `POLISH_STOP_WORDS` |
| Catalan | `CATALAN_STOP_WORDS` | Portuguese | `PORTUGUESE_STOP_WORDS` |
| Chinese | `CHINESE_STOP_WORDS` | Romanian | `ROMANIAN_STOP_WORDS` |
| CJK (generic) | `CJK_STOP_WORDS` | Russian | `RUSSIAN_STOP_WORDS` |
| Croatian | `CROATIAN_STOP_WORDS` | Serbian | `SERBIAN_STOP_WORDS` |
| Czech | `CZECH_STOP_WORDS` | Slovak | `SLOVAK_STOP_WORDS` |
| Danish | `DANISH_STOP_WORDS` | Slovenian | `SLOVENIAN_STOP_WORDS` |
| Dutch | `DUTCH_STOP_WORDS` | Sorani Kurdish | `SORANI_STOP_WORDS` |
| English | `ENGLISH_STOP_WORDS` | Spanish | `SPANISH_STOP_WORDS` |
| Estonian | `ESTONIAN_STOP_WORDS` | Swahili | `SWAHILI_STOP_WORDS` |
| Filipino | `FILIPINO_STOP_WORDS` | Swedish | `SWEDISH_STOP_WORDS` |
| Finnish | `FINNISH_STOP_WORDS` | Tagalog | `TAGALOG_STOP_WORDS` |
| French | `FRENCH_STOP_WORDS` | Tamil | `TAMIL_STOP_WORDS` |
| Galician | `GALICIAN_STOP_WORDS` | Thai | `THAI_STOP_WORDS` |
| Georgian | `GEORGIAN_STOP_WORDS` | Turkish | `TURKISH_STOP_WORDS` |
| German | `GERMAN_STOP_WORDS` | Ukrainian | `UKRAINIAN_STOP_WORDS` |
| Greek | `GREEK_STOP_WORDS` | Urdu | `URDU_STOP_WORDS` |
| Hebrew | `HEBREW_STOP_WORDS` | Vietnamese | `VIETNAMESE_STOP_WORDS` |
| Hindi | `HINDI_STOP_WORDS` | | |
| Hungarian | `HUNGARIAN_STOP_WORDS` | | |
| Indonesian | `INDONESIAN_STOP_WORDS` | | |
| Irish | `IRISH_STOP_WORDS` | | |
| Italian | `ITALIAN_STOP_WORDS` | | |
| Japanese | `JAPANESE_STOP_WORDS` | | |
| Korean | `KOREAN_STOP_WORDS` | | |

### Dynamic Language Lookup

```rust
use pizza_analysis_core::token_filters::stopwords::get_stop_words;

// Look up by language name
if let Some(words) = get_stop_words("french") {
    println!("French has {} stop words", words.len());
}

// Also supports underscore-wrapped format (ES-compatible)
let words = get_stop_words("_german_").unwrap();
```

---

## Full Pipeline Examples

### Example 1: Basic English Pipeline

```rust
use pizza_analysis_core::*;
use pizza_engine::analysis::{Normalizer, Tokenizer, Token, TokenFilter};

// 1. Normalize: strip HTML
let normalizer = HtmlStripNormalizer::new();
let mut text = String::from("<p>The Quick Brown Fox</p>");
normalizer.normalize(&mut text);
// text = " The Quick Brown Fox "

// 2. Tokenize
let tokenizer = LetterTokenizer::new();
let mut tokens = tokenizer.tokenize(&text);
// ["The", "Quick", "Brown", "Fox"]

// 3. Lowercase
let lowercase = LowercaseTokenFilter::new();
for token in &mut tokens {
    lowercase.filter(token);
}
// ["the", "quick", "brown", "fox"]

// 4. Remove stop words
let stop_words = token_filters::stopwords::get_stop_words("english").unwrap();
let stop = StopTokenFilter::new(stop_words);
tokens.retain(|token| {
    let mut t = token.clone();
    let (remove, _) = stop.filter(&mut t);
    !remove
});
// ["quick", "brown", "fox"]
```

### Example 2: German Compound Analysis

```rust
use pizza_analysis_core::*;
use pizza_engine::analysis::{StandardTokenizer, Tokenizer, Token, TokenFilter};

let tokenizer = StandardTokenizer::new();
let mut tokens = tokenizer.tokenize("Donaudampfschifffahrtsgesellschaft");

let lowercase = LowercaseTokenFilter::new();
let stop = StopTokenFilter::new(
    token_filters::stopwords::get_stop_words("german").unwrap()
);
let norm = GermanNormalizationTokenFilter::new();
let stem = GermanLightStemTokenFilter::new();
let decomp = DictionaryDecompounderTokenFilter::new(
    vec!["donau".into(), "dampf".into(), "schiff".into(),
         "fahrt".into(), "gesellschaft".into()]
).with_min_subword_size(4);

for token in &mut tokens {
    lowercase.filter(token);
    norm.filter(token);
    stem.filter(token);
}
// Tokens processed through German normalization + stemming
// Decompounding produces sub-words for compound terms
```

### Example 3: Autocomplete (Edge N-gram at Index Time)

```rust
use pizza_analysis_core::*;
use pizza_engine::analysis::{Analyzer, StandardTokenizer, TokenFilter, Normalizer};

// Index-time analyzer: generate prefixes
let index_analyzer = Analyzer::new(
    vec![],
    Box::new(StandardTokenizer::new()),
    vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(EdgeNgramTokenFilter::new(2, 15)),
    ],
);

// Search-time analyzer: just lowercase (no edge-grams)
let search_analyzer = Analyzer::new(
    vec![],
    Box::new(StandardTokenizer::new()),
    vec![
        Box::new(LowercaseTokenFilter::new()),
    ],
);
```

### Example 4: Phonetic "Sounds Like" Search

```rust
use pizza_analysis_core::*;
use pizza_engine::analysis::{Analyzer, StandardTokenizer, TokenFilter};

let analyzer = Analyzer::new(
    vec![],
    Box::new(StandardTokenizer::new()),
    vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(PhoneticTokenFilter::new(PhoneticEncoder::DoubleMetaphone(6))
            .with_replace(false)), // Keep original + phonetic
    ],
);
// "Stephen" → ["stephen", "STFN"] (both indexed at same position)
// Matches queries for "Steven", "Stefan", etc.
```

### Example 5: Multi-language with Synonyms

```rust
use pizza_analysis_core::*;
use pizza_engine::analysis::{Analyzer, StandardTokenizer, TokenFilter};

let mut synonyms = SynonymTokenFilter::new(true); // case-insensitive
synonyms.add_equivalence(&["laptop", "notebook", "portable computer"]);
synonyms.add_mapping("ny", &["new york"], SynonymMode::Expand);

let analyzer = Analyzer::new(
    vec![],
    Box::new(StandardTokenizer::new()),
    vec![
        Box::new(LowercaseTokenFilter::new()),
        Box::new(synonyms),
        Box::new(StopTokenFilter::new(
            token_filters::stopwords::get_stop_words("english").unwrap()
        )),
    ],
);
```

---

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | ✓ | Standard library support |
| (none) | | `no_std` compatible (uses `alloc` only) |

---

## Related Crates

| Crate | Description |
|-------|-------------|
| [`pizza-engine`](../../lib/engine) | Core engine: `Normalizer`, `Tokenizer`, `TokenFilter`, `Analyzer` traits |
| [`pizza-analysis-all`](https://github.com/pizza-rs/analysis-all) | Auto-generated meta-crate — one `register_all()` that wires every discovered plugin |
| [`pizza-plugin-discovery`](https://github.com/pizza-rs/plugin-discovery) | CLI tool that scans contrib crates and (re-)generates `pizza-analysis-all` |
| [`pizza-analysis-stemmers`](https://github.com/pizza-rs/analysis-stemmers) | Snowball stemming algorithms (33 languages) |
| [`pizza-analysis-ik`](https://github.com/pizza-rs/analysis-ik) | IK Chinese segmenter (smart/max_word modes) |
| [`pizza-analysis-jieba`](https://github.com/pizza-rs/analysis-jieba) | Jieba Chinese segmenter |
| [`pizza-analysis-pinyin`](https://github.com/pizza-rs/analysis-pinyin) | Chinese Pinyin tokenizer + filter |
| [`pizza-analysis-stconvert`](https://github.com/pizza-rs/analysis-stconvert) | Simplified ↔ Traditional Chinese conversion |

---

## License

MIT

---

<div align="center">
<sub>Part of the <a href="https://pizza.rs">INFINI Pizza</a> ecosystem</sub>
</div>
