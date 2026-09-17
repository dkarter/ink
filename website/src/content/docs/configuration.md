---
title: Configuration
description: Configuration discovery, syntax, and precedence for Ink.
---

## File location

Ink looks for one user configuration file:

1. `$XDG_CONFIG_HOME/ink/config.toml` when `XDG_CONFIG_HOME` is a non-empty absolute path
2. `$HOME/.config/ink/config.toml` when `XDG_CONFIG_HOME` is unset, empty, or relative

When the XDG path is active, Ink will not also read the home fallback.

## Precedence

Settings resolve in this order, from lowest to highest priority:

1. Built-in defaults
2. User configuration
3. Explicit command-line options

For example, `--theme nord` will override a theme selected in `config.toml`.

## Settings

The configuration file accepts these settings:

- `normal`: whether the prompt starts in Normal mode (`false` by default)
- `theme`: a case-insensitive bundled theme name, using hyphens as separators (`tokyo-night` by default)
- `colors`: semantic `#RRGGBB` overrides applied after the selected theme

```toml
normal = true
theme = "catppuccin-mocha"

[colors]
selection = "#33467c"
normal-mode = "#7aa2f7"
```

See [Themes](../themes/#semantic-overrides) for every supported color role.

## Invalid settings

Invalid syntax, unknown themes, unsupported values, and unknown settings fail before Ink enters terminal raw mode. The diagnostic identifies the file and setting, standard output remains empty, and the process exits with status `1`.
