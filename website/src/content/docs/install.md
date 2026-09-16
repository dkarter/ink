---
title: Install
description: Current installation status and source build instructions for Ink.
---

:::caution[Not released]
Ink does not have a supported package installation method yet. Build the current implementation from source while the project prepares its first release.
:::

## Build from source

Contributors can build the current command-line scaffold from source with a recent stable Rust toolchain:

```sh
git clone https://github.com/dkarter/ink.git
cd ink
cargo build
```

This produces a development binary at `target/debug/ink` with interactive input, textarea, help, and shell completion commands.

## Planned distribution

No release channel has been committed to yet. Installation commands will be documented here only after a usable version is published. Track [GitHub releases](https://github.com/dkarter/ink/releases) for that milestone.

## Requirements for future prompt use

The interactive commands require a controlling terminal. Piped text may provide an initial value, but Ink still needs a terminal for editing. A non-interactive environment without a controlling terminal fails cleanly without writing to standard output.
