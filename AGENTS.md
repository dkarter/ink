# Ink Development

- Use safe, idiomatic Rust and conventional commits.
- Define observable behavior in `openspec/specs/<capability>/spec.md` before implementation.
- Give every scenario a unique `{#ABC-001}` ID and start its Rust test with `abc_001_`.
- Replace ignored bootstrap scenario stubs as behavior is implemented; do not silently remove links.
- Run `mise ci` before pushing. It mirrors GitHub CI and includes strict OpenSpec traceability.
- Verify terminal UI changes interactively in input and textarea, including resize, cancellation, piped stdin, Unicode, and terminal restoration.
