# vim-editing Specification

## Purpose

Provide testable Vim-style editing over Unicode text in single-line and multiline prompts.

## Requirements

### Requirement: Transition between editing modes

Ink SHALL support Normal, Insert, and Visual modes in both prompts, plus Visual Line and Visual Block in textarea.

#### Scenario: Enter and leave Insert mode {#EDIT-001}

- GIVEN a prompt in Normal mode
- WHEN the user enters Insert mode, inserts text, and presses Escape
- THEN the text is inserted and the prompt returns to Normal mode on a valid grapheme

#### Scenario: Append at the end of a line {#EDIT-020}

- GIVEN input or textarea is in Normal mode on a non-empty or empty logical line
- WHEN the user invokes `A` and inserts text
- THEN Ink enters Insert mode at the line-end insertion point, after the final grapheme when the line is non-empty
- AND inserted text is appended without changing adjacent lines or splitting Unicode graphemes
- AND Escape returns to Normal mode on the final inserted grapheme, or the prior final grapheme when nothing was inserted

#### Scenario: Append after the cursor {#EDIT-023}

- GIVEN input or textarea is in Normal mode on a grapheme or an empty logical line
- WHEN the user invokes `a` and inserts text
- THEN Ink enters Insert mode immediately after the cursor grapheme, or at column zero on an empty line
- AND inserted text preserves adjacent Unicode graphemes and remains one undoable Insert action
- AND Escape returns to Normal mode on the final inserted grapheme, or the original grapheme when nothing was inserted

#### Scenario: Select characters visually {#EDIT-002}

- GIVEN a prompt in Normal mode on a non-empty buffer
- WHEN the user enters Visual mode and moves the cursor
- THEN the inclusive grapheme range from the anchor to the cursor is selected

#### Scenario: Select whole lines {#EDIT-003}

- GIVEN textarea contains multiple lines
- WHEN the user enters Visual Line mode and moves vertically
- THEN every complete logical line from the anchor line through the cursor line is selected, including intervening line breaks

#### Scenario: Select a text column {#EDIT-004}

- GIVEN textarea contains lines of unequal display width
- WHEN the user enters Visual Block mode and expands a rectangle
- THEN each intersected line selects graphemes whose display cells overlap the inclusive anchor-to-cursor columns, without splitting a grapheme

### Requirement: Execute command-line actions

Ink SHALL provide a Vim-style command line from Normal mode without modifying the editable value.

#### Scenario: Submit or cancel from the command line {#EDIT-017}

- GIVEN input or textarea is in Normal mode
- WHEN the user enters `:wq` or `:q!` and presses Enter
- THEN the status row becomes the visible command line while the command is being entered
- AND `:wq` accepts the current value while `:q!` cancels without standard output
- AND Escape dismisses the command line and returns to the Normal status row

#### Scenario: Report an invalid command {#EDIT-018}

- GIVEN input or textarea is accepting a command from Normal mode
- WHEN the user submits an unsupported command
- THEN the status row temporarily shows `not a valid command` in the theme's error color
- AND the Normal status row returns after one second or when the user continues

#### Scenario: Confirm submission before quitting {#EDIT-019}

- GIVEN input or textarea is accepting a command from Normal mode
- WHEN the user submits `:q` or `:qa`
- THEN the status row asks `Submit? (y/n)`
- AND `y` accepts the current value
- AND `n` or Escape returns to Normal editing

### Requirement: Apply operators to selections

Ink SHALL apply delete, change, and yank to the active selection and then clear it.

#### Scenario: Delete selected text {#EDIT-005}

- GIVEN any Visual selection
- WHEN the user invokes delete
- THEN exactly the selected graphemes are removed, the unnamed register receives them, and Ink enters Normal mode at the nearest surviving grapheme

#### Scenario: Change selected text {#EDIT-006}

- GIVEN any Visual selection
- WHEN the user invokes change
- THEN deletion follows the same range semantics as delete and Ink enters Insert mode at the start of the removed range

#### Scenario: Yank selected text {#EDIT-007}

- GIVEN any Visual selection
- WHEN the user invokes yank
- THEN the unnamed register receives exactly the selected text, the buffer is unchanged, and Ink enters Normal mode at the selection start

#### Scenario: Block operators preserve rows {#EDIT-008}

- GIVEN a Visual Block selection across lines of unequal width
- WHEN delete, change, or yank is invoked
- THEN the operator processes each row independently in top-to-bottom order and does not add or remove unselected line breaks

#### Scenario: Change text across a visual block {#EDIT-021}

- GIVEN a Visual Block selection spans multiple textarea lines
- WHEN the user invokes change, enters replacement text on the first selected line, and returns to Normal mode
- THEN Ink inserts the same replacement at the start of every selected row
- AND backspace within the replacement updates the text applied to every row
- AND replacement text preserves Unicode grapheme clusters and lines shorter than the selected column
- AND accepting directly from Insert mode finalizes the replacement before emitting the value

