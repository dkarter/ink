# completions-and-performance Specification

## Purpose

Make Ink discoverable in common shells without compromising measurable startup speed.

## Requirements

### Requirement: Generate shell completions from the CLI definition

Ink SHALL generate completion scripts from the same static `usage-rs` definition used to parse commands and flags.

#### Scenario: Generate common shell completions {#COMP-001}

- GIVEN bash, zsh, or fish is requested
- WHEN `ink completion <shell>` runs
- THEN stdout contains a completion script that calls the installed Ink binary for candidates

#### Scenario: Generate a portable shell completion {#COMP-002}

- GIVEN Nushell is requested
- WHEN `ink completion nu` runs
- THEN stdout contains a Nushell completion script generated from the same CLI definition

#### Scenario: Completion generation has no prompt side effects {#COMP-003}

- GIVEN a supported completion target
- WHEN its script is generated
- THEN Ink does not read prompt input, open the controlling terminal, or emit terminal control bytes

### Requirement: Keep normal startup independent of external completion tools

Ink SHALL parse normal invocations without spawning the `usage` executable or reading a generated specification file.

#### Scenario: Normal startup uses compiled tables {#PERF-001}

- GIVEN `ink input` or `ink textarea` starts
- WHEN command-line arguments are parsed
- THEN parsing uses compiled static tables and launches no helper process

### Requirement: Enforce a deterministic startup budget

Ink SHALL provide a benchmark harness that measures a non-interactive startup probe from a prebuilt release binary.

#### Scenario: Startup benchmark uses stable sampling {#PERF-002}

- GIVEN a prebuilt release binary on an otherwise idle supported runner
- WHEN the startup benchmark runs
- THEN it performs 20 warm-up invocations followed by 100 measured invocations and reports median elapsed wall time

#### Scenario: Startup stays within budget {#PERF-003}

- GIVEN the benchmark protocol and a release binary whose filesystem pages are warmed
- WHEN measured on the pinned CI runner
- THEN median `ink --version` wall time is no more than 20 milliseconds
- AND the benchmark is opt-in outside the pinned performance job so ordinary CI is not timing-flaky
