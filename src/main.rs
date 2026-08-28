//! `llmlint` — a command line detector for LLM cliches.

use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use argh::FromArgs;
use ignore::WalkBuilder;

use llmlint::lint;
use llmlint::report::{self, OutputFormat, parse_format};
use llmlint::rules::RULES;

/// Detect LLM cliches in text files.
///
/// Paths may be files or directories; directories are searched recursively,
/// skipping binary files and anything git ignores.
#[derive(FromArgs)]
struct Args {
    /// files or directories to lint
    #[argh(positional)]
    paths: Vec<PathBuf>,

    /// output format: `concise` (default) or `full`
    #[argh(
        option,
        default = "OutputFormat::Concise",
        from_str_fn(parse_format),
        long = "output-format"
    )]
    output_format: OutputFormat,

    /// comma-separated rule ids to run, replacing the default set of all rules
    #[argh(option)]
    select: Option<String>,

    /// comma-separated rule ids to skip
    #[argh(option)]
    ignore: Option<String>,

    /// lint files that git ignores
    #[argh(switch, long = "no-ignore-vcs")]
    no_ignore_vcs: bool,

    /// list every rule and exit
    #[argh(switch, long = "list-rules")]
    list_rules: bool,

    /// suppress the summary line
    #[argh(switch, short = 'q')]
    quiet: bool,

    /// print version and exit
    #[argh(switch, short = 'V')]
    version: bool,
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    let strs: Vec<&str> = argv.iter().map(String::as_str).collect();
    let args = match Args::from_args(&strs[..1], &strs[1..]) {
        Ok(args) => args,
        Err(early) => {
            return match early.status {
                Ok(()) => {
                    print!("{}", early.output);
                    ExitCode::SUCCESS
                }
                Err(()) => {
                    eprint!("{}", early.output);
                    ExitCode::from(2)
                }
            };
        }
    };
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("llmlint: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> io::Result<ExitCode> {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if args.version {
        writeln!(out, "llmlint {}", env!("CARGO_PKG_VERSION"))?;
        return out.flush().map(|()| ExitCode::SUCCESS);
    }
    if args.list_rules {
        list_rules(&mut out)?;
        return out.flush().map(|()| ExitCode::SUCCESS);
    }

    let selected = match select_rules(args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("llmlint: {e}");
            return Ok(ExitCode::from(2));
        }
    };

    if args.paths.is_empty() {
        eprintln!("llmlint: no paths given (try `llmlint --help`)");
        return Ok(ExitCode::from(2));
    }

    let mut total = 0usize;
    let mut files = 0usize;
    let mut errors = 0usize;
    for path in &args.paths {
        if path.is_dir() {
            for file in walk(path, args.no_ignore_vcs) {
                match read_text(&file) {
                    Ok(None) => {}
                    Ok(Some(text)) => {
                        files += 1;
                        total += check(&mut out, &file, &text, &selected, args.output_format)?;
                    }
                    Err(e) => {
                        eprintln!("llmlint: {}: {e}", file.display());
                        errors += 1;
                    }
                }
            }
        } else {
            match read_text(path) {
                Ok(Some(text)) => {
                    files += 1;
                    total += check(&mut out, path, &text, &selected, args.output_format)?;
                }
                Ok(None) => eprintln!("llmlint: skipping {} (not UTF-8 text)", path.display()),
                Err(e) => {
                    eprintln!("llmlint: {}: {e}", path.display());
                    errors += 1;
                }
            }
        }
    }

    if !args.quiet {
        if total == 0 {
            writeln!(out, "All clear: no cliches in {}.", plural(files, "file"))?;
        } else {
            writeln!(
                out,
                "Found {} in {}.",
                plural(total, "cliche"),
                plural(files, "file")
            )?;
        }
    }

    out.flush()?;

    Ok(match (errors, total) {
        (0, 0) => ExitCode::SUCCESS,
        (0, _) => ExitCode::FAILURE,
        _ => ExitCode::from(2),
    })
}

/// Lints one document and prints its findings, returning how many there were.
fn check(
    out: &mut impl Write,
    path: &Path,
    text: &str,
    selected: &[bool],
    format: OutputFormat,
) -> io::Result<usize> {
    let findings = lint::lint(text, selected);
    report::report(out, path, text, &findings, format)?;
    Ok(findings.len())
}

/// Resolves `--select` and `--ignore` into a per-rule enabled flag.
fn select_rules(args: &Args) -> Result<Vec<bool>, String> {
    let known = |id: &str| RULES.iter().any(|r| r.id == id);
    let split = |list: &str| -> Result<Vec<String>, String> {
        let ids: Vec<String> = list
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        match ids.iter().find(|id| !known(id)) {
            Some(bad) => Err(format!("unknown rule `{bad}` (try `llmlint --list-rules`)")),
            None => Ok(ids),
        }
    };

    let mut selected = match &args.select {
        Some(list) => {
            let ids = split(list)?;
            RULES
                .iter()
                .map(|r| ids.iter().any(|id| id == r.id))
                .collect()
        }
        None => vec![true; RULES.len()],
    };
    if let Some(list) = &args.ignore {
        for id in split(list)? {
            if let Some(idx) = RULES.iter().position(|r| r.id == id) {
                selected[idx] = false;
            }
        }
    }
    Ok(selected)
}

fn list_rules(out: &mut impl Write) -> io::Result<()> {
    let width = RULES.iter().map(|r| r.id.len()).max().unwrap_or(0);
    for rule in RULES {
        let group = rule.group.map_or(String::new(), |g| format!("  [{g}]"));
        writeln!(out, "{:width$}  {}{group}", rule.id, rule.name)?;
    }
    Ok(())
}

/// Files under `root`, in a stable order, honouring ignore files.
fn walk(root: &Path, no_ignore_vcs: bool) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .git_ignore(!no_ignore_vcs)
        .git_global(!no_ignore_vcs)
        .git_exclude(!no_ignore_vcs)
        .sort_by_file_path(Path::cmp)
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .map(|e| e.into_path())
        .collect()
}

/// Reads a file as text, returning `None` if it is not UTF-8 prose.
fn read_text(path: &Path) -> io::Result<Option<String>> {
    let bytes = std::fs::read(path)?;
    if bytes.iter().take(8192).any(|&b| b == 0) {
        return Ok(None);
    }
    Ok(String::from_utf8(bytes).ok())
}

fn plural(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}
