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

Ink will start in Insert mode by default. The accepted text will be printed to standard output with one trailing newline. Pasted line breaks will be removed from single-line input.

The input prompt defaults to `> `. Customize or remove it with `--prompt "Name: "` or `--prompt ""`.

## Edit multiple lines

`ink textarea` edits prose and other multiline values. Press Ctrl-D to accept:

```sh
notes=$(ink textarea --value "Draft release notes")
gh release create v1.0.0 --notes "$notes"
```

Textarea shows five editable rows in the command workflow by default. Use `ink textarea --fullscreen` for the complete terminal area.

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
