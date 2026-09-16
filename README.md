# Ink

Ink is a fast, composable terminal input prompt built with Rust, Ratatui, and Crossterm. It takes inspiration from `gum input` and adds first-class Vim editing for single-line input and multiline textareas.

## Interface

```sh
ink input --value "release title"
printf 'first line\nsecond line' | ink textarea
ink input --normal --theme catppuccin-mocha
ink input --prompt "Name: "
ink textarea --fullscreen
ink completion zsh > _ink
```

Accepted values are written cleanly to stdout. Interactive input and rendering use the controlling terminal so piped initial values remain compatible with shell composition. Input uses a compact three-row interface, and textarea provides five editable rows unless `--fullscreen` is set. Press Enter to accept an input, Ctrl-D to accept either prompt, Ctrl-C to cancel, or `q` to cancel from Normal mode.

### Planned placeholders (not implemented)

Both prompt commands will accept `--placeholder <text>`. Input placeholders will be normalized to one line, while textarea placeholders may span multiple lines.

```sh
ink input --placeholder "Release title"
ink textarea --placeholder $'Summary\n\nDetails'
```

Placeholders will use a dedicated `placeholder` theme color. The planned override uses the existing semantic color table:

```toml
[colors]
placeholder = "#a9b1d6"
```

## Configuration

Ink reads `$XDG_CONFIG_HOME/ink/config.toml` when `XDG_CONFIG_HOME` is a non-empty absolute path. Otherwise, it reads `$HOME/.config/ink/config.toml`. Command-line options override matching values in the file.

```toml
normal = true
theme = "catppuccin-mocha"
```

Theme names are case-insensitive and use hyphens as separators.

## Development

Install pinned tools and run all checks through mise:

```sh
mise install
mise ci
```

Product behavior is specified in [`openspec/specs`](openspec/specs). Every scenario has a stable ID and exactly one linked Rust test. Any ignored tests in `tests/specs/` are explicit placeholders for capabilities that have not shipped.

The documentation site lives in [`website`](website) and is built with Astro and Starlight.

## License

MIT
