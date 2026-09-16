---
title: Install
description: Current installation status and source build instructions for Ink.
---

:::caution[Not released]
Ink does not have a supported release or package installation method yet. The prompt editor is not implemented. Do not add it to production scripts.
:::

## Inspect the bootstrap

Contributors can build the current command-line scaffold from source with a recent stable Rust toolchain:

```sh
git clone https://github.com/dkarter/ink.git
cd ink
cargo build
```

This produces a development binary at `target/debug/ink`. At present, the binary can show help and generate shell completions. Running `ink input` or `ink textarea` reports that prompts are not implemented and exits unsuccessfully.

## Planned distribution

No release channel has been committed to yet. Installation commands will be documented here only after a usable version is published. Track [GitHub releases](https://github.com/dkarter/ink/releases) for that milestone.

## Requirements for future prompt use

The planned interactive commands require a controlling terminal. Piped text may provide an initial value, but Ink will still need a terminal for editing. A non-interactive environment without a controlling terminal is specified to fail cleanly without writing to standard output.
