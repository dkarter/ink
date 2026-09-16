# Ink Development

- Use safe, idiomatic Rust and conventional commits.
- Keep code clean and modular. Prefer focused files and modules with clear responsibilities; files should stay reasonably small unless there is a concrete justification, without adding unnecessary indirection.
- In parallel work, respect assigned module and test ownership, avoid unrelated files, and communicate shared-surface changes before editing to prevent conflicts.
- Define observable behavior in `openspec/specs/<capability>/spec.md` before implementation.
- Give every scenario a unique `{#ABC-001}` ID and start its Rust test with `abc_001_`.
- Replace ignored bootstrap scenario stubs as behavior is implemented; do not silently remove links.
- Run `mise ci` before pushing. It mirrors GitHub CI and includes strict OpenSpec traceability.
- Verify terminal UI changes interactively in input and textarea, including resize, cancellation, piped stdin, Unicode, and terminal restoration.
