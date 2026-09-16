# Ink Development

- Use safe, idiomatic Rust and conventional commits.
- Keep code clean and modular. Prefer focused files and modules with clear responsibilities; files should stay reasonably small unless there is a concrete justification, without adding unnecessary indirection.
- In parallel work, respect assigned module and test ownership, avoid unrelated files, and communicate shared-surface changes before editing to prevent conflicts.
- Define observable behavior in `openspec/specs/<capability>/spec.md` before implementation.
- Give every scenario a unique `{#ABC-001}` ID and start its Rust test with `abc_001_`.
- Replace ignored bootstrap scenario stubs as behavior is implemented; do not silently remove links.
- Run `mise ci` before pushing. It mirrors GitHub CI and includes strict OpenSpec traceability.
- For every terminal-facing change, build Ink and complete a thorough end-to-end pass of that built binary in dedicated Herdr panes; unit and widget tests do not replace this pass.
- Exercise both input and textarea through Insert, Normal, Visual, Visual Line, and Visual Block modes; resize and narrow layouts; Unicode; piped stdin; stdout cleanliness; acceptance, cancellation, and exit statuses; terminal restoration; and all feature-specific behavior such as placeholders.
