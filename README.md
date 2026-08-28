# LLM Lint

Command line tool for detecting LLM cliches. Ported directly from [Simon Willison's web tool](https://github.com/simonw/tools/blob/main/llm-cliche-highlighter.html).

```
$ llmlint myfile.md

myfile.md:1:43: no-chain “No X, no Y” chains
myfile.md:3:36: did-not-chain “Did not X, did not Y” chains
myfile.md:5:1: dont-verb-it “Don’t VERB it … VERB it”
myfile.md:5:61: is-real “Is real … and / not”
...
Found 38 cliches in 1 file.
```

## Usage

```
llmlint [OPTIONS] <paths...>
```

Paths may be files or directories. Directories are searched recursively for
anything that decodes as UTF-8, skipping binaries and paths git ignores. A file
named explicitly on the command line is always linted, even if git ignores it.

| Option | |
| --- | --- |
| `--output-format <concise\|full>` | `concise` (default) is one line per finding; `full` is a rustc-style block with the source line underlined |
| `--select <ids>` | Comma-separated rule ids to run, replacing the default set of all rules |
| `--ignore <ids>` | Comma-separated rule ids to skip |
| `--no-ignore-vcs` | Lint files that git ignores |
| `--list-rules` | List every rule and exit |
| `-q`, `--quiet` | Suppress the summary line |
| `-V`, `--version` | Print version and exit |

Exit status is `0` when nothing was found, `1` when cliches were found, and `2`
on a usage or I/O error — the same convention clippy and ruff use.

`--output-format full` gives the fuller diagnostic:

```
warning: “No X, no Y” chains
 --> myfile.md:1:43
  |
1 | We rebuilt the editor from the ground up. No sign-ups, no downloads, no hassle — just paste.
  |                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: `no-chain`
```

## Rules

38 rules, named with the ids upstream already uses — kebab-case, which is also
how ruff names its rules. `--list-rules` prints them all. Eleven are adapted
from Wikipedia's [Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
and are tagged as such; the rest are Simon's own.

Every rule is on by default, including `colon-triple`, which flags a colon
opening onto three or more comma-separated items. Upstream notes it is noisy on
technical writing — `--ignore colon-triple` when linting documentation.

## How the port works

`src/rules.rs`, `tests/generated/cases.rs` and `tests/fixtures/example.txt` are
generated from the upstream HTML, so the regexes are upstream's own rather than
retyped:

```
node tools/generate.js [path-or-url]   # defaults to fetching from simonw/tools
cargo fmt
```

Everything else is hand-written. `src/finders.rs` ports the `make*Finder`
detector factories and `src/lint.rs` ports `collectMatches`, including its
overlap resolution — where two rules match overlapping spans, the one declared
first wins, so the order of `RULES` is load-bearing.

Upstream ships 192 self-tests: 182 per-pattern cases plus 10 others. All 182
are ported in `tests/upstream.rs`, along with upstream's "example text trips
every pattern exactly once" integration test. The nine left behind cover the
GUI: sentence bounds, excerpt windows, tooltip text and the URL-fragment
loader.

Two deliberate deviations from upstream:

- **No match counts.** The web tool puts a badge on chain, echo, question and
  anaphora matches counting the items in the run. A linter has nowhere to show
  one, so the counts are dropped. Upstream's pattern cases assert those counts
  alongside the match count; the ported cases keep the match counts only.
- **Unicode `\w` and `\b`.** JavaScript regexes without the `u` flag treat these
  as ASCII. On ASCII prose the behaviour is identical, and on accented words the
  Rust reading is the better one — "café" is one word, not "caf" plus a stray
  letter.
