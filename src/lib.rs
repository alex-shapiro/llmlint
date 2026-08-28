//! Detection of LLM cliches in prose.
//!
//! [`lint::lint`] runs the rules in [`rules::RULES`] over a document and
//! returns non-overlapping findings, which [`report::report`] renders in
//! either of two formats.

pub mod finders;
pub mod lint;
pub mod report;
pub mod rules;
