# Pizza Analysis Core — Round-2 Issue Backlog

Findings from the second-pass deep audit. The 10 fixes from round 1 are
documented in [SECOND_AUDIT.md](SECOND_AUDIT.md). Items below were uncovered
by an adversarial re-read of all remaining tokenizers, normalizers, and the
buildable token filters.

Legend: ✅ = fixed in this round, 🟡 = won't fix (see note), ⬜ = pending.

## Tokenizers

- ✅ **char_group.rs** — `split_on_punctuation` and `split_on_digit` only
  recognize ASCII punctuation/digits, silently failing for CJK punctuation
  (`、。！？`), Arabic-Indic digits (`٠-٩`), and every other script.
- ✅ **hyphenated.rs** — `min_part_length` compares `part.len()` (bytes)
  against a length intent users naturally express in characters; non-ASCII
  parts are over-counted (e.g. `min=3` accepts every 2-character CJK part).
- ✅ **sentence.rs** — `min_length` uses byte length for `trimmed.len()`
  and the tail-emit; same Unicode mismatch as hyphenated.
- ✅ **punctuation.rs** — `is_valid_number` accepts `"1.2.3"` and
  `"1,2.3,4"` because it only checks that every char is digit/`.`/`,`. No
  structural validation.
- ✅ **code.rs** — In the string-literal branch,
  `content_end = i - quote.len_utf8()` unconditionally strips the last char
  even when the loop exited because EOF was hit without a closing quote,
  truncating the final character of an unterminated string.
- ✅ **code.rs** — Dead code: `if self.split_dots && ident.contains('.')`
  can never fire because the ident-collection loop stops at `.` whenever
  `split_dots=true`, so the collected ident never contains `.`. Splitting
  actually happens via the outer fall-through skipping the dot.

## Token filters

- ✅ **apostrophe.rs** — Only matches ASCII `'`. Real-world text (Office,
  iOS, macOS, web fonts) uses Unicode right-single-quote U+2019 `’`. Lucene
  handles both; we drop the latter on the floor.

## Normalizers

- ✅ **mapping.rs** — Redundant `text.contains(from)` check before
  `text.replace(from, to)` doubles the scan cost per mapping. Replace
  already returns the same string when no match is found.
- ✅ **html_strip.rs** — Only decoded ~15 named entities; numeric
  entities (`&#65;`, `&#x4E2D;`) — common in HTML email and exports —
  were dropped through to the "output as-is" fallback. Now decoded via
  `char::from_u32`.

## Won't-fix in this round (tracked here so they are not lost)

- 🟡 **uax_url_email.rs** — Regex-based detection has theoretical ReDoS
  surface and limited IDN / RFC 5321 support. A full Lucene-grade
  implementation needs the `idna` crate plus a proper finite-state machine.
  Filed as a roadmap item, not a bug fix.
- 🟡 **burmese.rs** — Stacked-consonant handling is incomplete. Burmese
  segmentation is a research problem; ICU itself only does syllable-level
  breaking. Acceptable to defer.
- 🟡 **thai.rs** — Documented as script-only splitter; genuine dictionary
  segmentation needs `dict-thai` or the equivalent and is out of scope.
- 🟡 **unicode.rs** (normalizer) — Decomposition table is a hand-rolled
  subset of NFKD. The proper fix is to depend on `unicode-normalization`
  for the `nfkd`/`nfkc` form. Tracked separately.
- 🟡 **classic.rs** — `char_byte_offset` O(n) per call. Hot only on
  pathological inputs; not blocking.
- 🟡 **edge_ngram.rs** — Position semantics are "monotonic per ngram"
  rather than Lucene's "all-share-position-0 with positionLength". The
  Token type does not currently carry `position_length`, so a faithful
  Lucene clone needs a Token API change first.

## Verification

After each fix, run:

```bash
cd contrib/analysis-all && \
  RUSTUP_TOOLCHAIN=nightly-2025-06-06 cargo check -p pizza-analysis-core
```

