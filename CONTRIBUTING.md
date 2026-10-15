# Contributing

Thank you for wanting to help. The goal is simple: **the best free, lightweight image compression and conversion app for fast websites ever made**, free and open source forever ([our promise](PROMISE.md)). Everyone is welcome: you do not need to write code to make a real difference.

## Ways to help

| You are… | You could… |
|---|---|
| A user | Try a preview, tell us what was confusing, share before/after results in [Discussions](../../discussions) |
| A designer | Improve the interface, icons, empty states or animation (see `docs/22`, `docs/23`, `docs/27`) |
| An accessibility tester | Try it with a screen reader, keyboard only, or 200% zoom, and report what breaks |
| A compression nerd | Benchmark encoders, tune presets, add quality metrics, try new codecs (with numbers) |
| A Rust or TypeScript developer | Pick an issue labelled `good first issue` or `help wanted` |
| A writer or translator | Improve docs, error messages and future translations |
| A security researcher | Review the parsers and IPC boundary; report privately (see [SECURITY.md](SECURITY.md)) |

Forking and building your own version is welcome too. The GPL licence lets you, as long as what you distribute stays free and open source.

## Making a change

1. For anything larger than a small fix, open an issue or discussion first so we can agree on the approach.
2. Keep pull requests small and focused. One idea per PR.
3. Run the checks before pushing:
   ```bash
   npm run lint && npm run test && npm run build
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   cargo deny check
   ```
4. Explain *why* in code comments where it is not obvious (see `docs/24-code-commenting-and-maintainer-guide.md`), update the docs your change affects, and record real test results in the PR. Encoder changes need before/after size and quality numbers; UI changes need screenshots and accessibility notes.
5. Use only synthetic or freely licensed test images. Never commit personal photos, private paths, tokens or signing keys.

Setup instructions are in the [README](README.md#for-developers). The architecture tour is in [docs/ARCHITECTURE-FOR-CONTRIBUTORS.md](docs/ARCHITECTURE-FOR-CONTRIBUTORS.md).

## The non-negotiables

These protect users and are enforced in review: no network or tracking in the app; never overwrite originals; treat every image as hostile; the web page never gets file paths or file-system access; never claim something is tested, signed or safe unless it is shown. Details: [AGENTS.md](AGENTS.md).

## Licence of contributions

By submitting a contribution you confirm you have the right to do so and agree that it is released under GPL-3.0-or-later. Please be kind: see the [Code of Conduct](CODE_OF_CONDUCT.md).
