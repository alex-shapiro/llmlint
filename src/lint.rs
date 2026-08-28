//! Running the rule set over a document.

use std::sync::LazyLock;

use crate::finders::Detector;
use crate::rules::{RULES, Rule};

/// Compiled detectors for all rules, indexed correspondingly with [`RULES`].
static DETECTORS: LazyLock<Vec<Detector>> =
    LazyLock::new(|| RULES.iter().map(|r| r.finder.compile()).collect());

/// A single detected cliché finding.
pub struct Finding {
    pub rule: &'static Rule,
    /// 1-based line number of the start of the match.
    pub line: usize,
    /// 1-based character column number of the start of the match.
    pub column: usize,
    /// Starting byte offset of the match within the document.
    pub start: usize,
    /// Ending byte offset of the match within the document.
    pub end: usize,
}

/// Returns a boolean mask with all rules enabled.
pub fn all_rules() -> Vec<bool> {
    vec![true; RULES.len()]
}

/// Finds all clichés in `text` matching the rules enabled in `enabled`,
/// which is indexed in the same order as [`RULES`].
///
/// Findings are returned in document order without overlapping spans.
/// Overlap conflicts are resolved by prioritizing:
/// 1. Earlier start position
/// 2. Longer match length
/// 3. Rule declared earlier in [`RULES`]
///
/// # Panics
///
/// Panics if `enabled.len() < RULES.len()`.
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

    /// Returns the 1-based line and character column corresponding to byte offset `offset`.
    pub fn locate(&self, text: &str, offset: usize) -> (usize, usize) {
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let column = text[self.starts[line]..offset].chars().count() + 1;
        (line + 1, column)
    }

    /// Returns the text of the given 1-based `line`, without its trailing newline.
    pub fn line_text<'t>(&self, text: &'t str, line: usize) -> &'t str {
        let start = self.starts[line - 1];
        let end = self.starts.get(line).map_or(text.len(), |&s| s - 1);
        text[start..end].trim_end_matches('\r')
    }
}
