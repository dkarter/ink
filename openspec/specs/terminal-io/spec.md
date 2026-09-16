# terminal-io Specification

## Purpose

Compose cleanly in shell pipelines while controlling and restoring an interactive terminal safely.

## Requirements

### Requirement: Separate interaction from accepted output

Ink SHALL render and issue terminal control sequences through the controlling terminal while reserving stdout for the accepted value.

#### Scenario: Accepted output is clean {#IO-001}

- GIVEN an interactive prompt
- WHEN the user accepts a value
- THEN stdout contains only the value followed by one newline and no terminal control bytes

#### Scenario: Piped input retains terminal control {#IO-002}

- GIVEN stdin is a pipe and a controlling terminal such as `/dev/tty` is available
- WHEN Ink reads the initial value from stdin
- THEN subsequent input and all UI output use the controlling terminal while stdout remains composable

### Requirement: Report cancellation predictably

Ink SHALL cancel on Ctrl-C in every mode and on `q` in Normal mode without emitting a value.

#### Scenario: Cancel without output {#IO-003}

- GIVEN a running prompt in any mode
- WHEN the user presses Ctrl-C or presses `q` in Normal mode
- THEN stdout is empty and Ink exits with status 130

### Requirement: Use stable exit statuses

Ink SHALL exit with status 0 after acceptance, 2 for command-line usage errors, and 1 for runtime failures.

#### Scenario: Exit statuses identify outcomes {#IO-004}

- GIVEN acceptance, invalid arguments, or a runtime failure
- WHEN Ink exits
- THEN its status is respectively 0, 2, or 1

### Requirement: Require an interactive terminal

Ink SHALL fail before entering raw mode when no controlling terminal is available.

#### Scenario: Fail clearly without a TTY {#IO-005}

- GIVEN stdin is not interactive and no controlling terminal can be opened
- WHEN a prompt command starts
- THEN Ink writes a concise diagnostic to stderr, emits nothing to stdout, and exits with status 1

### Requirement: Restore terminal state

Ink SHALL restore cursor visibility, cursor shape, raw mode, and the screen region it owns on acceptance, cancellation, runtime errors, and caught panics.

#### Scenario: Restore after every ordinary outcome {#IO-006}

- GIVEN Ink has changed terminal state
- WHEN it accepts, cancels, or returns an error
- THEN every changed terminal setting is restored before the process exits

#### Scenario: Restore after panic {#IO-007}

- GIVEN Ink has changed terminal state and a panic occurs
- WHEN the panic hook runs
- THEN terminal restoration is attempted before the original panic is reported
