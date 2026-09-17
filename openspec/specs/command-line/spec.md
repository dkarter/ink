# command-line Specification

## Purpose

Expose composable single-line and multiline prompt commands with predictable initial state and results.

## Requirements

### Requirement: Prompt for one line

Ink SHALL provide `ink input` for editing and accepting exactly one logical line.

#### Scenario: Accept single-line input {#CLI-001}

- GIVEN an interactive terminal and `ink input`
- WHEN the user edits text containing no line break and accepts it
- THEN Ink emits that text as one accepted value

#### Scenario: Reject line breaks in input {#CLI-002}

- GIVEN `ink input` is active
- WHEN an explicit value, piped seed, or bracketed paste contains LF, CRLF, or lone CR line breaks
- THEN Ink keeps a single logical line by removing every line-break character

### Requirement: Prompt for multiple lines

Ink SHALL provide `ink textarea` for editing and accepting text containing line breaks.

#### Scenario: Accept multiline text {#CLI-003}

- GIVEN an interactive terminal and `ink textarea`
- WHEN an explicit value, piped seed, keyboard edit, or bracketed paste supplies multiline text
- THEN Ink normalizes CRLF and lone CR line endings to LF in the logical buffer
- AND emits all other content unchanged

### Requirement: Choose the startup mode

Ink SHALL start in Insert mode unless the user requests Normal mode.

#### Scenario: Default to Insert mode {#CLI-004}

- GIVEN no startup-mode option or configuration
- WHEN either prompt opens
- THEN the active mode is Insert

#### Scenario: Start in Normal mode {#CLI-005}

- GIVEN the user requests Normal startup mode
- WHEN either prompt opens
- THEN the active mode is Normal and the text is not modified

### Requirement: Seed the editable value

Ink SHALL accept initial text from a command-line option and from piped standard input, with an explicit command-line value taking precedence.

#### Scenario: Seed from command line {#CLI-006}

- GIVEN `--value` contains initial text
- WHEN the prompt opens
- THEN the editable buffer contains that text with the cursor at its end in Insert mode

#### Scenario: Seed from piped input {#CLI-007}

- GIVEN standard input is a pipe and `--value` is absent
- WHEN the prompt opens
- THEN Ink reads the pipe to end-of-file and uses its contents as the editable value

#### Scenario: Command-line seed wins {#CLI-008}

- GIVEN both piped input and `--value` are present
- WHEN the prompt opens
- THEN `--value` is used and piped input does not alter the buffer

### Requirement: Fit command workflows

Ink SHALL render input and textarea as compact inline prompts by default, while allowing input prompt text to be configured and textarea to use the full terminal.

#### Scenario: Configure the input prompt {#CLI-009}

- GIVEN `ink input` with or without `--prompt <text>`
- WHEN the prompt renders
- THEN no prefix is shown by default
- AND configured text prefixes the editable value when requested

#### Scenario: Keep textarea compact by default {#CLI-010}

- GIVEN `ink textarea` without a layout option
- WHEN the prompt renders
- THEN Ink reserves five editable rows and one status row within the command workflow
- AND startup does not wait for a terminal cursor-position report

#### Scenario: Expand textarea to full screen {#CLI-011}

- GIVEN `ink textarea --fullscreen`
- WHEN the prompt renders
- THEN Ink uses the complete available terminal area
- AND isolates that area from the command workflow using the alternate screen

#### Scenario: Remove prompt UI after completion {#CLI-012}

- GIVEN a compact or fullscreen prompt is visible
- WHEN it accepts, cancels, errors, or panics
- THEN every owned compact row is cleared or the fullscreen alternate screen is left
- AND the command workflow screen and cursor are restored before control returns
