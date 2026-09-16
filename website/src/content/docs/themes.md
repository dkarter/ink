---
title: Themes
description: Planned bundled themes and semantic color overrides for Ink.
---

:::caution[Not implemented]
Theme resolution and prompt rendering are still design work. The names on this page come from the current specification.
:::

## Default

Tokyo Night is the planned default when neither configuration nor `--theme` selects another palette.

## Bundled palettes

Ink is specified to bundle these themes without requiring filesystem access:

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

Theme names will be matched case-insensitively with hyphens as separators. For example, the intended command-line form is:

```sh
ink textarea --theme catppuccin-mocha
```

## Semantic overrides

User configuration is designed to override individual semantic color roles after loading a bundled base palette. Roles not overridden will retain the selected theme’s values.

Unknown theme names, color roles, or color values will fail startup before terminal state changes. The exact role names and accepted color syntax will be documented once implemented.

## Accessibility

The interface design does not use color as the only signal for editing state. Every mode is also identified by a visible text label, and Insert mode uses a bar cursor while the other modes use a block cursor.
