# LLM Lint

A command-line tool for detecting LLM clichés. Ported from [Simon Willison's web tool](https://github.com/simonw/tools/blob/main/llm-cliche-highlighter.html).

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

Paths may be files or directories. Directories are searched recursively for valid UTF-8 text files, skipping binaries and paths ignored by git. A file passed explicitly is linted even if gitignored.

| Option                            |                                                                                                                      |
| --------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `--output-format <concise\|full>` | `concise` (default) prints one line per finding; `full` displays a rustc-style block with the source line underlined |
| `--select <ids>`                  | Comma-separated rule IDs to run, replacing the default set of all rules                                              |
| `--ignore <ids>`                  | Comma-separated rule IDs to skip                                                                                     |
| `--no-ignore-vcs`                 | Lint files ignored by git                                                                                            |
| `--list-rules`                    | List all rules and exit                                                                                              |
| `-q`, `--quiet`                   | Suppress the summary line                                                                                            |
| `-V`, `--version`                 | Print version and exit                                                                                               |

The exit code is `0` if no clichés were found, `1` if clichés were found, and `2` on a usage or I/O error — following the same convention used by clippy and ruff.

`--output-format full` displays detailed diagnostics:

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

There are 38 rules, named using the kebab-case IDs from upstream (the same naming convention used by tools like ruff). Run `--list-rules` to display all of them. Eleven are adapted from Wikipedia's [Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) and are tagged accordingly; the rest were created by Simon Willison.

All rules are enabled by default, including `colon-triple`, which flags a colon opening onto three or more comma-separated items. Upstream notes that this rule can produce false positives in technical writing; use `--ignore colon-triple` when linting documentation.
