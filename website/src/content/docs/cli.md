---
title: CLI
description: Commands, options, output, and exit statuses for Ink.
---

## Commands

| Command                  | Purpose                      |
| ------------------------ | ---------------------------- |
| `ink input`              | Edit one logical line        |
| `ink textarea`           | Edit multiline text          |
| `ink completion <shell>` | Generate a completion script |

## Prompt options

The same options apply to `input` and `textarea`.

| Option           | Behavior                                                   |
| ---------------- | ---------------------------------------------------------- |
| `--value <TEXT>` | Seed the editable value; takes precedence over piped input |
| `--normal`       | Start in Normal mode instead of Insert mode                |
| `--theme <NAME>` | Select a bundled or configured theme                       |

`input` also accepts `--prompt <TEXT>`. Its default is `> `; pass an empty string to remove it. `textarea` uses five editable rows plus a status row by default and accepts `--fullscreen` to use the complete terminal.

## Completion targets

The CLI accepts `bash`, `zsh`, `fish`, and `nu`:

```sh
ink completion zsh
```

Completion scripts are generated from the same static CLI definition used for parsing. Generation does not open a prompt or access the controlling terminal.

## Output contract

Accepted values will be emitted to standard output with exactly one final newline. Interface rendering and terminal control sequences will use the controlling terminal instead.

| Outcome         | Exit status | Standard output            |
| --------------- | ----------: | -------------------------- |
| Accepted        |         `0` | Accepted value and newline |
| Runtime failure |         `1` | Empty                      |
| Usage error     |         `2` | Empty                      |
| Cancelled       |       `130` | Empty                      |

Enter accepts `input`; Ctrl-D accepts either prompt. Ctrl-C cancels in every mode, and `q` cancels in Normal mode.
