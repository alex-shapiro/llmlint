# LLM Lint

Command line tool for detecting LLM cliches. Ported directly from [Simon Willison's web tool](https://github.com/simonw/tools/blob/main/llm-cliche-highlighter.html).

```
$ llmlint myfile.md

myfile.md:1 nox_noy No sign-ups, no downloads, no hassle
myfile.md:2 notx_noty Did not flinch, did not blink, did not reach for the red pen
myfile.md:4 dontverb_verb Don't call it a rewrite — call it
myfile.md:4 isreal_and is real, and
...
```
