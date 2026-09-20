# Development

## Setup

Install the pinned tools with mise:

```sh
mise install
```

## Checks

Run the same checks as GitHub CI before pushing:

```sh
mise ci
```

Individual tasks include `mise fmt`, `mise fmt-check`, `mise lint`, `mise test`, `mise openspec-check`, and `mise website-build`.

## Specifications

Product behavior is specified in [`openspec/specs`](openspec/specs). Every scenario has a stable ID and exactly one linked Rust test. Any ignored tests in `tests/specs/` are explicit placeholders for capabilities that have not shipped.

Define observable behavior in the relevant specification before implementing it. Give each scenario a unique ID and begin its Rust test name with the lowercase form of that ID.

## Documentation Website

The documentation site lives in [`website`](website) and is built with Astro and Starlight. Pitchfork runs the development server on port `4321`, or the next available port, and provides a stable HTTPS URL.

Start it and open its worktree-aware Pitchfork URL in the default browser with:

```sh
mise website-open
```

To start it without opening a browser, use:

```sh
pitchfork start website
```

The primary checkout is available at:

```text
https://website.ink.localhost/
```

Linked worktrees receive distinct URLs in this form:

```text
https://website.<worktree>.ink.localhost/
```

### One-Time macOS Proxy Setup

The committed `pitchfork.toml` configures TLS on the unprivileged proxy port `8443`. Each development machine must separately configure macOS DNS, trust Pitchfork's local certificate authority, and redirect standard HTTPS port `443` to `8443`.

After cloning the repository and running `mise install`, run:

```sh
pitchfork start website
pitchfork proxy setup
pitchfork supervisor start --force
pitchfork start website
pitchfork proxy doctor
```

`pitchfork proxy setup` requests administrator authorization because it updates the macOS resolver, certificate trust, and `pf` rules. A successful `pitchfork proxy doctor` reports that the proxy listener, DNS resolver, system resolution, certificate, and standard port all pass.

To start Pitchfork automatically after login, register its user-level service without `sudo`:

```sh
pitchfork boot enable
```

The machine-specific certificate and network configuration are not stored in this repository. Repeat the one-time setup on every Mac used for development.

To run Astro without Pitchfork, use:

```sh
mise website-dev
```
