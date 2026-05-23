# Pizza Analysis Core — Cross-Review vs Lucene / Elasticsearch / Tantivy

Date: 2026-05-23
Reviewer: cross-audit pass after four rounds of per-file bug fixes
(see [SECOND_AUDIT.md](SECOND_AUDIT.md) and
[SECOND_AUDIT_TODO.md](SECOND_AUDIT_TODO.md) rounds 1-4).

This document is a **structural and behavioural** comparison. The pure
correctness work (UTF-8 panics, off-by-ones, dead branches) is already
covered by the four-round audit. Here we ask: *given the components are
individually safe, does the library behave like Lucene/ES/Tantivy when
plugged together, and where does it diverge?*

---

## Scope of the crate

```
 38 tokenizers
~110 token filters
  7 normalizers (incl. one re-exported from pizza-engine)
 11 pre-composed language analyzers
```

That puts Pizza in the same weight class as Lucene's `analysis-common` +
the language `analysis-*` modules combined, and **much** larger than
Tantivy's analyzer crate (`tantivy/src/tokenizer/`, ~12 tokenizers, ~10
filters) or Meilisearch's `charabia` (segmenter-only, no filter pipeline).

The closest peer in surface area is Lucene's analyzer family. Tantivy
deliberately keeps its analyzer count small and pushes language-specific
work to user code; Meilisearch handles it at a different layer (charabia
+ deunicode). So the most informative comparison is Pizza ↔ Lucene/ES.

---

## A. Architectural divergences from Lucene

### A1. `TokenFilter` has no end-of-stream hook  🟥 HIGH

The Pizza `TokenFilter` trait exposes only:
```rust
fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>);
```
Per-token, with no `flush()` / `end()` / `incrementToken() -> false`
boundary. Lucene's `TokenFilter` is a *pull* stream where filters can:

- Buffer tokens and emit them at end-of-stream
  (`FingerprintFilter`, `LimitTokenCountFilter`, `SortedTermFilter`)
- Emit one synthetic token after the last input
  (`MinHashFilter`, `ConcatenateGraphFilter`)
- Clear stateful buffers via `reset()` between documents

Pizza filters that fundamentally need this hook all work around it the
same way: shared `Arc<Mutex<State>>` inside the filter struct, plus an
**unscheduled** `reset()` method that callers must remember.

Files affected:

| File | Behaviour |
|---|---|
| `token_filters/fingerprint.rs` | `filter()` returns `(true, None)` for every token; **the analyzer pipeline emits zero tokens**. Result is hidden behind `take_fingerprint()`. |
| `token_filters/shingle.rs` | Per-token shingle emission "works" but offsets are wrong (see B2) and the buffer leaks across documents. |
| `token_filters/common_grams.rs` | Two-token state survives document boundaries — first token of doc N is bigram'd with last token of doc N-1. |
| `token_filters/concatenate_graph.rs` | Same `Arc<Mutex>` accumulation pattern. |

**Recommendation:** add `fn flush(&self) -> Vec<Token<'static>>` and
`fn reset(&self)` to the `TokenFilter` trait in `pizza-engine`, then drive
both from the analyzer at end-of-stream and start-of-document. This is
the largest single API change suggested by this review.

### A2. Shared filter instances + interior mutability ≠ thread-safe  🟥 HIGH

`register_all` registers ONE instance of each filter into the global
`AnalysisFactory`:
```rust
factory.register_token_filter("fingerprint", Box::new(FingerprintTokenFilter::new()));
```
Combined with the `Arc<Mutex<...>>` pattern from A1, this means:

- Document A and document B analyzed concurrently share the same buffer.
- Without an enforced `reset()` between documents, buffers grow
  unboundedly and produce wrong output.
- The crate is `no_std`-tagged (see B6) but uses `std::sync::Mutex`,
  forcing std on consumers anyway.

Lucene avoids this by instantiating new `TokenFilter` per stream via
`Analyzer.createComponents(fieldName)`. Tantivy avoids it with
`TokenStream` objects that own per-call state.

