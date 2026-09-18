---
title: CLI
description: Commands, options, output, and exit statuses for Ink.
---

## Commands

| Command                  | Purpose                      |
| ------------------------ | ---------------------------- |
| `ink input`              | Edit one logical line        |
| `ink textarea`           | Edit multiline text          |
| `ink theme`              | Preview and save a theme     |
| `ink config validate`    | Validate resolved config     |
| `ink completion <shell>` | Generate a completion script |

## Prompt options

The same options apply to `input` and `textarea`.

| Option                 | Behavior                                                   |
| ---------------------- | ---------------------------------------------------------- |
| `--value <TEXT>`       | Seed the editable value; takes precedence over piped input |
| `--normal`             | Start in Normal mode instead of Insert mode                |
| `--theme <NAME>`       | Select a bundled or configured theme                       |
| `--placeholder <TEXT>` | Show guidance while the editable value is empty            |
| `--fullscreen`         | Use the complete terminal and alternate screen             |

`input` also accepts `--prompt <TEXT>` and shows no prefix by default. Inline input uses one editable row, one blank padding row, and one status row. Textarea uses five editable rows plus a status row. In fullscreen layouts, the status remains at the bottom while editing starts at the top.

`textarea` shows line numbers by default. Use `--no-line-numbers` to hide them or `--line-numbers` to override a disabled global setting.

Input removes LF, CRLF, and lone CR line breaks from explicit values, piped seeds, and bracketed paste. Textarea uses LF for its logical line model, normalizing CRLF and lone CR from those sources to LF.

Placeholders are presentation-only and never become part of the editable or accepted value. Input removes line breaks from placeholder text, while textarea preserves its logical lines.

## Completion targets

The CLI accepts `bash`, `zsh`, `fish`, and `nu`:

```sh
ink completion zsh
```

Completion scripts are generated from the same static CLI definition used for parsing. Generation does not open a prompt or access the controlling terminal.

Generated completions include the `theme` command and nested `config validate` command.

## Output contract

Accepted values are emitted to standard output followed by one output-record newline. Interface rendering and terminal control sequences use the controlling terminal instead. Textarea logical line endings are emitted as LF.

| Outcome         | Exit status | Standard output            |
| --------------- | ----------: | -------------------------- |
| Accepted        |         `0` | Accepted value and newline |
| Runtime failure |         `1` | Empty                      |
| Usage error     |         `2` | Empty                      |
| Cancelled       |       `130` | Empty                      |

Enter accepts `input`; Ctrl-D accepts either prompt. Ctrl-C cancels in every mode, and `q` cancels in Normal mode.
