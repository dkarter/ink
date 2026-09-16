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
