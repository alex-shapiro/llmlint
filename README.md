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

| Option                            |                                                                                                 |
| --------------------------------- | ----------------------------------------------------------------------------------------------- |
| `--output-format <concise\|full>` | `concise` (default) prints one line per finding; `full` displays a block with source underlined |
| `--select <ids>`                  | Comma-separated rule IDs to run                                                                 |
| `--ignore <ids>`                  | Comma-separated rule IDs to skip                                                                |
| `--no-ignore-vcs`                 | Lint files ignored by git                                                                       |
| `--list-rules`                    | List all rules and exit                                                                         |
| `-q`, `--quiet`                   | Suppress the summary line                                                                       |
| `-V`, `--version`                 | Print version and exit                                                                          |

The exit code is `0` if no clichés were found, `1` if clichés were found, and `2` on a usage or I/O error.

## Rules

Run `--list-rules` to display all rules. Some are adapted from Wikipedia's [Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) and are tagged accordingly; the rest were created by Simon Willison.

All rules are enabled by default. Upstream notes that `colon-triple` can produce false positives in technical writing. To avoid, use `--ignore colon-triple`.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
