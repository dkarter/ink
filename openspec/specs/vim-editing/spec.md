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

### Requirement: Edit visible characters atomically

Ink SHALL move, select, delete, change, and yank by extended grapheme cluster rather than Unicode scalar value or byte.

#### Scenario: Treat joined Unicode as one character {#EDIT-009}

- GIVEN text containing combining marks or a joined emoji sequence
- WHEN a movement or editing command crosses that visible character
- THEN the complete grapheme cluster is moved across, selected, or modified atomically

### Requirement: Keep Normal cursors valid

Ink SHALL keep a Normal-mode cursor on an existing grapheme when the buffer is non-empty and at zero when empty.

#### Scenario: Deleting at boundaries keeps a valid cursor {#EDIT-010}

- GIVEN the cursor is at the beginning or end of a buffer
- WHEN deletion removes adjacent or final text
- THEN the cursor is clamped to the nearest remaining grapheme or zero for an empty buffer

### Requirement: Move by words

Ink SHALL support Vim word motions in Normal and Visual modes, treating `w`, `b`, and `e` as small-word motions and `W`, `B`, and `E` as whitespace-delimited WORD motions.

#### Scenario: Move by word and WORD boundaries {#EDIT-011}

- GIVEN text containing words, punctuation, whitespace, and Unicode graphemes
- WHEN the user invokes `w`, `b`, `e`, `W`, `B`, or `E`
- THEN the cursor moves to the matching next or previous boundary without splitting a grapheme
- AND an active Visual selection expands to that boundary

### Requirement: Compose operators with motions

Ink SHALL compose delete, change, and yank operators with word motions and `iw`/`aw` text objects in Normal mode.

#### Scenario: Delete or change a motion range {#EDIT-012}

- GIVEN a prompt in Normal mode on text containing words and whitespace
- WHEN the user invokes `d` or `c` followed by a word motion, `iw`, or `aw`
- THEN exactly the computed motion or text-object range is deleted
- AND delete returns to Normal mode while change enters Insert mode at the start of the deleted range

#### Scenario: Yank and paste operator ranges {#EDIT-013}

- GIVEN a prompt in Normal mode
- WHEN the user yanks with a motion, text object, or `yy` and invokes `p` or `P`
- THEN the unnamed register is pasted after or before the cursor for characterwise text
- AND linewise text is pasted below or above the current textarea line

#### Scenario: Apply linewise operators {#EDIT-014}

- GIVEN a textarea in Normal mode
- WHEN the user invokes `dd`, `cc`, or `yy`
- THEN the operation applies to the complete current logical line
- AND `cc` enters Insert mode while delete and yank remain in Normal mode

### Requirement: Open textarea lines

Ink SHALL support Vim line opening in textarea Normal mode.

#### Scenario: Open a line for insertion {#EDIT-015}

- GIVEN a textarea in Normal mode
- WHEN the user invokes `o` or `O`
- THEN Ink inserts an empty line below or above the current line respectively
- AND enters Insert mode at the start of that line
