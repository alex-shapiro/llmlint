//! Running the rule set over a document.

use std::sync::LazyLock;

use crate::finders::Detector;
use crate::rules::{RULES, Rule};

/// Every rule's detector, compiled once, indexed in step with [`RULES`].
static DETECTORS: LazyLock<Vec<Detector>> =
    LazyLock::new(|| RULES.iter().map(|r| r.finder.compile()).collect());

/// One reported cliche.
pub struct Finding {
    pub rule: &'static Rule,
    /// 1-based line of the span's first character.
    pub line: usize,
    /// 1-based column, counted in characters.
    pub column: usize,
    /// Byte range of the match within the document.
    pub start: usize,
    pub end: usize,
}

/// Every rule enabled, for callers that want the default set.
pub fn all_rules() -> Vec<bool> {
    vec![true; RULES.len()]
}

/// Finds every cliche in `text`, restricted to the rules flagged in `enabled`,
/// which is indexed in step with [`RULES`].
///
/// Findings come back in document order and never overlap. Where two matches
/// would overlap, the one starting earlier wins; between matches starting
/// together the longer wins; and between identical spans the rule declared
/// first in [`RULES`] wins.
///
/// # Panics
///
/// If `enabled` is shorter than [`RULES`].
pub fn lint(text: &str, enabled: &[bool]) -> Vec<Finding> {
    let mut raw: Vec<(usize, usize, usize)> = Vec::new();
    for idx in 0..RULES.len() {
        if !enabled[idx] {
            continue;
        }
        for span in DETECTORS[idx].find(text) {
            raw.push((span.start, span.end, idx));
        }
    }
    raw.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));

    let lines = LineIndex::new(text);
    let mut findings: Vec<Finding> = Vec::new();
    for (start, end, idx) in raw {
        if findings.last().is_some_and(|last| start < last.end) {
            continue;
        }
        let (line, column) = lines.locate(text, start);
        findings.push(Finding {
            rule: &RULES[idx],
            line,
            column,
            start,
            end,
        });
    }
    findings
}

/// Maps byte offsets in a document to 1-based line and column positions.
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    /// Indexes the line starts of `text`.
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Self { starts }
    }

    /// The 1-based line and character column containing byte offset `offset`.
    pub fn locate(&self, text: &str, offset: usize) -> (usize, usize) {
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let column = text[self.starts[line]..offset].chars().count() + 1;
        (line + 1, column)
    }

    /// The text of the 1-based `line`, without its trailing newline.
    pub fn line_text<'t>(&self, text: &'t str, line: usize) -> &'t str {
        let start = self.starts[line - 1];
        let end = self.starts.get(line).map_or(text.len(), |&s| s - 1);
        text[start..end].trim_end_matches('\r')
    }
}
