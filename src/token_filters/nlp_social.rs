use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

// ═══════════════════════════════════════════════════════════════════════════════
// NLP & SOCIAL MEDIA FILTERS — Language intelligence, social content parsing
// ═══════════════════════════════════════════════════════════════════════════════

/// Expands English contractions: "don't" → "do not", "I'm" → "I am"
#[derive(Clone, Debug)]
pub struct ContractionExpandTokenFilter;
impl ContractionExpandTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ContractionExpandTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for ContractionExpandTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        let expanded = expand_contraction(&lower);
        if let Some(exp) = expanded {
            token.term = Cow::Owned(String::from(exp));
        }
        (false, None)
    }
}

/// Expands common abbreviations: "govt" → "government", "approx" → "approximately"
#[derive(Clone, Debug)]
pub struct AbbreviationExpandTokenFilter;
impl AbbreviationExpandTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for AbbreviationExpandTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for AbbreviationExpandTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        let expanded = expand_abbreviation(&lower);
        if let Some(exp) = expanded {
            // Emit expansion as synonym, keep original
            let synonym = Token {
                term: Cow::Owned(String::from(exp)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![synonym]));
        }
        (false, None)
    }
}

/// Splits #CamelCase hashtags: "#OpenSource" → ["opensource", "open", "source"]
#[derive(Clone, Debug)]
pub struct HashtagSplitTokenFilter {
    pub keep_original: bool,
}
impl HashtagSplitTokenFilter {
    pub fn new() -> Self {
        Self {
            keep_original: true,
        }
    }
}
impl Default for HashtagSplitTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for HashtagSplitTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let body = text.strip_prefix('#').unwrap_or(text);
        if body.is_empty() {
            return (false, None);
        }

        let parts = split_camel_case(body);
        if parts.len() <= 1 {
            return (false, None);
        }

        let mut extras: Vec<Token<'a>> = parts
            .iter()
            .map(|p| Token {
                term: Cow::Owned(p.to_lowercase()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            })
            .collect();

        if self.keep_original {
            // Also add the full hashtag body without #
            extras.push(Token {
                term: Cow::Owned(body.to_lowercase()),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            });
            (false, Some(extras))
        } else {
            token.term = Cow::Owned(extras[0].term.to_string());
            (false, Some(extras[1..].to_vec()))
        }
    }
}

/// Normalizes internet slang: "lol" → "laughing out loud", "brb" → "be right back"
#[derive(Clone, Debug)]
pub struct SlangNormTokenFilter;
impl SlangNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SlangNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SlangNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        if let Some(expanded) = expand_slang(&lower) {
            let synonym = Token {
                term: Cow::Owned(String::from(expanded)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![synonym]));
        }
        (false, None)
    }
}

/// Converts text to sentence case (first letter uppercase, rest lowercase).
#[derive(Clone, Debug)]
pub struct SentenceCaseTokenFilter;
impl SentenceCaseTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SentenceCaseTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SentenceCaseTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.is_empty() {
            return (false, None);
        }
        let mut chars = text.chars();
        let first = chars.next().unwrap().to_uppercase().to_string();
        let rest: String = chars.as_str().to_lowercase();
        token.term = Cow::Owned(format!("{}{}", first, rest));
        (false, None)
    }
}

/// Detects and tags @mentions: "@user123" → emits "_mention:user123"
#[derive(Clone, Debug)]
pub struct MentionTagTokenFilter;
impl MentionTagTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for MentionTagTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for MentionTagTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(user) = text.strip_prefix('@') {
            if !user.is_empty() {
                let tag = Token {
                    term: Cow::Owned(format!("_mention:{}", user)),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                // Strip the @ from main token
                token.term = Cow::Owned(String::from(user));
                return (false, Some(alloc::vec![tag]));
            }
        }
        (false, None)
    }
}

/// Detects and tags #hashtags: "#rust" → emits "_hashtag:rust"
#[derive(Clone, Debug)]
pub struct HashtagTagTokenFilter;
impl HashtagTagTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HashtagTagTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HashtagTagTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if let Some(tag_text) = text.strip_prefix('#') {
            if !tag_text.is_empty() {
                let tag = Token {
                    term: Cow::Owned(format!("_hashtag:{}", tag_text.to_lowercase())),
                    start_offset: token.start_offset,
                    end_offset: token.end_offset,
                    position: token.position,
                };
                token.term = Cow::Owned(tag_text.to_lowercase());
                return (false, Some(alloc::vec![tag]));
            }
        }
        (false, None)
    }
}