### Requirement: Undo and redo edits

Ink SHALL provide bounded text-change history through `u` for undo and `r` for redo in Normal mode.

#### Scenario: Traverse editing history {#EDIT-022}

- GIVEN input or textarea has completed text-changing actions
- WHEN the user invokes `u` or `r` in Normal mode
- THEN `u` restores the text and cursor from the preceding action and `r` restores the most recently undone action
- AND an Insert session, operator change, or Visual Block replacement is one history action
- AND a new text change after undo clears the redo history
- AND unavailable undo or redo commands leave the buffer unchanged

### Requirement: Edit visible characters atomically

Ink SHALL move, select, delete, change, and yank by extended grapheme cluster rather than Unicode scalar value or byte.

#### Scenario: Treat joined Unicode as one character {#EDIT-009}

- GIVEN text containing combining marks or a joined emoji sequence
- WHEN a movement or editing command crosses that visible character
- THEN the complete grapheme cluster is moved across, selected, or modified atomically

### Requirement: Keep Normal cursors valid

Ink SHALL keep a Normal-mode cursor on an existing grapheme or at column zero on an empty logical textarea line.

#### Scenario: Deleting at boundaries keeps a valid cursor {#EDIT-010}

- GIVEN the cursor is at the beginning or end of a buffer or on an empty textarea line
- WHEN deletion removes adjacent or final text or movement targets the empty line
- THEN the cursor is clamped to the nearest remaining grapheme or column zero on that logical line

### Requirement: Move by words

Ink SHALL support Vim word motions in Normal and Visual modes, treating `w`, `b`, and `e` as small-word motions and `W`, `B`, and `E` as whitespace-delimited WORD motions.

#### Scenario: Move by word and WORD boundaries {#EDIT-011}

- GIVEN text containing words, punctuation, whitespace, and Unicode graphemes
- WHEN the user invokes `w`, `b`, `e`, `W`, `B`, or `E`
- THEN the cursor moves to the matching next or previous boundary without splitting a grapheme
- AND an active Visual selection expands to that boundary

### Requirement: Compose operators with motions

Ink SHALL compose delete, change, and yank operators with word and WORD motions and `iw`, `aw`, `iW`, and `aW` text objects in Normal mode.

#### Scenario: Delete or change a motion range {#EDIT-012}

- GIVEN a prompt in Normal mode on text containing words, horizontal whitespace, line breaks, and Unicode graphemes
- WHEN the user invokes `d` or `c` followed by a forward, backward, or end word or WORD motion or text object
- THEN exactly the computed motion or text-object range is deleted
- AND `cw` and `cW` at a word end change only that current word or WORD
- AND horizontal-whitespace text objects do not consume a line break or adjacent-line indentation
- AND delete returns to Normal mode while change enters Insert mode at the start of the deleted range

#### Scenario: Yank and paste operator ranges {#EDIT-013}

- GIVEN a prompt in Normal mode containing characterwise and multiline ranges
- WHEN the user yanks with a forward, backward, or end word or WORD motion, any word text object, or `yy` and invokes `p` or `P`
- THEN the unnamed register is pasted after or before the cursor for characterwise text
- AND linewise text is pasted below or above the current textarea line

#### Scenario: Apply linewise operators {#EDIT-014}

- GIVEN a textarea in Normal mode on a first, middle, final, or empty logical line
- WHEN the user invokes `dd`, `cc`, or `yy`, directly or through the public editor API
- THEN the operation applies only to the complete current logical line
- AND deleting the final line consumes its preceding separator without leaving a trailing empty line
- AND `cc` enters Insert mode while delete and yank remain in Normal mode
- AND linewise APIs reject input buffers and non-Normal modes

### Requirement: Open textarea lines

Ink SHALL support Vim line opening in textarea Normal mode.

#### Scenario: Open a line for insertion {#EDIT-015}

- GIVEN a textarea in Normal mode
- WHEN the user invokes `o` or `O`
- THEN Ink inserts an empty line below or above the current line respectively
- AND enters Insert mode at the start of that line

### Requirement: Use terminal display cells consistently

Ink SHALL use one display-cell policy for editor movement, Visual Block ranges, viewport scrolling, rendering, and cursor placement.

#### Scenario: Align editing geometry with rendered cells {#EDIT-016}

- GIVEN text containing tabs, controls, combining marks, and wide graphemes
- WHEN Ink moves vertically, scrolls, or selects a Visual Block
- THEN editor and UI geometry assign the same cells to every grapheme
- AND no grapheme is split or addressed at a cell where it is not rendered
