# Ink

Ink is a fast, composable terminal input prompt built with Rust, Ratatui, and Crossterm. It takes inspiration from `gum input` and adds first-class Vim editing for single-line input and multiline textareas.

## Install

Install a prebuilt release with mise:

```sh
mise use --global github:dkarter/ink
```

## Interface

```sh
ink input --value "release title"
printf 'first line\nsecond line' | ink textarea
ink input --normal --theme catppuccin-mocha
ink input --prompt "Name: "
ink textarea --fullscreen
ink theme
ink config validate
ink completion zsh > _ink
```

Accepted values are written cleanly to stdout. Interactive input and rendering use the controlling terminal so piped initial values remain compatible with shell composition. Input uses a compact three-row interface, and textarea provides five editable rows unless `--fullscreen` is set. Press Enter to accept an input, Ctrl-D to accept either prompt, Ctrl-C to cancel, or `q` to cancel from Normal mode.

Textarea shows line numbers by default. Set `line-numbers = false` in `config.toml` or pass `--no-line-numbers` to hide them; `--line-numbers` overrides the global setting for one invocation.

### Placeholders

Both prompt commands accept `--placeholder <text>`. Placeholders appear only while the editable value is empty and never become part of the accepted output. Input placeholders are normalized to one line, while textarea placeholders may span multiple lines.

```sh
ink input --placeholder "Release title"
ink textarea --placeholder $'Summary\n\nDetails'
```

Placeholders use a dedicated `placeholder` theme color. Override it through the existing semantic color table:

```toml
[colors]
placeholder = "#a9b1d6"
```

## Configuration

Ink reads `$XDG_CONFIG_HOME/ink/config.toml` when `XDG_CONFIG_HOME` is a non-empty absolute path. Otherwise, it reads `$HOME/.config/ink/config.toml`. Command-line options override matching values in the file.

```text
#:schema https://dkarter.github.io/ink/schema/ink-config.schema.json
normal = true
theme = "catppuccin-mocha"
```

Run `ink theme` to preview every bundled palette and save one to this file. Ink preserves existing settings and comments. Run `ink config validate` to check the resolved file without opening a prompt.

Theme names are case-insensitive and use hyphens as separators. Add the shown `#:schema` directive to an existing hand-written file for Taplo-compatible editor completion and validation.
Single-line input has no prefix or background by default. Use `--prompt "Name: "` to add a prefix; configure `[colors].background` to fill the input row.
Input removes LF, CRLF, and lone CR line breaks from seeds and pasted text. Textarea normalizes CRLF and lone CR to LF. See the [theme role reference](website/src/content/docs/themes.md#semantic-overrides) for every supported `[colors]` key.

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
