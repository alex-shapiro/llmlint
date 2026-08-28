//! Cliche detectors and the spans they find.
//!
//! Each [`Rule`](crate::rules::Rule) carries a [`Finder`] describing how its
//! cliche is detected. [`Finder::compile`] turns that into a [`Detector`],
//! which reports matches as byte-offset [`Span`]s.

use std::collections::HashSet;
use std::sync::LazyLock;

use fancy_regex::Regex;

/// A single detected span, as byte offsets into the text it was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// How a rule locates its cliche: the static description stored in the rule
/// table, before any regex has been compiled.
pub enum Finder {
    /// A plain regex; every match is a hit.
    Regex(&'static str),
    /// A "HEAD x, HEAD y, ..." list, where the head is this regex fragment.
    Chain(&'static str),
    /// Runs of adjacent sentences sharing an n-gram skeleton.
    Echo { min_gram: usize, min_run: usize },
    /// Runs of consecutive question sentences.
    QuestionChain { min_run: usize },
    /// Runs of consecutive sentences opening on the same word.
    Anaphora { min_run: usize },
}

/// A finder with its regexes compiled.
pub enum Detector {
    Regex(Regex),
    Chain(Regex),
    Echo { min_gram: usize, min_run: usize },
    QuestionChain { min_run: usize },
    Anaphora { min_run: usize },
}

impl Finder {
    /// Compiles this finder's regexes into a runnable [`Detector`].
    ///
    /// # Panics
    ///
    /// If a pattern is malformed. Patterns are generated and compiled into the
    /// binary, so this is a build-time bug rather than a runtime condition.
    pub fn compile(&self) -> Detector {
        let build = |source: &str| {
            Regex::new(source).unwrap_or_else(|e| panic!("invalid rule regex {source:?}: {e}"))
        };
        match *self {
            Finder::Regex(source) => Detector::Regex(build(source)),
            Finder::Chain(head) => {
                let item = format!("{head}{CHAIN_BODY}");
                Detector::Chain(build(&format!(r"(?i)\b{item}(?:{CHAIN_SEP}{item})+")))
            }
            Finder::Echo { min_gram, min_run } => Detector::Echo { min_gram, min_run },
            Finder::QuestionChain { min_run } => Detector::QuestionChain { min_run },
            Finder::Anaphora { min_run } => Detector::Anaphora { min_run },
        }
    }
}

impl Detector {
    /// Every span of `text` this detector matches, in document order.
    pub fn find(&self, text: &str) -> Vec<Span> {
        match self {
            Detector::Regex(re) => find_regex(re, text),
            Detector::Chain(re) => find_chain(re, text),
            Detector::Echo { min_gram, min_run } => find_echo(text, *min_gram, *min_run),
            Detector::QuestionChain { min_run } => find_question_chain(text, *min_run),
            Detector::Anaphora { min_run } => find_anaphora(text, *min_run),
        }
    }
}

fn find_regex(re: &Regex, text: &str) -> Vec<Span> {
    re.find_iter(text)
        .filter_map(Result::ok)
        .map(|m| Span {
            start: m.start(),
            end: m.end(),
        })
        .collect()
}

/// The offset `end` walked back over any trailing whitespace, floored at
/// `floor`.
fn trim_end_ws(text: &str, end: usize, floor: usize) -> usize {
    floor + text[floor..end].trim_end().len()
}

/// The mirror of [`trim_end_ws`], advancing `start` past leading whitespace.
fn trim_start_ws(text: &str, start: usize, ceil: usize) -> usize {
    ceil - text[start..ceil].trim_start().len()
}

fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}

// --------------------------------------------------------------------- chains

const CHAIN_BODY: &str = r"[^,.;:!?\n–—…]*";
const CHAIN_SEP: &str =
    r"(?:\s*,\s*(?:and\s+|or\s+)?|\s+(?:and|or)\s+|\s*[;&–—]\s*(?:and\s+|or\s+)?|\s+-{1,2}\s+)";

fn find_chain(re: &Regex, text: &str) -> Vec<Span> {
    re.find_iter(text)
        .filter_map(Result::ok)
        .map(|m| Span {
            start: m.start(),
            end: trim_end_ws(text, m.end(), m.start()),
        })
        .collect()
}

// ---------------------------------------------------------------------- echo

struct Sentence {
    start: usize,
    end: usize,
}

static ECHO_SENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^.!?\n]+[.!?]?").unwrap());
static GRAM_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[a-z0-9'’-]+").unwrap());

/// The set of `n`-word shingles in a sentence, lowercased.
fn grams(s: &str, n: usize) -> HashSet<String> {
    let lower = s.to_lowercase();
    let words: Vec<&str> = GRAM_WORD
        .find_iter(&lower)
        .filter_map(Result::ok)
        .map(|m| m.as_str())
        .collect();
    let mut out = HashSet::new();
    if words.len() >= n {
        for w in words.windows(n) {
            out.insert(w.join(" "));
        }
    }
    out
}

fn find_echo(text: &str, min_gram: usize, min_run: usize) -> Vec<Span> {
    let sents: Vec<Sentence> = ECHO_SENT
        .find_iter(text)
        .filter_map(Result::ok)
        .filter(|m| word_count(m.as_str()) >= 4)
        .map(|m| Sentence {
            start: m.start(),
            end: m.end(),
        })
        .collect();

    let mut found = Vec::new();
    let mut i = 0;
    while i < sents.len() {
        let mut j = i;
        let mut shared = false;
        while j + 1 < sents.len() {
            if sents[j + 1].start - sents[j].end > 3 {
                break;
            }
            let a = grams(&text[sents[j].start..sents[j].end], min_gram);
            let b = grams(&text[sents[j + 1].start..sents[j + 1].end], min_gram);
            if a.is_disjoint(&b) {
                break;
            }
            shared = true;
            j += 1;
        }
        let run = j - i + 1;
        if run >= min_run && shared {
            let end = trim_end_ws(text, sents[j].end, sents[i].start);
            found.push(Span {
                start: sents[i].start,
                end,
            });
            i = j + 1;
        } else {
            i += 1;
        }
    }
    found
}

static QUESTION_CHAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^.!?\n]+\?(?:\s+[^.!?\n]+\?)+").unwrap());

