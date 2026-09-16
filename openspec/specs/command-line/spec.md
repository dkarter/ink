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
- WHEN the user pastes or types a line break
- THEN Ink keeps a single logical line by removing line-break characters

### Requirement: Prompt for multiple lines

Ink SHALL provide `ink textarea` for editing and accepting text containing line breaks.

#### Scenario: Accept multiline text {#CLI-003}

- GIVEN an interactive terminal and `ink textarea`
- WHEN the user inserts line breaks and accepts the buffer
- THEN Ink emits the complete multiline buffer unchanged

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