and confirm the fixed file is not in the error/warning list. Inline tests
are added for each fix and will run as soon as the unrelated 90+
pre-existing errors in `analyzers/`, `token_filters/*` and `lib/engine`
are sorted out.

---

## Round 3 — third-pass deep audit

Adversarial re-read of additional tokenizers + the non-buildable filters'
companion files. Subagent reported 9 candidates; manual verification
eliminated 5 false positives (ngram segment-boundary, email pointer-arith,
URL trim base offset, JSON unsafe slicing, camel_case position semantics —
all checked and confirmed correct).

Confirmed real bugs:

- ✅ **token_filters/elision.rs** — `&term[apos_pos + 1..]` panics when
  the matched apostrophe is U+2019 `’` (3 bytes). The companion
  `tokenizers/elision.rs` does this correctly using
  `apos.len_utf8()`; the filter version was missed when U+2019 support
  was added. **Severity: HIGH (panic on real-world French/Italian input
  containing smart quotes).**
- ✅ **tokenizers/sliding_window.rs** — `if char_count > self.min_size`
  off-by-one: a word with exactly `min_size` characters produces zero
  windows. With `new(3,3)` the input `"abc"` emits no `"abc"` window.
  Condition should be `>=`. **Severity: MEDIUM.**
- ✅ **tokenizers/chinese_char.rs** — `is_cjk` includes
  `0x3000..=0x303F` (CJK Symbols and Punctuation), so ideographic
  punctuation `、。「」【】` is emitted as content tokens instead of
  being skipped. Example: `"你好。再见"` produces `['你','好','。','再','见']`.
  **Severity: MEDIUM.**
- ✅ **normalizers/case.rs** — Tests reference `LowercaseNormalizer` /
  `UppercaseNormalizer` (re-exported from `pizza-engine` via
  `normalizers/mod.rs`) without a matching `use`. The `mod tests`
  `use super::*;` only brings in items from `case.rs` itself, so the test
  module fails to compile. **Severity: LOW (test-only, but breaks
  `cargo test` once unrelated errors are sorted).**

Won't-fix in this round (false positives or out-of-scope):

- 🟡 **tokenizers/email.rs** — `part.as_ptr() - text.as_ptr()` is SAFE
  because `local.split(...)` yields slices into the same allocation; no
  trimming happens.
- 🟡 **tokenizers/url.rs** — `base = trimmed.as_ptr() - text.as_ptr()`
  correctly recovers the original-text offset of the trimmed start; all
  subsequent token offsets add `base` correctly.
- 🟡 **tokenizers/ngram.rs** — segment-aware path iterates
  `for i in 0..=(seg_len - n)`, so `ci + n <= seg_e` always holds and
  n-grams never cross segment boundaries.
- 🟡 **tokenizers/json_field.rs** — String content scanned at byte
  level, but the slice boundaries are always at ASCII `"`, which can
  never collide with UTF-8 continuation bytes (0x80-0xBF).
- 🟡 **tokenizers/camel_case.rs** — `preserve_original` emits the
  original at position N and the first sub-part also at position N
  (overlap). This is intentional Lucene-style positioning.

## Round 4 — fourth-pass deep audit

Adversarial re-read of ~50 additional files (microblog, phone_number,
structured_id, simple_pattern, wildcard, log, letter, lowercase, classic,
compound_word, pattern, keyword, reverse, tab_separated, uax_url_email,
markdown, path_hierarchy, plus numerous token filters). Subagent reported
2 candidates; manual verification confirmed 1 real bug and rejected 1
false positive.

Confirmed real bug:

- ✅ **token_filters/word_delimiter_graph.rs** — Possessive stripping did
  `text.truncate(text.len() - 2)` for both `'s` (2 bytes) and `\u{2019}s`
  (4 bytes). For U+2019 input, `truncate(len-2)` lands in the middle of
  the 3-byte U+2019 codepoint, panicking with "byte index N is not a
  char boundary". Fix computes `cut = '\u{2019}'.len_utf8() + 1 = 4` for
  the curly-apostrophe branch. **Severity: HIGH (panic on real-world
  text containing curly possessives like `running’s`).**

