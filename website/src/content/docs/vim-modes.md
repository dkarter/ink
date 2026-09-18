---
title: Vim modes
description: The modal editing model for Ink prompts.
---

## Mode model

Both prompt types support three core modes:

| Mode   | Purpose                             | Cursor       |
| ------ | ----------------------------------- | ------------ |
| Insert | Enter text                          | Steady bar   |
| Normal | Navigate and issue editing commands | Steady block |
| Visual | Select an inclusive grapheme range  | Steady block |

`textarea` additionally specifies **Visual Line** for whole logical lines and **Visual Block** for rectangular selections across display columns.

The active mode appears as a textual label such as `INSERT` or `VISUAL BLOCK`. Color is supplemental, not the only mode cue.

## Selections and operators

Ink supports delete, change, and yank over active selections:

- Delete removes selected graphemes, writes them to the unnamed register, and returns to Normal mode.
- Change follows the same deletion rules and enters Insert mode at the start of the removed range.
- Yank preserves the buffer, copies the exact selection, and returns to Normal mode at the selection start.
- Block operators process each selected row independently without removing unselected line breaks.

## Unicode text

Movement and editing operate on extended grapheme clusters rather than bytes or Unicode scalar values. A base character with combining marks, or a joined emoji sequence, moves and edits as one unit.

## Viewports

For multiline values, the viewport follows the cursor horizontally and vertically without altering the underlying buffer. Layout is recomputed after terminal resize events, including very narrow and transient zero-sized areas.

Arrow keys move in Insert mode. Normal and Visual modes support `h`, `j`, `k`, `l`, `0`, `$`, `w`, `b`, `e`, `W`, `B`, and `E`; `i` inserts at the cursor, `A` appends at the end of the line, and `v`, `V`, and Ctrl-V enter Visual modes. The `d`, `c`, and `y` keys act on Visual selections. Lowercase word motions stop at keyword and punctuation boundaries, while uppercase WORD motions stop at whitespace boundaries. From Normal mode, `:` turns the status row into a command line; `:wq` accepts, `:q!` cancels, and `:q` or `:qa` asks for confirmation before submitting. Invalid commands briefly appear in the theme's error color. Ctrl-C cancels in every mode, `q` cancels from Normal mode, and Ctrl-D accepts.

In Normal mode, `d`, `c`, and `y` compose with word motions and `iw`/`aw`. Repeating the operator (`dd`, `cc`, or `yy`) applies it linewise. Visual Block `c` applies the replacement entered on the first selected row to every selected row when Insert mode ends. Use `p` or `P` to paste after/before the cursor; linewise textarea registers paste below/above the current line. Textarea also supports `o` and `O` to open a line below/above and enter Insert mode.