**Recommendation:** make `TokenFilter` construction a *factory* call
(`fn build(&self) -> Box<dyn TokenFilterInstance>`), so stateful filters
get a fresh instance per document. Until that lands, document loudly
that `FingerprintTokenFilter`, `ShingleTokenFilter`, `CommonGramsTokenFilter`,
and `ConcatenateGraphTokenFilter` are **not** safe under concurrent or
multi-document use.

### A3. No graph/positionLength support  🟧 MEDIUM

Lucene's `PositionLengthAttribute` lets a filter emit a single token
that "spans" multiple positions (e.g. `WordDelimiterGraphFilter` emits
`wifi` at position 0 with length 2 alongside `wi` (len 1) + `fi` (len 1)
at positions 0 and 1). This is essential for correct phrase matching
on shingles, synonyms, and word-delimiter output.

Pizza's `Token`:
```rust
pub struct Token<'a> {
    pub term: Cow<'a, str>,
    pub start_offset: u32,
    pub end_offset: u32,
    pub position: u32,
}
```
…has no `position_length`. Consequences:

- `WordDelimiterGraphTokenFilter` (name notwithstanding) cannot
  represent the graph correctly — `concatenate_words` output sits at
  the same `position` as the first part, but the engine has no way to
  know it spans the whole original token.
- `SynonymTokenFilter` only supports single-token synonyms (see B4).
- Phrase queries over shingled fields will mis-match.

**Recommendation:** add `position_length: u32` (default 1) to `Token`.
This is a one-field change with large downstream value.

### A4. No Lucene-style "type" attribute  🟧 MEDIUM

Lucene tokens carry `TypeAttribute` (`<ALPHANUM>`, `<NUM>`, `<HOST>`,
`<EMAIL>`, etc.) which `ClassicTokenizer`, `KeepTypesTokenFilter`,
`TypeAsSynonymFilter`, etc. all use.

Pizza's `token_filters/keep_types.rs` and `type_as_synonym.rs` exist but
have nothing to read because the `Token` struct has no type field. They
necessarily fake it (and are likely no-ops or pattern-matchers). Worth
checking individually whether they are functional or vestigial.

---

## B. Per-file behavioural divergences

Listed by severity. These are *not* panics (those are fixed); they are
spec-deviations where a Lucene/ES user would get surprising output.

### B1. `tokenizers/sliding_window.rs` — not a Lucene component  🟦 INFO

No counterpart in Lucene. Closest analogue is Lucene's `EdgeNGramFilter`
applied per-word, but the semantics differ. The off-by-one fix in
round 3 made it self-consistent; just flagging that there is no spec to
diverge from here.

### B2. `token_filters/shingle.rs` — offsets wrong  🟥 HIGH

```rust
shingles.push(Token {
    term: Cow::Owned(shingle),
    start_offset: token.start_offset,   // ← latest token's start
    end_offset:   token.end_offset,     // ← latest token's end
    position:     token.position,
});
```
For shingle `"the quick"`, Lucene's `ShingleFilter` sets
`start_offset = offsetOf("the")` and `end_offset = offsetOf("quick")`.
Pizza sets both to `quick`'s offsets, so highlighting and snippet
extraction over a shingled field will underline the wrong text.

**Fix sketch:** push original `Token` (not just terms) into the buffer
so the first/last offsets can be recovered.

### B3. `token_filters/common_grams.rs` — bigram offsets + position  🟧 MEDIUM

Same offset bug as B2 plus a position bug: Lucene's `CommonGramsFilter`
emits the bigram at the position of the FIRST (common) word, then the
following unigram at the next position. Pizza emits the bigram at the
CURRENT (second) word's position. Phrase queries spanning common words
will produce off-by-one positions.

### B4. `token_filters/synonym.rs` — single-token only  🟧 MEDIUM (scope)

No multi-word synonym support. Lucene's `SynonymGraphFilter` (since 6.6)
handles `"new york" => nyc` and emits a graph token. Pizza can map
`"york" => "ny"` but not the multi-word form.

