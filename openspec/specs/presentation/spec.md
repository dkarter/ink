# presentation Specification

## Purpose

Keep editing state legible and usable across terminal sizes and prompt viewports.

## Requirements

### Requirement: Show mode through cursor shape

Ink SHALL use a steady bar cursor in Insert mode and a steady block cursor in every other mode.

#### Scenario: Cursor shape follows mode {#UI-001}

- GIVEN each supported editing mode
- WHEN Ink renders the cursor
- THEN Insert uses a bar and Normal, Visual, Visual Line, and Visual Block use a block

### Requirement: Label the active mode

Ink SHALL display the current mode using a visible textual indicator that does not rely on color alone.

#### Scenario: Mode indicator names every mode {#UI-002}

- GIVEN the user transitions through supported modes
- WHEN each frame is rendered
- THEN the indicator reads INSERT, NORMAL, VISUAL, VISUAL LINE, or VISUAL BLOCK as applicable

### Requirement: Adapt single-line layout

Ink SHALL preserve an editable cell and visible cursor in narrow terminals, and use additional width for prompt, value, and mode information when available.

#### Scenario: Input remains usable when narrow {#UI-003}

- GIVEN an input terminal narrower than its preferred layout
- WHEN Ink renders or the terminal shrinks
- THEN optional chrome is compacted or omitted before the editable cell and cursor are lost

### Requirement: Viewport multiline text

Ink SHALL keep the textarea cursor visible horizontally and vertically without changing the underlying buffer.

#### Scenario: Textarea scrolls around cursor {#UI-004}

- GIVEN textarea content exceeds the available rows or columns
- WHEN the cursor moves beyond the visible viewport
- THEN the viewport scrolls by display cells and logical lines until the complete cursor grapheme is visible

### Requirement: Respond to resize events

Ink SHALL recompute layout and viewport bounds from the latest terminal dimensions after every resize event.

#### Scenario: Resize triggers bounded redraw {#UI-005}

- GIVEN a real prompt is running in a terminal
- WHEN the terminal reports larger, narrow, and zero-sized dimensions where supported
- THEN the next frame uses the new area and clamps all dimensions without panicking
- AND editing, acceptance, and cleanup continue after the terminal returns to a usable size

### Requirement: Preserve the command background

Ink SHALL leave the single-line input background transparent unless the user configures the `background` color role.

#### Scenario: Input background is opt-in {#UI-006}

- GIVEN a bundled theme without an explicit user background override
- WHEN input renders
- THEN its prompt and editable cells retain the terminal background
- AND a configured `background` override fills those cells with the requested color