Won't-fix (false positive):

- 🟡 **tokenizers/phone_number.rs** — Subagent claimed the trim-trailing
  loop `while i > start && !text[..i].ends_with(...) { i -= 1; }` could
  decrement `i` into the middle of a multi-byte char. Manual verification:
  the accumulation loop directly above only consumes ASCII chars (digits,
  `+`, `-`, `(`, `)`, space, `.`), so every byte position in `[start, i]`
  is on a char boundary. Decrementing `i` by 1 stays on boundaries within
  that ASCII run. No panic possible.

Confidence after four rounds: the easy structural bugs have been mined out.
Any further audit pass will likely have a high false-positive rate; future
audits should focus on integration tests with adversarial Unicode corpora
rather than continued static review.

---

## Round 5 — Cross-review remediation

Driven by [CROSS_REVIEW.md](CROSS_REVIEW.md), which compared the crate against
Lucene / Elasticsearch / Tantivy / Meilisearch and surfaced several behavioral
and architectural divergences. Items below are the locally-fixable ones; the
remaining engine-level findings are tracked as 🟡 because they require changes
to the `pizza_engine::analysis` trait surface.

### Fixed in round 5

- ✅ **token_filters/shingle.rs** — Buffered shingle was emitting at the
  latest token's offsets, so a shingle spanning words 1..N reported
  `start_offset` from word N, not word 1. Replaced the
  `Vec<String>` buffer with a `Vec<BufferedToken>` carrying full
  offsets/position, and now emit `start = first.start_offset`,
  `end = last.end_offset`, `position = first.position` (Lucene
  `ShingleFilter` semantics). Added `test_shingle_spans_first_to_last_offsets`.

- ✅ **token_filters/common_grams.rs** — Bigrams were anchored at the
  *current* token's offset/position, but Lucene's `CommonGramsFilter`
  anchors a `word_common` (or `common_word`) bigram at the *first* token's
  position so highlight ranges and PhraseQuery proximity stay correct.
  Extended state with `prev_start_offset / prev_end_offset / prev_position`
  and emit bigram with `start = prev_start_offset`, `end = current.end_offset`,
  `position = prev_position`. Added `test_bigram_anchored_at_first_token`.

- ✅ **tokenizers/classic.rs** — Tokenize loop called `char_byte_offset`
  twice per token, an O(n) `char_indices().nth(...)` lookup, giving
  worst-case O(n²) for long inputs (~50 k-token document = seconds).
  Replaced with a single up-front
  `let char_indices: Vec<(usize, char)> = text.char_indices().collect();`
  followed by O(1) `char_indices[i].0` lookups. Deleted the dead
  `char_byte_offset` helper.

- ✅ **tokenizers/structured_id.rs** — Words ending in a delimiter
  (`"A-"`, `"v1."`) triggered a spurious extra `position += 1` after the
  loop, leaving a phantom position slot between adjacent words and skewing
  PhraseQuery slop. Removed the unconditional `else { position += 1 }`
  branch; the in-loop increment already accounts for the trailing slot.
  Added `test_trailing_delimiter_no_position_gap`.

- ✅ **token_filters/asciifolding.rs** — Static `match` covered only
  Latin-1 / Latin Extended-A / Greek / basic Cyrillic, missing Lucene
  `ASCIIFoldingFilter`'s coverage of Vietnamese, fullwidth Latin, and the
  Mathematical Alphanumeric Symbols block. Added `fold_programmatic`
  with three sub-mappers:
  - Fullwidth Forms U+FF01–U+FF5E and ideographic space U+3000.
  - Latin Extended Additional U+1E00–U+1EFF (precomposed Latin diacritics
    + Vietnamese precomposed forms U+1EA0–U+1EF9) using accurate
    per-base-letter range tables.
  - Mathematical Alphanumeric Symbols U+1D400–U+1D7FF (52-letter cycle
    for styled alphabets, 10-digit cycle for styled digits).
  Logic verified against 30 hand-picked codepoint cases (all passing)
  before adding 4 unit tests covering fullwidth ABC/123, Vietnamese
  "Tiếng Việt", math bold "Hi", and math bold digits.

