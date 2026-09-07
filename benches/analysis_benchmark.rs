//! Benchmarks for the analysis pipeline.
//!
//! Run with: `cargo bench`

use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use pizza_analysis_core::*;
use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::StandardTokenizer;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use pizza_engine::analysis::Tokenizer;

const SHORT_TEXT: &str = "The quick brown fox jumps over the lazy dog";
const MEDIUM_TEXT: &str = "Elasticsearch is a distributed, RESTful search and analytics engine \
    capable of addressing a growing number of use cases. As the heart of the Elastic Stack, \
    it centrally stores your data for lightning fast search, fine-tuned relevancy, and powerful \
    analytics that scale with ease.";
const LONG_TEXT: &str = "In computer science, a search engine is an information retrieval \
    system designed to help find information stored on a computer system. The search results \
    are usually presented in a list and are commonly called hits. Search engines help to \
    minimize the time required to find information and the amount of information which must \
    be consulted, akin to other techniques for managing information overload. The most public, \
    visible form of a search engine is a Web search engine which searches for information on \
    the World Wide Web. Search engines typically employ techniques of text mining and \
    information retrieval to process queries and return relevant documents. A search engine \
    maintains real-time information by running an algorithm on a web crawler. Content that \
    cannot be indexed by the search engine is considered part of the deep web.";

fn bench_standard_tokenizer(c: &mut Criterion) {
    let tokenizer = StandardTokenizer::new();

    let mut group = c.benchmark_group("standard_tokenizer");
    group.bench_with_input(
        BenchmarkId::new("short", SHORT_TEXT.len()),
        SHORT_TEXT,
        |b, text| b.iter(|| tokenizer.tokenize(black_box(text))),
    );
    group.bench_with_input(
        BenchmarkId::new("medium", MEDIUM_TEXT.len()),
        MEDIUM_TEXT,
        |b, text| b.iter(|| tokenizer.tokenize(black_box(text))),
    );
    group.bench_with_input(
        BenchmarkId::new("long", LONG_TEXT.len()),
        LONG_TEXT,
        |b, text| b.iter(|| tokenizer.tokenize(black_box(text))),
    );
    group.finish();
}

fn bench_lowercase_filter(c: &mut Criterion) {
    let tokenizer = StandardTokenizer::new();
    let filter = LowercaseTokenFilter::new();
    let tokens = tokenizer.tokenize(MEDIUM_TEXT);

    c.bench_function("lowercase_filter/medium", |b| {
        b.iter(|| {
            let mut toks = tokens.clone();
            for token in &mut toks {
                black_box(filter.filter(token));
            }
            toks
        })
    });
}

fn bench_stop_filter(c: &mut Criterion) {
    let tokenizer = StandardTokenizer::new();
    let filter = StopTokenFilter::english();
    let tokens = tokenizer.tokenize(MEDIUM_TEXT);

    c.bench_function("stop_filter_english/medium", |b| {
        b.iter(|| {
            let mut toks = tokens.clone();
            let mut result = Vec::with_capacity(toks.len());
            for mut token in toks.drain(..) {
                let (deleted, _) = filter.filter(&mut token);
                if !deleted {
                    result.push(token);
                }
            }
            result
        })
    });
}

fn bench_asciifolding(c: &mut Criterion) {
    let filter = AsciiFoldingTokenFilter::new();
    let text_with_diacritics = "Ménü résümé naïve café exposé fiancée";
    let tokenizer = StandardTokenizer::new();
    let tokens = tokenizer.tokenize(text_with_diacritics);

    c.bench_function("asciifolding/diacritics", |b| {
        b.iter(|| {
            let mut toks = tokens.clone();
            for token in &mut toks {
                black_box(filter.filter(token));
            }
            toks
        })
    });
}

fn bench_english_analyzer(c: &mut Criterion) {
    let mut factory = AnalysisFactory::new();
    register_all(&mut factory);
    let analyzer = factory.get_analyzer("english").unwrap();

    let mut group = c.benchmark_group("english_analyzer");
    group.bench_with_input(
        BenchmarkId::new("short", SHORT_TEXT.len()),
        SHORT_TEXT,
        |b, text| {
            b.iter(|| {
                let mut input = String::from(black_box(text));
                let tokens = analyzer.analyze_and_return_tokens(&mut input);
                black_box(tokens.len())
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("medium", MEDIUM_TEXT.len()),
        MEDIUM_TEXT,
        |b, text| {
            b.iter(|| {
                let mut input = String::from(black_box(text));
                let tokens = analyzer.analyze_and_return_tokens(&mut input);
                black_box(tokens.len())
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("long", LONG_TEXT.len()),
        LONG_TEXT,
        |b, text| {
            b.iter(|| {
                let mut input = String::from(black_box(text));
                let tokens = analyzer.analyze_and_return_tokens(&mut input);
                black_box(tokens.len())
            })
        },
    );
    group.finish();
}

fn bench_token_stream(c: &mut Criterion) {
    let tokenizer = StandardTokenizer::new();
    let lowercase = LowercaseTokenFilter::new();
    let stop = StopTokenFilter::english();
    let tokens = tokenizer.tokenize(LONG_TEXT);

    c.bench_function("token_stream/filter_chain_long", |b| {
        b.iter(|| {
            let toks = tokens.clone();
            let stream = TokenStream::from_tokens(toks)
                .filter_with(&lowercase)
                .filter_with(&stop);
            black_box(stream.into_terms())
        })
    });
}

fn bench_registry(c: &mut Criterion) {
    let registry = AnalysisRegistry::new();

    c.bench_function("registry/analyze_english_medium", |b| {
        b.iter(|| registry.analyze("english", black_box(MEDIUM_TEXT)))
    });
}

fn bench_factory_creation(c: &mut Criterion) {
    c.bench_function("factory/register_all", |b| {
        b.iter(|| {
            let mut factory = AnalysisFactory::new();
            register_all(&mut factory);
            black_box(&factory);
        })
    });
}

fn bench_multiple_languages(c: &mut Criterion) {
    let registry = AnalysisRegistry::new();
    let languages = ["english", "french", "german", "spanish", "italian"];

    let mut group = c.benchmark_group("multilingual");
    for lang in &languages {
        group.bench_with_input(BenchmarkId::new("analyze", lang), MEDIUM_TEXT, |b, text| {
            b.iter(|| registry.analyze(lang, black_box(text)))
        });
    }
    group.finish();
}

use std::string::String;

criterion_group!(
    benches,
    bench_standard_tokenizer,
    bench_lowercase_filter,
    bench_stop_filter,
    bench_asciifolding,
    bench_english_analyzer,
    bench_token_stream,
    bench_registry,
    bench_factory_creation,
    bench_multiple_languages,
);
criterion_main!(benches);
