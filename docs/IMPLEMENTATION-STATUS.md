# Implementation status — evidence ledger

Updated: 2026-10-13. **This handoff is specifications/workflow templates only. No application code, installers, compiled artefacts, dependency audits or manual installation tests have been run.**

| Area | Status | Evidence / blocker |
|---|---|---|
| Product requirements and coding instructions | Prepared | `docs/00`–`docs/20` |
| Privacy/security/governance document templates | Prepared | `SECURITY.md`, `PRIVACY.md`, `CONTRIBUTING.md` |
| GitHub Actions workflow templates | Prepared, unrun | `.github/workflows/` — requires real app scaffold |
| Rust codec project and image support | Not implemented | Codex M0 and M1 |
| React/Tauri UI and animation | Not implemented | Codex M0/M3 |
| Windows x64 NSIS build | Not built or tested | Codex M5 |
| Mac Apple Silicon DMG | Not built or tested | Codex M5 |
| Mac Intel DMG | Not built or tested | Codex M5 |
| GitHub attestations/checksums | Not generated | Only source CI workflow/scripts prepared |
| Windows signing | Not provisioned | Optional SignPath Foundation application/other |
| Apple notarisation | Not provisioned | Paid Developer ID programme and CI secrets required |
| Manual clean OS installation | Not tested | Needs devices/VMs |

Add timestamped dated rows with PR/commit and CI run links, reproducible command output, OS version, and reviewer for each feature. Don't overwrite historic negative results.

## v3 preparation update — 2026-10-15

Documentation templates for downloader-first GitHub, dual coding agents, logo/design system, maintainable comments, public trust and feedback were added. **No GitHub settings, application source implementation, live logo design approval, release binaries, runtime security checks or installer smoke tests were performed by this documentation update.**

## M0 local evidence — 2026-10-15 (Claude Code, branch `m0-bootstrap`, macOS 26.1 arm64)

Toolchain recorded: Node 24.15.0, npm 11.12.1, Rust 1.99.1 (rustup, pinned in `rust-toolchain.toml`), Xcode at `/Applications/Xcode.app`, Tauri 2.12.4, create-tauri-app 4.8.0.

| Area | Status | Evidence |
|---|---|---|
| Tauri v2 + React/TS/Vite scaffold, Rust workspace, `pancake-core` crate | Implemented | Official `create-tauri-app` react-ts template, merged and reduced (ADR-020) |
| `get_app_info` IPC command | Implemented, unit-tested | `src-tauri/src/commands.rs`; 2 tests incl. camelCase contract |
| Front-end checks | Passed locally | `npm run lint` (0 warnings), `npm run test` (3/3), `npm run build` (JS 222 kB / 69.7 kB gzip, CSS 11.8 kB) |
| Rust checks | Passed locally | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked` (8 unit/integration + 1 doctest) |
| Contrast of colour tokens | Measured | All text pairs ≥ 4.5:1 after ADR-018; values recorded in `src/styles/tokens.css` |
| macOS arm64 DMG | Built locally, **not install-tested on a clean machine** | `npm run tauri build -- --target aarch64-apple-darwin --bundles dmg` with `APPLE_SIGNING_IDENTITY=-`. DMG 1,902,351 bytes, SHA256 `b55ce1c54c22dc1f80b14f51916d01e2ce8f6574fe1c6ddfed376dceabc6f292` (local build, not a release). App 3.8 MB, arm64 only, `LSMinimumSystemVersion` 13.0 |
| DMG behaviour | Checked | Mounts without a licence prompt (after ADR-023), contains app + Applications link |
| Bundle identifier | Changed after this build | The DMG above was built before the bundle identifier was finalised (ADR-016), so CI builds supersede it |
| Signature | Ad-hoc only | `codesign --verify --deep --strict` passes; `Signature=adhoc`, hardened runtime flag set. `spctl --assess` **rejects** it: Gatekeeper will block downloaded copies. Not notarised |
| Launch smoke test (build machine, not clean OS) | Passed | Process started, main window 960×640 on screen within ~0.75 s of `open`, still running after 6 s at ~100 MB RSS. `lsof -i` showed 0 sockets for the app process. WebKit helper processes were **not** inspected, so this is not yet a full no-network proof |
| Screenshot | Not captured | Terminal lacks macOS Screen Recording permission |
| Windows x64 NSIS, macOS Intel DMG | **Not built** | Needs GitHub Actions run (blocked on push permission for workflow files) |
| GitHub CI on 3 runners | **Not run** | Moving to `matthewnel87/pancake-compressor`; push pending GitHub CLI login for that account |
| Dependency audits (`npm audit`, `cargo audit`), licence check | `npm install` reported 0 vulnerabilities; `cargo audit` and licence tree **not run** | CI `audit` job, and cargo-deny still to be added |

**Gate M0 is not yet met**: it requires green Windows and macOS (Intel + ARM) CI bundles.

## M1 step 1: format sniffing and preflight — 2026-10-15 (branch `m1-format-sniffing`, stacked on `m0-bootstrap`)

| Area | Status | Evidence |
|---|---|---|
| Magic-byte detection (PNG, JPEG, WebP, AVIF) and named unsupported formats (GIF, BMP, TIFF, HEIF, JPEG XL, uncommon JPEG processes) | Implemented, unit-tested | `crates/pancake-core/src/format.rs`; `cargo test --workspace --locked` on macOS arm64: 22 engine tests + 3 doctests pass |
| Header dimensions and alpha hint | Implemented, unit-tested | Synthetic header fixtures per format (`fixtures/README.md`) |
| Animated input refusal (APNG, animated WebP, AVIF sequence) | Implemented, unit-tested | `animated_inputs_are_refused_not_flattened` |
| Decompression-bomb refusal before decode | Implemented, unit-tested | `preflight_refuses_decompression_bombs_before_decoding` (ADR-024) |
| Robustness against hostile headers | Tested, **not fuzzed** | Every truncation of every fixture and 20,000 seeded random inputs return without panic. Real fuzzing (cargo-fuzz) is M4 work |
| Checked against real files from other encoders | **Not done** | Fixtures are hand-built headers; real encoder output is checked once codecs land |
| Windows / macOS Intel runs of these tests | **Not run** | Will run in CI when this branch has a PR |