/// Repeats/emphasizes words: stretches like "sooooo" → "so", "goooood" → "good"
#[derive(Clone, Debug)]
pub struct StretchedWordNormTokenFilter {
    pub max_repeat: usize,
}
impl StretchedWordNormTokenFilter {
    pub fn new() -> Self {
        Self { max_repeat: 2 }
    }
}
impl Default for StretchedWordNormTokenFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for StretchedWordNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let collapsed = collapse_stretches(text, self.max_repeat);
        if collapsed != text {
            token.term = Cow::Owned(collapsed);
        }
        (false, None)
    }
}

/// Detects text sentiment markers (simple keyword-based).
#[derive(Clone, Debug)]
pub struct SentimentTagTokenFilter;
impl SentimentTagTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for SentimentTagTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for SentimentTagTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        let lower = text.to_lowercase();
        let sentiment = get_word_sentiment(&lower);
        if let Some(s) = sentiment {
            let tag = Token {
                term: Cow::Owned(format!("_sentiment:{}", s)),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

/// Strip @mentions entirely from the stream.
#[derive(Clone, Debug)]
pub struct MentionRemoveTokenFilter;
impl MentionRemoveTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for MentionRemoveTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for MentionRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with('@') {
            return (true, None);
        }
        (false, None)
    }
}

/// Strip #hashtags entirely from the stream.
#[derive(Clone, Debug)]
pub struct HashtagRemoveTokenFilter;
impl HashtagRemoveTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for HashtagRemoveTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for HashtagRemoveTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        if token.term.starts_with('#') {
            return (true, None);
        }
        (false, None)
    }
}

/// Detects if a token is ALL CAPS (shouting) and normalizes.
#[derive(Clone, Debug)]
pub struct ShoutingNormTokenFilter;
impl ShoutingNormTokenFilter {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ShoutingNormTokenFilter {
    fn default() -> Self {
        Self
    }
}

impl TokenFilter for ShoutingNormTokenFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let text = token.term.as_ref();
        if text.len() > 1 && text.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
            token.term = Cow::Owned(text.to_lowercase());
            let tag = Token {
                term: Cow::Owned(String::from("_emphasis:shouting")),
                start_offset: token.start_offset,
                end_offset: token.end_offset,
                position: token.position,
            };
            return (false, Some(alloc::vec![tag]));
        }
        (false, None)
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn expand_contraction(s: &str) -> Option<&'static str> {
    match s {
        "don't" | "dont" => Some("do not"),
        "doesn't" | "doesnt" => Some("does not"),
        "didn't" | "didnt" => Some("did not"),
        "won't" | "wont" => Some("will not"),
        "wouldn't" | "wouldnt" => Some("would not"),
        "shouldn't" | "shouldnt" => Some("should not"),
        "couldn't" | "couldnt" => Some("could not"),
        "can't" | "cant" => Some("cannot"),
        "isn't" | "isnt" => Some("is not"),
        "aren't" | "arent" => Some("are not"),
        "wasn't" | "wasnt" => Some("was not"),
        "weren't" | "werent" => Some("were not"),
        "hasn't" | "hasnt" => Some("has not"),
        "haven't" | "havent" => Some("have not"),
        "hadn't" | "hadnt" => Some("had not"),
        "i'm" | "im" => Some("i am"),
        "you're" | "youre" => Some("you are"),
        "we're" | "were" => None, // ambiguous
        "they're" | "theyre" => Some("they are"),
        "he's" | "hes" => Some("he is"),
        "she's" | "shes" => Some("she is"),
        "it's" | "its" => None, // ambiguous (possessive vs contraction)
        "i've" | "ive" => Some("i have"),
        "you've" | "youve" => Some("you have"),
        "we've" | "weve" => Some("we have"),
        "they've" | "theyve" => Some("they have"),
        "i'll" | "ill" => None, // ambiguous
        "you'll" | "youll" => Some("you will"),
        "he'll" | "hell" => None,   // ambiguous
        "she'll" | "shell" => None, // ambiguous
        "we'll" | "well" => None,   // ambiguous
        "they'll" | "theyll" => Some("they will"),
        "i'd" | "id" => None, // ambiguous
        "you'd" | "youd" => Some("you would"),
        "he'd" | "hed" => Some("he would"),
        "she'd" | "shed" => None, // ambiguous
        "we'd" | "wed" => None,   // ambiguous
        "they'd" | "theyd" => Some("they would"),
        "let's" | "lets" => None, // ambiguous
        "that's" | "thats" => Some("that is"),
        "who's" | "whos" => Some("who is"),
        "what's" | "whats" => Some("what is"),
        "there's" | "theres" => Some("there is"),
        "here's" | "heres" => Some("here is"),
        _ => None,
    }
}

