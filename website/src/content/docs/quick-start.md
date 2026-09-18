---
title: Quick start
description: Start using Ink for single-line and multiline terminal input.
---

## Edit one line

`ink input` edits a single logical line:

```sh
title=$(ink input --value "Draft release title")
printf '%s\n' "$title"
```

Ink will start in Insert mode by default. The accepted text will be printed to standard output with one trailing newline. LF, CRLF, and lone CR line breaks are removed from single-line seeds and pasted text.

The input has no prefix by default. Add one with `--prompt "Name: "`.

Add empty-state guidance with `--placeholder "Release title"`. It disappears when you type and is never included in the accepted value.

Input uses three compact rows by default: input, blank padding, and status. Use `ink input --fullscreen` in a popup or other dedicated terminal area to keep the input at the top and the status at the bottom.

## Edit multiple lines

`ink textarea` edits prose and other multiline values. Press Ctrl-D to accept:

```sh
notes=$(ink textarea --value "Draft release notes")
gh release create v1.0.0 --notes "$notes"
```

Textarea shows five editable rows in the command workflow by default. Use `ink textarea --fullscreen` for the complete terminal area.

Textarea normalizes CRLF and lone CR line endings in seeds and pasted text to LF.

Textarea placeholders can span multiple lines, for example `ink textarea --placeholder $'Summary\nDetails'`.

## Seed from a pipe

When `--value` is absent, piped standard input seeds the editor:

```sh
git log -5 --oneline | ink textarea
```

Ink will use the controlling terminal for interaction so standard output remains clean. If a pipe and `--value` are both present, `--value` wins.

## Start in Normal mode

The `--normal` flag changes the initial mode without modifying the seed:

```sh
ink textarea --normal --value "Review this text"
```

Continue with [Vim modes](../vim-modes/) or review the complete [CLI reference](../cli/).
