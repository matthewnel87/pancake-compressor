# Contributing

Pancake Compressor welcomes accessibility, UI, compression-engine, testing and documentation contributions. Follow `AGENTS.md` and the current milestone. Create an issue before a large change; keep PRs small; include before/after performance evidence for encoder changes and screenshots/accessibility notes for UI changes.

## Developer setup (once M0 scaffolding exists)

Install Node LTS, Rust stable, platform-specific Tauri prerequisites. Run `npm ci`, `npm run build`, `npm run test`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`. Actual commands may be refined in M0 once workspace layout is final. Use `npm run tauri dev` for the desktop app.

Use only synthetic or permissively licensed fixtures. Don't commit personal images, API tokens, test output, release signing credentials or private crash logs. Report security flaws privately as specified in `SECURITY.md`. By submitting a contribution you agree that it is available under GPL-3.0-or-later and that you have rights to contribute it.