fn expand_abbreviation(s: &str) -> Option<&'static str> {
    match s {
        "govt" => Some("government"),
        "approx" => Some("approximately"),
        "dept" => Some("department"),
        "mgmt" => Some("management"),
        "info" => Some("information"),
        "tech" => Some("technology"),
        "env" => Some("environment"),
        "config" => Some("configuration"),
        "auth" => Some("authentication"),
        "admin" => Some("administrator"),
        "dev" => Some("development"),
        "prod" => Some("production"),
        "impl" => Some("implementation"),
        "repo" => Some("repository"),
        "dir" => Some("directory"),
        "src" => Some("source"),
        "msg" => Some("message"),
        "err" => Some("error"),
        "req" => Some("request"),
        "res" => Some("response"),
        "btn" => Some("button"),
        "img" => Some("image"),
        "num" => Some("number"),
        "str" => Some("string"),
        "obj" => Some("object"),
        "arr" => Some("array"),
        "fn" => Some("function"),
        "var" => Some("variable"),
        "ctx" => Some("context"),
        "ref" => Some("reference"),
        "ptr" => Some("pointer"),
        "alloc" => Some("allocation"),
        "dealloc" => Some("deallocation"),
        "init" => Some("initialize"),
        "exec" => Some("execute"),
        "calc" => Some("calculate"),
        "util" => Some("utility"),
        "tmp" => Some("temporary"),
        "prev" => Some("previous"),
        "cur" | "curr" => Some("current"),
        _ => None,
    }
}

fn expand_slang(s: &str) -> Option<&'static str> {
    match s {
        "lol" => Some("laughing out loud"),
        "brb" => Some("be right back"),
        "tbh" => Some("to be honest"),
        "imo" => Some("in my opinion"),
        "imho" => Some("in my humble opinion"),
        "fwiw" => Some("for what it is worth"),
        "afaik" => Some("as far as i know"),
        "iirc" => Some("if i recall correctly"),
        "tl;dr" | "tldr" => Some("too long did not read"),
        "btw" => Some("by the way"),
        "smh" => Some("shaking my head"),
        "irl" => Some("in real life"),
        "fyi" => Some("for your information"),
        "asap" => Some("as soon as possible"),
        "eta" => Some("estimated time of arrival"),
        "diy" => Some("do it yourself"),
        "tbd" => Some("to be determined"),
        "wip" => Some("work in progress"),
        "afk" => Some("away from keyboard"),
        "idk" => Some("i do not know"),
        "nvm" => Some("never mind"),
        "omg" => Some("oh my god"),
        "rofl" => Some("rolling on floor laughing"),
        "ty" => Some("thank you"),
        "np" => Some("no problem"),
        "gg" => Some("good game"),
        "gl" => Some("good luck"),
        "pls" | "plz" => Some("please"),
        "thx" | "thnx" => Some("thanks"),
        _ => None,
    }
}

fn split_camel_case(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if c.is_uppercase() && !current.is_empty() {
            parts.push(current.clone());
            current.clear();
        }
        current.push(c);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn collapse_stretches(s: &str, max_repeat: usize) -> String {
    let mut result = String::new();
    let mut prev_char: Option<char> = None;
    let mut count = 0usize;
    for c in s.chars() {
        if Some(c) == prev_char {
            count += 1;
            if count <= max_repeat {
                result.push(c);
            }
        } else {
            result.push(c);
            prev_char = Some(c);
            count = 1;
        }
    }
    result
}

fn get_word_sentiment(s: &str) -> Option<&'static str> {
    match s {
        "good" | "great" | "excellent" | "amazing" | "wonderful" | "fantastic" | "awesome"
        | "love" | "happy" | "joy" | "beautiful" | "perfect" | "best" | "brilliant"
        | "outstanding" | "superb" | "incredible" | "delightful" | "pleasant" | "positive" => {
            Some("positive")
        }

        "bad" | "terrible" | "horrible" | "awful" | "worst" | "hate" | "ugly" | "disgusting"
        | "pathetic" | "dreadful" | "miserable" | "angry" | "sad" | "disappointing" | "poor"
        | "negative" | "toxic" | "broken" | "failed" | "useless" => Some("negative"),

        _ => None,
    }
}