This may be a deliberate scope decision; if so, document it. Otherwise
needs the position-length attribute (A3) + lookahead buffering (A1).

### B5. `token_filters/asciifolding.rs` — incomplete coverage  🟦 INFO

The fold table covers Latin-1 + Latin Extended-A and ß / Æ / Œ / Þ. It
omits everything Lucene's `ASCIIFoldingFilter` adds: IPA Extensions,
Letterlike Symbols, Mathematical Alphanumerics, Fullwidth Latin,
Spacing Modifier Letters, the box of Latin Extended-B, etc.

Lucene's table is ~3,000 codepoints (`ASCIIFoldingFilter.foldToASCII`).
Pizza's is ~80. Real-world impact: Vietnamese, IPA-annotated text,
mathematical alphanumeric letters, fullwidth ASCII (Japanese forms)
will not fold.

**Recommendation:** generate the table from Lucene's source or from the
Unicode `Decomposition_Mapping` data via a `build.rs`.

### B6. `lib.rs` claims `no_std` but pulls `std`  🟧 MEDIUM (honesty)

```toml
[features]
default = []
std = ["pizza-engine/std"]
```
```rust
#![cfg_attr(not(feature = "std"), no_std)]
```
…yet 4 files use `std::sync::Mutex` unconditionally and the `regex`
dependency requires `std`. The crate is **not** no_std-buildable in
practice. Either:
- Drop the `no_std` claim and the `cfg_attr`, or
- Move the `std::sync::Mutex` filters behind a `std` feature gate, or
- Switch to `spin::Mutex` / a `lock_api`-based mutex and gate `regex`
  behind a `regex` feature.

### B7. `tokenizers/classic.rs` — O(n²) byte/char conversion  🟦 PERF

```rust
fn char_byte_offset(text: &str, char_index: usize) -> usize {
    text.char_indices().nth(char_index).map(|(i, _)| i).unwrap_or(text.len())
}
```
Called per emitted token, each call walks the string from byte 0. For
long inputs this is quadratic. Lucene's `ClassicTokenizer` tracks byte
offsets directly in the JFlex-generated scanner. Tantivy's tokenizers
maintain a running `(byte_pos, char_pos)` cursor.

**Recommendation:** pre-compute `chars: Vec<(usize, char)>` (already
done in many siblings) and look up by index.

### B8. `tokenizers/structured_id.rs` — position semantics  🟦 INFO

Emits the original ID at position N, then each sub-part at N+1, N+2,
… Lucene's convention for "original + parts" filters (see
`WordDelimiterGraphFilter`) puts the original AND the first part at
position N, with subsequent parts at N+1, N+2 (and uses
`positionLength` for the original to span all parts).

This is a phrase-query correctness issue: a query `"PROD A 123 XL"`
against a doc containing `PROD-A-123-XL` analyzed with this tokenizer
will fail to match because the original consumes a position slot.

### B9. Position-length on word-delimiter output  🟧 MEDIUM

Already implied by A3, but specifically: the file is named
`word_delimiter_graph.rs` after Lucene's `WordDelimiterGraphFilter`.
Without `position_length` on the `Token` struct, the file cannot
fulfill the contract its name implies — it is structurally a
`WordDelimiterFilter` (Lucene-deprecated since 6.5 because it breaks
phrase queries).

---

## C. Comparison snapshot

