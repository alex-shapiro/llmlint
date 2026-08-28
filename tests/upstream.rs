//! The upstream self-test suite, ported.
//!
//! `llm-cliche-highlighter.html` carries 192 self-tests, runnable with the
//! `node -e ...` one-liner printed on the page. The 182 pattern cases below are
//! generated from its `patternCases` table, and the example-text test is its
//! "example text trips every pattern exactly once".
//!
//! The tests that did not come across cover the GUI: sentence bounds, excerpt
//! windows, tooltip text and the URL-fragment loader, none of which a linter has.

use llmlint::lint::{all_rules, lint};
use llmlint::rules::RULES;

include!("generated/cases.rs");

fn rule(id: &str) -> &'static llmlint::rules::Rule {
    RULES
        .iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("no rule `{id}`"))
}

#[test]
fn every_rule_compiles() {
    for r in RULES {
        let _ = r.finder.compile();
    }
}

#[test]
fn rule_ids_are_unique() {
    let mut ids: Vec<&str> = RULES.iter().map(|r| r.id).collect();
    ids.sort_unstable();
    let count = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), count, "duplicate rule id");
}

/// Upstream calls `patternsById[id].find(sample)` directly, so these exercise a
/// single detector with no cross-rule overlap resolution.
#[test]
fn upstream_pattern_cases() {
    let mut failures = Vec::new();
    for (id, sample, expected) in PATTERN_CASES {
        let found = rule(id).finder.compile().find(sample);
        if found.len() != *expected {
            let spans: Vec<&str> = found.iter().map(|s| &sample[s.start..s.end]).collect();
            failures.push(format!(
                "{id} · {sample:?}\n      expected {expected} match(es), got {}: {spans:?}",
                found.len()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} case(s) failed:\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// Upstream: "example text trips every pattern exactly once".
#[test]
fn example_text_trips_every_pattern_once() {
    let text = include_str!("fixtures/example.txt");
    let findings = lint(text, &all_rules());

    assert_eq!(findings.len(), RULES.len(), "one match per rule");

    let mut ids: Vec<&str> = findings.iter().map(|f| f.rule.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), RULES.len(), "every rule distinct");
}

/// Overlapping matches collapse to the first-declared, longest-reaching rule,
/// and positions are reported 1-based.
#[test]
fn positions_are_one_based() {
    let text = "Intro line.\nNo fluff, no filler.\n";
    let findings = lint(text, &all_rules());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule.id, "no-chain");
    assert_eq!((findings[0].line, findings[0].column), (2, 1));
    assert_eq!(
        &text[findings[0].start..findings[0].end],
        "No fluff, no filler"
    );
}

/// Columns count characters, not bytes, so a match after an em dash lands where
/// a reader would point.
#[test]
fn columns_count_characters() {
    let text = "A — B. That's the whole point.";
    let findings = lint(text, &all_rules());
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0].column,
        text.chars().position(|c| c == 'T').unwrap() + 1
    );
}

/// Guards against the generated table silently emptying out.
#[test]
fn suite_is_fully_populated() {
    assert_eq!(PATTERN_CASES.len(), 182, "upstream pattern cases");
    assert_eq!(RULES.len(), 38, "upstream rules");
}