fn find_question_chain(text: &str, min_run: usize) -> Vec<Span> {
    QUESTION_CHAIN
        .find_iter(text)
        .filter_map(Result::ok)
        .filter(|m| m.as_str().matches('?').count() >= min_run)
        .map(|m| Span {
            start: trim_start_ws(text, m.start(), m.end()),
            end: m.end(),
        })
        .collect()
}

static ANAPHORA_SENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^.!?\n]+[.!?]").unwrap());
static ANAPHORA_HEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[A-Za-z'’-]+").unwrap());

/// Pronouns and articles whose repetition is ordinary prose.
static ANAPHORA_SKIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(?:i|it|the|a|an|this|that|we|you|they|he|she|there|but|and|so|in|as|if|my|his|her|their|its|these|those|for|at|on|of|to|is|was)$",
    )
    .unwrap()
});

struct Opener {
    start: usize,
    end: usize,
    head: String,
}

fn find_anaphora(text: &str, min_run: usize) -> Vec<Span> {
    let mut sents: Vec<Opener> = Vec::new();
    for m in ANAPHORA_SENT.find_iter(text).filter_map(Result::ok) {
        if let Ok(Some(w)) = ANAPHORA_HEAD.find(m.as_str()) {
            sents.push(Opener {
                start: m.start() + w.start(),
                end: m.end(),
                head: w.as_str().to_lowercase(),
            });
        }
    }

    let mut found = Vec::new();
    let mut i = 0;
    while i < sents.len() {
        let mut j = i;
        while j + 1 < sents.len()
            && sents[j + 1].head == sents[i].head
            && sents[j + 1].start - sents[j].end < 4
        {
            j += 1;
        }
        let run = j - i + 1;
        if run >= min_run && !ANAPHORA_SKIP.is_match(&sents[i].head).unwrap_or(false) {
            found.push(Span {
                start: sents[i].start,
                end: sents[j].end,
            });
            i = j + 1;
        } else {
            i += 1;
        }
    }
    found
}
