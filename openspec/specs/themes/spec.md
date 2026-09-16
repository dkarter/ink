# themes Specification

## Purpose

Provide accessible bundled color schemes with deterministic selection and user overrides.

## Requirements

### Requirement: Default to Tokyo Night

Ink SHALL use the bundled Tokyo Night theme when no theme is selected.

#### Scenario: Use default theme {#THEME-001}

- GIVEN no theme configuration or option
- WHEN Ink resolves presentation colors
- THEN the Tokyo Night palette is active

### Requirement: Bundle popular themes

Ink SHALL bundle Tokyo Night; Catppuccin Latte, Frappe, Macchiato, and Mocha; Dracula; Gruvbox Dark; Nord; Solarized Dark; and Solarized Light.

#### Scenario: Select every bundled theme {#THEME-002}

- GIVEN the canonical name of any bundled theme
- WHEN the user selects it
- THEN Ink resolves a complete palette without filesystem access

### Requirement: Select themes predictably

Ink SHALL accept theme names from configuration and `--theme`, with command-line selection taking precedence and names matched case-insensitively with hyphens as separators.

#### Scenario: Command-line theme wins {#THEME-003}

- GIVEN configuration and `--theme` select different valid themes
- WHEN Ink resolves the theme
- THEN the command-line theme is the base palette

### Requirement: Override theme roles

Ink SHALL allow configuration to override individual semantic color roles after the selected bundled base theme is loaded.

#### Scenario: User colors overlay a base theme {#THEME-004}

- GIVEN a selected bundled theme and valid overrides for some color roles
- WHEN Ink resolves the palette
- THEN overridden roles use user colors and every other role retains its base-theme value

#### Scenario: Reject unknown theme values {#THEME-005}

- GIVEN an unknown theme name, color role, or color value
- WHEN Ink resolves the palette
- THEN startup fails with a diagnostic naming the invalid value before terminal state changes
