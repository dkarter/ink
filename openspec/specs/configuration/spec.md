# configuration Specification

## Purpose

Discover portable user configuration and combine it predictably with command-line choices.

## Requirements

### Requirement: Discover configuration through XDG paths

Ink SHALL read configuration from `$XDG_CONFIG_HOME/ink/config.toml`, falling back to `$HOME/.config/ink/config.toml` when `XDG_CONFIG_HOME` is unset.

#### Scenario: Prefer XDG config home {#CFG-001}

- GIVEN both XDG and home fallback configuration files exist
- WHEN `XDG_CONFIG_HOME` is set
- THEN Ink reads only the configuration below `XDG_CONFIG_HOME`

#### Scenario: Fall back to home config {#CFG-002}

- GIVEN `XDG_CONFIG_HOME` is unset and `$HOME/.config/ink/config.toml` exists
- WHEN Ink loads configuration
- THEN that file supplies user defaults

### Requirement: Apply explicit options last

Ink SHALL resolve built-in defaults, then user configuration, then command-line options.

#### Scenario: Command line overrides configuration {#CFG-003}

- GIVEN a setting differs between built-in defaults, configuration, and the command line
- WHEN Ink resolves settings
- THEN the explicit command-line value wins

### Requirement: Reject invalid configuration safely

Ink SHALL identify invalid files and settings without entering terminal raw mode.

#### Scenario: Invalid configuration is actionable {#CFG-004}

- GIVEN configuration contains invalid syntax, an unknown theme, or an unsupported value
- WHEN Ink starts
- THEN stderr identifies the file and setting, stdout is empty, and the process exits with status 1