| Concern | Lucene/ES | Tantivy | Pizza | Notes |
|---|---|---|---|---|
| `flush()` / end-of-stream | yes (`incrementToken()` returns false) | yes (`TokenStream` iterator) | **no** | Forces all stateful filters into Arc<Mutex> hacks. |
| `positionLength` attribute | yes | n/a | **no** | Graph filters can't be correct without it. |
| `type` attribute | yes | no (custom payload) | **no** | `KeepTypesTokenFilter` is vestigial. |
| Per-document filter instance | yes (factory pattern) | yes | **no** (singleton) | Leaks state across documents. |
| Multi-word synonyms | yes (`SynonymGraphFilter`) | no | **no** | |
| ASCIIFolding coverage | ~3000 cp | ~3000 cp (via deunicode) | ~80 cp | Vietnamese / mathematical alphabets miss. |
| ICU rule-based segmentation | yes (icu addon) | yes (`tantivy-tokenizer-api` impls) | partial (custom heuristics) | CJK, Thai, Burmese handled ad-hoc. |
| Hunspell / dictionary stemmers | yes | no | yes (`hunspell.rs`) | Pizza wins here. |
| Phonetic algorithms | yes (Beider-Morse, etc.) | no | yes (`beider_morse.rs`, `phonetic.rs`) | Pizza wins here. |
| Per-language stemmers count | ~30 | ~17 (rust-stemmers) | ~40+ | Pizza wins here. |
| Pre-tokenization normalizers | minimal (CharFilter chain) | minimal | rich (HTML strip, mapping, pattern replace, NFC/NFD/NFKC/NFKD) | Pizza wins here. |

---

## D. What's solid

Even with the criticisms above, several things are notably **better**
than Tantivy and on par with Lucene:

1. **Breadth of language stemmers + normalizers**: 30+ Latin scripts,
   Indic, Sorani, Tibetan, Cherokee, Khmer, Navajo, Yiddish, Welsh,
   Irish — far beyond Tantivy's `rust-stemmers` integration.
2. **Domain-specific tokenizers**: `LogTokenizer`, `CodeTokenizer`,
   `MicroBlogTokenizer`, `EmailTokenizer`, `JsonFieldTokenizer`,
   `PhoneNumberTokenizer`. Lucene only has a few of these (most via
   `analysis-extras`); Tantivy has none.
3. **Normalizer pipeline as a first-class stage** is cleaner than
   Lucene's `CharFilter` chain (which interleaves offsets confusingly)
   and Tantivy (which has no normalizers).
4. **No-panic posture** (after the four-round audit): the round-1-to-4
   work removed every reachable panic on adversarial Unicode input we
   could find. That's something Lucene itself does *not* uniformly
   guarantee — `KuromojiTokenizer` has had multiple CVEs over the years
   for malformed CJK input.

---

## E. Prioritised follow-up

Items are independent unless noted; each can be tackled in isolation.

1. **(A1+A2)** Switch `TokenFilter` trait to per-document instances
   *or* add `flush()` + `reset()` + factory wrapper. This unblocks
   fingerprint/shingle/common-grams correctness.
2. **(A3)** Add `position_length: u32` to `Token`. One line in
   `pizza-engine`, plus updating shingle / word-delimiter-graph /
   (future) synonym-graph to populate it.
3. **(B2, B3)** Fix shingle/common-grams offsets to span first→last
   token. Trivial after (1) gives a real buffer.
4. **(B5)** Generate ASCII-fold table from Unicode data via `build.rs`.
   No design questions, mechanical work.
5. **(B6)** Pick one of {drop no_std, gate properly, port to spin}.
   Pick before the lie hardens further.
6. **(B7)** Tighten `classic.rs` char→byte indexing. Easy O(n²)→O(n).
7. **(A4)** Add `type` attribute or remove `keep_types.rs` /
   `type_as_synonym.rs`. Either is fine; current state is misleading.

Items 1, 2 are API-breaking and warrant a single coordinated bump.
Items 3-7 are local and can land independently.

---

## F. What this review did NOT cover

- Performance benchmarks vs Lucene/Tantivy (would need a corpus + jmh
  harness).
- Memory-safety under concurrent `Send`/`Sync` use beyond the
  `Arc<Mutex>` issue (no fuzz harness in the repo).
- Correctness of stemming output vs Snowball reference. Pizza has its
  own stemmers; per-language conformance tests against the Snowball
  test data are absent.
- The unbuildable filters (anything outside what `register_all` wires
  up) — there are ~30 such files. They compile but are not exercised
  end-to-end. Suggest either wire them up or move them under
  `experimental/` so contributors know.