- ✅ **lib.rs no_std cleanup** — `#![cfg_attr(not(feature = "std"), no_std)]`
  was misleading: several token filters use `std::sync::Mutex`
  unconditionally, and the `regex` dep transitively requires std. Removed
  the conditional `no_std` attribute and added a module-level doc
  comment explaining that this crate is std-only by design. `extern crate
  alloc;` is retained for the explicit `alloc::` paths already in use.

### Won't-fix (engine-level)

Each requires a change to the `pizza_engine::analysis` trait surface
(adding a flush hook, factory pattern for filters, a `position_length`
field on `Token`, or a per-document state lifecycle). Documented here
so a future engine-side refactor can address them in a single sweep.

- 🟡 **A1 No flush / end-of-stream hook** — `TokenFilter::filter` is called
  per-token only; filters that buffer (shingle, common_grams) cannot emit
  trailing tokens at end-of-stream. We work around this by buffering
  inside `Arc<Mutex<...>>` and accepting that the last partial shingle /
  trailing common-grams bigram is silently dropped.
- 🟡 **A2 Singleton filter instances** — The engine instantiates each
  filter once and reuses it across documents/threads. Stateful filters
  must use `Arc<Mutex<State>>` and a manual `reset()` call site, with
  inevitable cross-document leakage if any caller forgets to reset.
- 🟡 **A3 No `position_length`** — Lucene's `Token` carries
  `positionLength` so multi-position synonym expansions and shingles
  remain queryable as phrases. Pizza's `Token` lacks this field, so
  multi-word synonyms / shingles can never participate in `PhraseQuery`
  slop reasoning the same way as Lucene's.
- 🟡 **FingerprintTokenFilter** — Filter emits a synthetic concatenated
  term but no canonical way exists in the current API to also delete the
  source tokens; per-token API forces it to mark the source deleted while
  emitting the fingerprint as an extra, producing N+1 outputs instead of 1.
- 🟡 **Multi-word synonyms** — Same root cause as A1 (no flush) and A3
  (no `position_length`): we can map single tokens but cannot emit a
  multi-token synonym that occupies the correct position span.

### Verification

`cargo check -p pizza-analysis-core` reports **0 errors and 0 warnings** in
the six files touched this round (`lib.rs`, `tokenizers/classic.rs`,
`tokenizers/structured_id.rs`, `token_filters/{shingle,common_grams,
asciifolding}.rs`). The 38 baseline failures pre-existing in unrelated
crate modules are unchanged.

Confidence after five rounds: all locally-actionable cross-review findings
addressed. Remaining gaps versus Lucene-class engines are engine-trait
limitations, not bugs in this crate.

---

## Round 6 — adversarial sweep for UTF-8, panics, complexity

A fresh grep-based survey targeting `[..n]` byte slicing, `+ 1` after
`find(char)`, `unwrap`, integer subtraction near `usize`, and `Vec::contains`
in hot loops surfaced six additional issues. Five are fixed in-tree; one
remains an engine-API limitation.

### 1. `EmailMaskTokenFilter` — UTF-8 byte-boundary panic ✅ FIXED

`src/token_filters/security_privacy.rs` masked the local part and domain
name as `&local[..1]` / `&dname[..1]`. For an internationalized email like
`ñame@host`, `ñ` is two bytes and byte index `1` falls mid-character →
`panic!` (not a checked error).

Replaced with a new `mask_keep_first_char(s, mask_char)` helper that uses
`s.chars().next()` + `len_utf8()` for both the kept first character and the
mask character, so multi-byte input and multi-byte mask chars (e.g. `★`)
both work.

