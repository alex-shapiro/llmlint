//! Detection of LLM cliches in prose.
//!
//! A Rust port of Simon Willison's `llm-cliche-highlighter.html`. The rule
//! table in [`rules`] is generated from that file; [`finders`] ports its
//! detectors and [`lint`] its overlap resolution.

pub mod finders;
pub mod lint;
pub mod report;
pub mod rules;
