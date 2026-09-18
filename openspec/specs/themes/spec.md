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

Ink SHALL allow configuration to override the exact kebab-case semantic color roles `foreground`, `background`, `status-background`, `muted`, `accent`, `border`, `selection`, `selection-foreground`, `cursor`, `insert-mode`, `normal-mode`, `visual-mode`, `error`, and `warning` after the selected bundled base theme is loaded.

#### Scenario: User colors overlay a base theme {#THEME-004}

- GIVEN a selected bundled theme and valid overrides for any supported color roles
- WHEN Ink resolves the palette
- THEN overridden roles use user colors and every other role retains its base-theme value

#### Scenario: Reject unknown theme values {#THEME-005}

- GIVEN an unknown theme name, color role, or color value
- WHEN Ink resolves the palette
- THEN startup fails with a diagnostic naming the invalid value before terminal state changes

#### Scenario: Configure the status background {#THEME-010}

- GIVEN `status-background` is absent or configured with a valid color
- WHEN input or textarea renders a multi-row status line
- THEN each status row retains its existing background by default
- AND an explicit override fills the complete status row with that color while preserving the mode indicator

### Requirement: Browse bundled themes interactively

Ink SHALL provide `ink theme` as an interactive browser that starts on the configured or default theme and live-previews every bundled palette while the user navigates.

#### Scenario: Navigate live theme previews {#THEME-006}

- GIVEN a configured theme and an interactive terminal
- WHEN the user runs `ink theme` and navigates with the displayed keyboard controls
- THEN the browser starts on that theme, identifies the highlighted theme by name, and redraws using each highlighted palette

#### Scenario: Persist an accepted theme {#THEME-007}

- GIVEN an existing, absent, or not-yet-parented configuration file
- WHEN the user accepts a highlighted theme
- THEN Ink writes that theme to the resolved XDG-compatible config path
- AND preserves unrelated settings and comments
- AND adds a Taplo-compatible schema association
- AND emits no standard output

#### Scenario: Leave config unchanged on cancellation {#THEME-008}

- GIVEN any configuration state before the theme browser opens
- WHEN the user cancels with a displayed cancellation control
- THEN Ink exits with cancellation status, emits no standard output, restores terminal state, and does not modify configuration

#### Scenario: Adapt the browser presentation {#THEME-009}

- GIVEN a narrow or resized terminal and theme preview text containing Unicode
- WHEN the theme browser renders or redraws
- THEN the selected theme and usable controls remain visible without panicking or corrupting text
