# placeholders Specification

## Purpose

Provide helpful empty-state guidance without making placeholder text part of the value being edited or accepted.

## Requirements

### Requirement: Accept prompt placeholder text

Ink SHALL accept `--placeholder <text>` on both `ink input` and `ink textarea`.

#### Scenario: Set a placeholder for either prompt {#PH-001}

- GIVEN `ink input` or `ink textarea`
- WHEN the command is started with `--placeholder <text>`
- THEN Ink uses that text as the prompt's placeholder without adding it to the editable value

### Requirement: Show placeholders only for empty buffers

Ink SHALL render the placeholder if and only if the editable buffer is empty.

#### Scenario: Show the placeholder at empty startup {#PH-002}

- GIVEN either prompt has a placeholder and no explicit or piped initial content
- WHEN its first frame is rendered
- THEN the placeholder is visible and the editable buffer remains empty

#### Scenario: Initial content suppresses the placeholder {#PH-003}

- GIVEN either prompt has a placeholder and receives non-empty initial content from `--value` or piped standard input
- WHEN its first frame is rendered
- THEN the initial content is visible and no placeholder text is rendered

### Requirement: Keep placeholders outside editor state and output

Ink SHALL treat placeholder text as presentation only: it SHALL NOT enter the editor buffer, selection, yank contents, delete operations, or accepted output.

#### Scenario: Editing operations cannot act on a placeholder {#PH-004}

- GIVEN an empty prompt is displaying a placeholder
- WHEN the user enters Normal or any Visual mode and selects, yanks, or deletes
- THEN the editor buffer stays empty, nothing from the placeholder is selected or yanked, and the placeholder remains available to render

#### Scenario: Accept an untouched empty prompt {#PH-005}

- GIVEN an empty prompt is displaying a placeholder
- WHEN the user accepts without inserting content
- THEN Ink exits with status 0 and stdout contains exactly one newline for the accepted empty value, with no placeholder text or terminal control bytes

### Requirement: Follow buffer emptiness while editing

Ink SHALL stop rendering the placeholder immediately after content enters the buffer and render it again immediately after editing makes the buffer empty.

#### Scenario: Hide and restore the placeholder during editing {#PH-006}

- GIVEN an empty prompt is displaying a placeholder
- WHEN the user inserts content and then edits the buffer back to empty
- THEN the next frame after insertion omits the placeholder and the next frame after the buffer becomes empty shows it again

### Requirement: Present placeholders safely for each prompt shape

Ink SHALL normalize an input placeholder to one logical line using the same line-break removal as input values, while preserving line breaks in a textarea placeholder and clipping its display to the available terminal cells without changing editor state.

#### Scenario: Normalize an input placeholder to one line {#PH-007}

- GIVEN `ink input --placeholder <text>` where the text contains LF, CR, or CRLF line breaks
- WHEN the empty input is rendered
- THEN the displayed placeholder is the same single logical line produced by input value normalization

#### Scenario: Clip a multiline textarea placeholder safely {#PH-008}

- GIVEN an empty textarea has a multiline placeholder containing narrow, wide, or combining Unicode graphemes
- WHEN it is rendered in an area too narrow or short for the complete placeholder, including a zero-sized intermediate area
- THEN rendering clips to the available display cells and rows without panicking, changing the empty buffer, or moving the logical cursor from its pre-render position

### Requirement: Theme placeholders semantically

Ink SHALL provide a dedicated `placeholder` semantic color role in every bundled theme, with at least 4.5:1 contrast against that theme's background, and SHALL allow the role to be changed through the existing color override mechanism.

#### Scenario: Every bundled theme supplies an accessible placeholder color {#PH-009}

- GIVEN any bundled theme is selected
- WHEN Ink resolves its complete palette
- THEN the palette contains a distinct `placeholder` role whose contrast against `background` is at least 4.5:1

#### Scenario: Override the placeholder color {#PH-010}

- GIVEN configuration sets `[colors].placeholder` to a valid color
- WHEN Ink resolves the selected bundled theme and user color overrides
- THEN the resolved `placeholder` role uses the configured color and all non-overridden roles retain their bundled values
