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

## Semantic overrides

User configuration overrides individual semantic color roles after loading a bundled base palette. Roles not overridden retain the selected theme’s values.

Unknown theme names, color roles, or color values fail startup before terminal state changes. Colors use `#RRGGBB` syntax; configuration supports the semantic roles listed in the project README.

## Accessibility

The interface design does not use color as the only signal for editing state. Every mode is also identified by a visible text label, and Insert mode uses a bar cursor while the other modes use a block cursor.
