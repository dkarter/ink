---
title: Themes
description: Bundled themes and semantic color overrides for Ink.
---

## Default

Tokyo Night is the default when neither configuration nor `--theme` selects another palette.

## Bundled palettes

Ink bundles these themes without requiring filesystem access:

- Tokyo Night
- Catppuccin Latte
- Catppuccin Frappe
- Catppuccin Macchiato
- Catppuccin Mocha
- Dracula
- Gruvbox Dark
- Nord
- Solarized Dark
- Solarized Light

Theme names are matched case-insensitively with hyphens as separators. For example:

```sh
ink textarea --theme catppuccin-mocha
```

## Interactive browser

Run `ink theme` to browse all bundled palettes. The browser starts on the configured theme, or Tokyo Night when none is configured, and applies each palette as you move with Up/Down or `j`/`k`. Enter saves the highlighted theme; Escape, `q`, or Ctrl-C cancels without changing configuration. Controls remain visible in narrow layouts.

## Semantic overrides

User configuration overrides individual semantic color roles after loading a bundled base palette. Roles not overridden retain the selected theme’s values.

| Role                   | Purpose                   |
| ---------------------- | ------------------------- |
| `foreground`           | Primary text              |
| `background`           | Filled widget backgrounds |
| `muted`                | Hints and secondary text  |
| `placeholder`          | Empty-value guidance      |
| `accent`               | Reserved accent color     |
| `border`               | Reserved border color     |
| `selection`            | Selected-cell background  |
| `selection-foreground` | Selected-cell text        |
| `cursor`               | Reserved cursor color     |
| `insert-mode`          | Insert mode indicator     |
| `normal-mode`          | Normal mode indicator     |
| `visual-mode`          | Visual mode indicators    |
| `error`                | Reserved error color      |
| `warning`              | Reserved warning color    |

Use the exact kebab-case role name with a `#RRGGBB` value:

```toml
[colors]
selection = "#33467c"
selection-foreground = "#c0caf5"
normal-mode = "#7aa2f7"
```

Single-line input leaves the terminal background untouched by default. Set the `background` role explicitly when a filled input row is preferred.

Unknown theme names, color roles, or color values fail startup before terminal state changes.

## Accessibility

The interface design does not use color as the only signal for editing state. Every mode is also identified by a visible text label, and Insert mode uses a bar cursor while the other modes use a block cursor.

Every bundled placeholder color has at least 4.5:1 contrast against its background.
