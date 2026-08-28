//! Rendering findings for the terminal.

use std::io::{self, Write};
use std::path::Path;

use crate::lint::{Finding, LineIndex};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// One line per finding
    Concise,
    /// A block with the source line and the span underlined.
    Full,
}

/// Parses an `--output-format` argument.
pub fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "concise" => Ok(OutputFormat::Concise),
        "full" => Ok(OutputFormat::Full),
        other => Err(format!(
            "unknown output format `{other}` (expected concise or full)"
        )),
    }
}

/// Writes `findings` against `path` to `out` in the requested format.
pub fn report(
    out: &mut impl Write,
    path: &Path,
    text: &str,
    findings: &[Finding],
    format: OutputFormat,
) -> io::Result<()> {
    let path = path.display();
    match format {
        OutputFormat::Concise => {
            for f in findings {
                writeln!(
                    out,
                    "{path}:{}:{}: {} {}",
                    f.line, f.column, f.rule.id, f.rule.name
                )?;
            }
        }
        OutputFormat::Full => {
            let lines = LineIndex::new(text);
            for f in findings {
                let source = lines.line_text(text, f.line);
                let gutter = " ".repeat(f.line.to_string().len());

                let span = &text[f.start..f.end];
                let width = span
                    .split('\n')
                    .next()
                    .unwrap_or(span)
                    .chars()
                    .count()
                    .max(1);

                writeln!(out, "warning: {}", f.rule.name)?;
                writeln!(out, "{gutter}--> {path}:{}:{}", f.line, f.column)?;
                writeln!(out, "{gutter} |")?;
                writeln!(out, "{} | {source}", f.line)?;
                writeln!(
                    out,
                    "{gutter} | {}{}",
                    " ".repeat(f.column - 1),
                    "^".repeat(width)
                )?;
                writeln!(out, "{gutter} |")?;
                writeln!(out, "{gutter} = note: `{}`", f.rule.id)?;
                writeln!(out)?;
            }
        }
    }
    Ok(())
}
