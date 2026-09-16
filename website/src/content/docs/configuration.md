---
title: Configuration
description: Planned configuration discovery and precedence for Ink.
---

:::caution[Not implemented]
Configuration loading is specified but not available in the current bootstrap. The example below is illustrative and may change before release.
:::

## File location

Ink is designed to look for one user configuration file:

1. `$XDG_CONFIG_HOME/ink/config.toml` when `XDG_CONFIG_HOME` is set
2. `$HOME/.config/ink/config.toml` otherwise

When the XDG path is active, Ink will not also read the home fallback.

## Precedence

Settings will resolve in this order, from lowest to highest priority:

1. Built-in defaults
2. User configuration
3. Explicit command-line options

For example, `--theme nord` will override a theme selected in `config.toml`.

## Proposed shape

The specification establishes settings for startup mode, base theme, and semantic color roles. Final TOML keys have not been stabilized. A future configuration may resemble:

```toml
# Illustrative only; not accepted by the current bootstrap.
theme = "tokyo-night"
start_mode = "insert"

[colors]
mode_insert = "#9ece6a"
```

## Invalid settings

Invalid syntax, unknown themes, unsupported values, and unknown color roles are designed to fail before Ink enters terminal raw mode. The diagnostic will identify the file and setting, standard output will remain empty, and the process will exit with status `1`.
