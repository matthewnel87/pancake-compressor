# AGENTS.md — Mandatory coding-agent contract

You are implementing **Pancake Compressor**, an open-source, offline-only Windows/macOS desktop app built with Tauri v2, React/TypeScript and Rust. The docs in this repository are product requirements and implementation guidance; their code snippets must be validated against installed versions before use.

## Operating rules

1. **Work in small increments, not one huge unreviewed change.** Follow `docs/17-codex-execution-plan.md` in strict milestone order; finish acceptance checks and report results before moving to the next milestone. Never state tests or installers passed unless they actually ran.
2. **No cloud dependency in the installed app.** No HTTP client, remote images, CDN fonts, tracking/telemetry, user accounts, analytics, crash-report uploads, auto-updater, or external APIs in MVP. GitHub CI and developer dependency retrieval are separate from runtime operation.
3. **Never overwrite originals by default.** Do not recurse into `_pancake`, follow symlinks/reparse points, write to source files, or silently discard image frames, transparency, orientation or colour information. Use per-file explicit failure or skip states.
4. **Treat files as hostile.** Verify magic bytes, file size, dimensions/pixel count and destination policy before decoding; bound simultaneous work using a memory budget; avoid `unwrap()`/panic on inputs; never run commands supplied by filenames.
5. **Front end is untrusted.** All disk I/O is handled by narrow Rust commands; validate paths, options and user permissions in Rust. Keep Tauri capabilities minimal, disable remote content, enforce CSP. Do not casually add shell/fs/http plugins.
6. **Progress reflects measurable facts.** Batch progress = terminally processed files / total; status can be scanning, queued, decoding, encoding, writing, completed, skipped, failed or cancelled. Do not invent per-file percent in codecs that expose no reliable progress. Events monotonic by sequence number.
7. **Codec correctness before UI polish.** Implement deterministic core tests and format matrix first; large files must fail safely instead of crashing when memory is insufficient. No false 'unlimited bytes' promise.
8. **Accessibility is baseline.** Keyboard access, meaningful focus order, contrast, screen-reader announcements and reduced-motion pancake animation.
9. **Free/open-source compliance.** Keep GPL-3.0-or-later licence, check all dependencies via cargo-deny/license checks, include third-party notices; ensure no paywall or premium-only functionality.
10. **Do not leak local paths or user content.** No secrets in logs, no fixtures based on private user images; test fixtures must be synthetic or redistributable with documented licensing.
11. **Production release gating.** Never publish 'trusted/signed/notarized' claims unless verified. Test package installation, app launch, compression, exports and removal on both OS families. Unsigned previews must be clearly labelled.
12. **Document any deviation.** Put trade-offs, upstream dependency choices and unsupported features into `docs/02-decisions-and-open-questions.md` as a new dated ADR.

## Required tools and architecture

- Tauri v2, React, TypeScript, Vite, Tailwind CSS; Rust workspace with an independently testable `pancake-core` crate.
- Use repo-local pinned toolchain and package lockfiles; prefer maintained codec libraries (imagequant/oxipng/mozjpeg/libwebp/ravif). Do not assume they compile on all platforms: prove each with small spikes.
- GitHub Actions standard hosted runners for macOS Intel/Apple Silicon and Windows x64. Produce NSIS EXE and both DMGs; capture SHA256 and provenance evidence.
- PR checks: formatter, lint, UI build/test, Rust fmt/clippy/test, dependency audit, basic file security tests. Native installer smoke-tests required before GA.

## Public-facing documentation and readable code are release requirements

- Treat GitHub as both a **download product** and a contributor workspace. Keep the README downloader-first; keep `README.md`, `SUPPORT.md`, `ACCESSIBILITY.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `PRIVACY.md`, `docs/FAQ.md`, both OS install guides, and review/feedback guide current.
- The project is currently **specifications only**. Do not show working Download buttons, authentic screenshots, performance numbers, scan badges, signed status or success claims until backed by actual produced and checked builds.
- Read `docs/22-brand-design-system-v3.md`, `docs/23-ui-ux-flows-performance-v3.md`, and `docs/27-logo-and-brand-asset-production.md` before implementing the visual interface. The desired result is sleek, warm, quick and lightly quirky, not childish or overanimated.
- Create original, editable brand SVGs and reproducible platform icon exports. Document provenance and licences; compare 3 design sketches. Do not clone competitors' logos or ship emoji as production icon assets.
- **Explain the code for future open-source contributors**: module-level documentation, meaningful Rustdoc, interface contracts, memory budgets, cancellation, filesystem invariants and `unsafe` safety reasoning. Explain why choices were made, not each obvious line. Read `docs/24-code-commenting-and-maintainer-guide.md`.
- Every code PR updates affected public documentation, release notes where relevant and real test/status evidence. Once the repository is created, configure GitHub Discussions and private vulnerability reporting; do not claim the pack itself enables repository settings.
- Claude Code reads `CLAUDE.md` which directs it to this policy; never maintain divergent implementation plans. A second agent should review in a separate branch or worktree.

## Response format after every milestone

```text
MILESTONE:
Changed paths:
Implementation decisions:
Commands executed and outcome:
Tests genuinely passed / not run:
Supported OS builds actually produced:
Known risks and blockers:
Next milestone:
```

## Entry point

Start with `docs/17-codex-execution-plan.md` M0. Read the linked design/test/security specs as needed. If this is a document-only handoff repo, **scaffold the application** first rather than inventing already-existing project files.
