---
title: CLI
description: The current scaffold and proposed command-line contract for Ink.
---

:::note[Implementation status]
The command names and flags below are present in the Rust CLI scaffold. Completion generation works, but the two prompt commands are placeholders and return a failure.
:::

## Commands

| Command                  | Intended purpose             | Current state   |
| ------------------------ | ---------------------------- | --------------- |
| `ink input`              | Edit one logical line        | Not implemented |
| `ink textarea`           | Edit multiline text          | Not implemented |
| `ink completion <shell>` | Generate a completion script | Scaffolded      |

## Prompt options

The same options apply to `input` and `textarea`.

| Option           | Intended behavior                                          |
| ---------------- | ---------------------------------------------------------- |
| `--value <TEXT>` | Seed the editable value; takes precedence over piped input |
| `--normal`       | Start in Normal mode instead of Insert mode                |
| `--theme <NAME>` | Select a bundled or configured theme                       |

## Completion targets

The CLI accepts `bash`, `zsh`, `fish`, and `nu`:

```sh
ink completion zsh
```

Completion scripts are generated from the same static CLI definition used for parsing. Generation does not open a prompt or access the controlling terminal.

## Proposed output contract

Accepted values will be emitted to standard output with exactly one final newline. Interface rendering and terminal control sequences will use the controlling terminal instead.

| Outcome         | Exit status | Standard output            |
| --------------- | ----------: | -------------------------- |
| Accepted        |         `0` | Accepted value and newline |
| Runtime failure |         `1` | Empty                      |
| Usage error     |         `2` | Empty                      |
| Cancelled       |       `130` | Empty                      |

These prompt outcomes describe the target behavior and are not yet implemented.
