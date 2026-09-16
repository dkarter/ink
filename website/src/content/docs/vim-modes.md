---
title: Vim modes
description: The planned modal editing model for Ink prompts.
---

:::caution[Specification, not current behavior]
Modal prompt editing is still under development. This page records the intended behavior, not a working keybinding reference.
:::

## Mode model

Both prompt types are designed to support three core modes:

| Mode   | Purpose                             | Cursor       |
| ------ | ----------------------------------- | ------------ |
| Insert | Enter text                          | Steady bar   |
| Normal | Navigate and issue editing commands | Steady block |
| Visual | Select an inclusive grapheme range  | Steady block |

`textarea` additionally specifies **Visual Line** for whole logical lines and **Visual Block** for rectangular selections across display columns.

The active mode will always appear as a textual label such as `INSERT` or `VISUAL BLOCK`. Color is supplemental, not the only mode cue.

## Selections and operators

The design includes delete, change, and yank over active selections:

- Delete removes selected graphemes, writes them to the unnamed register, and returns to Normal mode.
- Change follows the same deletion rules and enters Insert mode at the start of the removed range.
- Yank preserves the buffer, copies the exact selection, and returns to Normal mode at the selection start.
- Block operators process each selected row independently without removing unselected line breaks.

## Unicode text

Movement and editing are specified in extended grapheme clusters rather than bytes or Unicode scalar values. A visible character made from a base character plus combining marks, or a joined emoji sequence, should move and edit as one unit.

## Viewports

For multiline values, the planned viewport follows the cursor horizontally and vertically without altering the underlying buffer. Layout will be recomputed after terminal resize events, including very narrow and transient zero-sized areas.

Most exact keybindings will be documented after they are implemented and tested. Ctrl-C cancels in every mode, and `q` cancels from Normal mode.
