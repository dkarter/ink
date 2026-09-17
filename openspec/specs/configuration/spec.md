# configuration Specification

## Purpose

Discover portable user configuration and combine it predictably with command-line choices.

## Requirements

### Requirement: Discover configuration through XDG paths

Ink SHALL read configuration from `$XDG_CONFIG_HOME/ink/config.toml` when `XDG_CONFIG_HOME` is a non-empty absolute path, falling back to `$HOME/.config/ink/config.toml` otherwise.

#### Scenario: Prefer XDG config home {#CFG-001}

- GIVEN both XDG and home fallback configuration files exist
- WHEN `XDG_CONFIG_HOME` is set
- THEN Ink reads only the configuration below `XDG_CONFIG_HOME`

#### Scenario: Fall back to home config {#CFG-002}

- GIVEN `XDG_CONFIG_HOME` is unset and `$HOME/.config/ink/config.toml` exists
- WHEN Ink loads configuration
- THEN that file supplies user defaults

#### Scenario: Ignore invalid XDG config home {#CFG-005}

- GIVEN `XDG_CONFIG_HOME` is empty or a relative path and the home fallback configuration exists
- WHEN Ink loads configuration
- THEN the home fallback file supplies user defaults

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

### Requirement: Validate configuration explicitly

Ink SHALL provide `ink config validate` using the same resolved path and parser as normal startup.

#### Scenario: Validate valid or absent configuration {#CFG-006}

- GIVEN a valid config file or no config file at the resolved path
- WHEN the user runs `ink config validate`
- THEN stdout reports concise success with the resolved path when available and the process exits with status 0

#### Scenario: Diagnose invalid or unreadable configuration {#CFG-007}

- GIVEN invalid TOML, an unsupported setting, or an unreadable file at the resolved path
- WHEN the user runs `ink config validate`
- THEN stderr reports an actionable diagnostic containing that path, stdout is empty, and the process exits with status 1

### Requirement: Publish a complete configuration schema

Ink SHALL publish a JSON Schema for its TOML configuration that documents every supported setting, exact bundled theme name and semantic color role, applicable defaults, and rejects unknown keys.

#### Scenario: Describe the complete config surface {#CFG-008}

- GIVEN the schema served from the website static assets
- WHEN its root and color properties are inspected
- THEN they exactly cover Ink's supported keys, types, theme names, semantic color names, descriptions, defaults where applicable, and disallow additional properties

### Requirement: Associate generated config with its schema

Ink SHALL add a Taplo-compatible `#:schema` URL when it creates or updates configuration while preserving existing user content.

#### Scenario: Mutate configuration without data loss {#CFG-009}

- GIVEN valid hand-written TOML with comments, unrelated settings, and optional color overrides
- WHEN Ink changes the selected theme
- THEN only the top-level theme value and missing schema directive are added or changed
- AND the resulting file remains valid Ink configuration
