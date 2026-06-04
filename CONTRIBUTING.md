# Contributing

Encom is in **Phase 0** — repo just stood up. The fastest way to be useful right now is one of:

1. **Try `cargo build --workspace`** on Linux/macOS/Windows and report any platform-specific friction.
2. **Open issues** for missing model providers you want — name + docs link is enough to start.
3. **Skill examples** — drop a directory under `examples/skills/` with an `encom.toml` and an `index.ts`.

PR rules:

- One concern per PR. Refactors and feature work go in separate PRs.
- `cargo fmt` and `cargo clippy --workspace -- -D warnings` must pass.
- Don't add a dependency without saying why in the PR description.

License: MIT. By contributing, you agree your contribution is MIT-licensed.
