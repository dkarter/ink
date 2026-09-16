---
title: Quick start
description: Preview the proposed Ink workflow without implying it is available today.
---

:::danger[Design preview only]
The commands on this page illustrate the intended interface. Prompt editing is not implemented in the current bootstrap.
:::

## Edit one line

`ink input` is planned for a single logical line:

```sh
title=$(ink input --value "Draft release title")
printf '%s\n' "$title"
```

Ink will start in Insert mode by default. The accepted text will be printed to standard output with one trailing newline. Pasted line breaks will be removed from single-line input.

## Edit multiple lines

`ink textarea` is planned for prose and other multiline values:

```sh
notes=$(ink textarea --value "Draft release notes")
gh release create v1.0.0 --notes "$notes"
```

## Seed from a pipe

When `--value` is absent, piped standard input is intended to seed the editor:

```sh
git log -5 --oneline | ink textarea
```

Ink will use the controlling terminal for interaction so standard output remains clean. If a pipe and `--value` are both present, `--value` wins.

## Start in Normal mode

The proposed `--normal` flag changes the initial mode without modifying the seed:

```sh
ink textarea --normal --value "Review this text"
```

Continue with [Vim modes](../vim-modes/) or review the complete [CLI design](../cli/).