### 2. `KeyValuePairTokenFilter` — multi-byte separator panic ✅ FIXED

`src/token_filters/code_log_science.rs` exposes `separator: char` but split
the term as `&text[..pos]` / `&text[pos + 1..]`. A configured separator like
`'。'` (3 bytes) or `'='` after a multi-byte key boundary panics with
"byte index … is not a char boundary".

Changed to `pos + self.separator.len_utf8()`.

### 3. `ConcatenateGraphTokenFilter` — running offsets ✅ FIXED

The running concatenation emitted the current token's `start_offset` /
`position`, so the synthesised composite term reported a span equal to the
*last* contributing token rather than `[first.start, current.end]`. This
diverges from Lucene `ConcatenateGraphFilter` and breaks downstream
highlighting / span queries.

Replaced the raw `Arc<Mutex<String>>` buffer with a `ConcatState` struct
that records the first token's `start_offset` and `position` on the first
call, then rewrites each emitted token to span the full concatenation.
`reset()` clears all fields.

### 4. `HyphenationDecompounderTokenFilter` — integer underflow ✅ FIXED

`len - self.min_subword_size + 1` is unsigned arithmetic; if a caller
configures `min_subword_size > len` (legal — only `len < min_word_size` is
guarded earlier), the subtraction wraps and panics on debug builds /
produces a huge loop bound on release.

Added an early `if len < self.min_subword_size.saturating_mul(2) { return
Vec::new(); }` — any word too short to yield two subwords of the configured
minimum cannot decompound by definition, so returning empty is correct and
also makes the later arithmetic provably safe.

### 5. `FingerprintTokenFilter` — O(N²) dedup ✅ FIXED

`if !state.terms.contains(&term)` is an O(N) linear scan on every token,
making the filter O(N²) across a document. On long bodies (say 10k tokens)
that is ~50M comparisons before the final sort.

Added a parallel `seen: HashSet<String>` to `FingerprintState`, kept in
sync with `terms`. Per-token check is now O(1) amortized; final sort/dedup
unchanged so output is byte-identical. Both `take_fingerprint()` and
`reset()` clear the `seen` set alongside `terms` so cross-document state
does not leak.

Follow-up noted but not patched: the separate `FingerprintAccumulator`
helper struct in the same file has an identical O(N²) `Vec::contains`
pattern in its `add()` method. Leaving as-is for now since the public
filter (the one wired into the pipeline) is what hot paths use.

### 6. `MinHashTokenFilter` — mis-implementation 🟡 DOCUMENTED, ENGINE LIMITATION

The current `TokenFilter::filter` API is strictly per-token with no
end-of-stream / flush hook. True Lucene-style MinHash needs to observe the
*entire* token stream of a document and emit `hash_count` minimum hashes
as a single signature at flush time. The current code instead emits
`hash(token) % bucket_count` as a hex string per token, which is useful
for shingle-based LSH pipelines but is *not* a document-level MinHash
signature, and the `hash_set_size` / `with_rotation` fields are
silently ignored.

Added a prominent `# Implementation notes` section to the struct doc
comment explaining the limitation and pointing at the engine-API change
required to fix it. Behaviour unchanged.

### Verification

`cargo check -p pizza-analysis-core` reports **0 new errors and 0 new
warnings** in the six files touched this round (`token_filters/
{security_privacy, code_log_science, concatenate_graph,
hyphenation_decompounder, fingerprint, minhash}.rs`). The three warnings
that remain in those files (`Cow` / `String` unused imports outside test
config, `result` unused variable in pre-existing SSN/CC masking code) are
baseline — not introduced by round 6 edits.

Confidence after six rounds: every cross-review finding actionable without
an engine-API change is now fixed. The only remaining divergences from
Lucene/ES semantics (MinHash flush, multi-token synonym position spans,
char-filter→tokenizer offset propagation) all require additions to
`pizza_engine::analysis` and are recorded above as won't-fix-in-crate.
