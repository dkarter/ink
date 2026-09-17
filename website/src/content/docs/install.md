---
title: Install
description: Install a prebuilt Ink release or build Ink from source.
---

## Prebuilt release

Install the latest GitHub release with mise:

```sh
mise use --global github:dkarter/ink
```

Mise selects the archive for the current supported platform. GitHub Releases provide Linux x86_64, Linux arm64, macOS arm64, and Windows x86_64 builds.

## Build from source

Contributors can build the current command-line scaffold from source with a recent stable Rust toolchain:

```sh
git clone https://github.com/dkarter/ink.git
cd ink
cargo build
```

This produces a development binary at `target/debug/ink` with interactive input, textarea, help, and shell completion commands.

## Requirements for future prompt use

The interactive commands require a controlling terminal. Piped text may provide an initial value, but Ink still needs a terminal for editing. A non-interactive environment without a controlling terminal fails cleanly without writing to standard output.
