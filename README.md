# Ink

Ink is a planned fast, composable terminal input prompt built with Rust, Ratatui, and Crossterm. It takes inspiration from `gum input` and adds first-class Vim editing for single-line input and multiline textareas.

> [!IMPORTANT]
> Ink is in its bootstrap phase. The CLI, specifications, tests, documentation, and release infrastructure compile and validate, but interactive prompt behavior is intentionally not implemented yet.

## Planned interface

```sh
ink input --value "release title"
printf 'first line\nsecond line' | ink textarea
ink input --normal --theme catppuccin-mocha
ink completion zsh > _ink
```

Accepted values will be written cleanly to stdout. Interactive input and rendering will use the controlling terminal so piped initial values remain compatible with shell composition.

## Development

Install pinned tools and run all checks through mise:

```sh
mise install
mise ci
```

Product behavior is specified in [`openspec/specs`](openspec/specs). Every scenario has a stable ID and exactly one linked Rust test. Ignored tests in `tests/scenarios.rs` are explicit implementation placeholders, not claims of completed behavior.

The documentation site lives in [`website`](website) and is built with Astro and Starlight.

## License

MIT
