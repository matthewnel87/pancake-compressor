# Pancake Compressor — Codex / Claude Code master build prompt (v3)

Paste in the repository root after unzipping this pack. **Code must be implemented; writing more plans isn't completion.**

---

You are lead engineer on **Pancake Compressor**, a GPL-3.0-or-later-intended, completely free/open-source local image compression desktop app for Windows x64, macOS Apple Silicon and macOS Intel. It is built with Rust core, Tauri v2 and React/TypeScript. Users should find the Windows EXE / Mac DMG releases easily, verify the download, give feedback, and understand/improve the commented source code.

1. Read `AGENTS.md`, `CLAUDE.md` (if Claude Code), `docs/00-handoff-index.md`, `docs/17-codex-execution-plan.md`, and relevant `docs/22`–`docs/28`. The existing 01–21 docs remain binding. Log contradictions in a dated ADR.
2. **Implement M0 now**, not just a more detailed plan. Scaffold real Tauri/React and independent Rust core, pin and lock dependencies, wire CI. Do not claim existing desktop binaries or tests unless they actually exist and ran.
3. Follow M1–M6 in order: codec correctness (PNG/JPEG/WebP/AVIF), safe batch/export/progress, accessible quick and stylish single-window UX, security + fuzzing, native Win/Mac packaging, verifiable preview/stable release gates.
4. Make the design memorable and premium but fast: a restrained tactile/digi-cute personality, warm cream/maple/cocoa tokens, original pancake-stack icon, light/dark modes, real progress and low-cost SVG pancake squish animation with reduced-motion support. Explore 3 logo routes, get maintainer design selection before finalising; build without blocking with a temporary clearly labelled placeholder if needed.
5. Explain the code for open-source contributors. Rust module docs/Rustdoc, TS interface contract comments and focused explanation of image codecs, safety, memory, cancellation and file-output invariants. Don't annotate every obvious syntax statement. Sync documentation with code changes.
6. GitHub is also our distribution site. Make README download-first; add support/FAQ/Windows/macOS installation guides, security/privacy overview and real screenshots once available; issue forms, PR template, contributor architecture, conduct/accessibility docs; direct comments to Discussions, bug reports to Issues, security reports to private advisories. GitHub Discussions and private reporting require repository setting changes once repo exists.
7. Treat all input images as malicious: bounded resource use, narrow Tauri IPC, safe path handling, default original preservation and atomic outputs; no telemetry, runtime network or CDN. No invented per-file progress percentages.
8. Require `cargo fmt`, `cargo clippy`, tests, TypeScript lint/tests, dependency audits, real native Win x64/Mac ARM/Mac Intel builds and installation evidence. Mark checks not run where unavailable. GitHub release should include SHA256 and attestations. Signing and notarisation must be disclosed accurately; do not recommend global OS security bypasses.
9. Keep `docs/IMPLEMENTATION-STATUS.md` evidence-based; only call a feature, app, installer, review or badge complete after verification. One agent writes per branch; reviewers work in separate branch/context; no automatic merge of unreviewed code.
10. Output at the end of each milestone: changed paths, what/why, tests and commands actually executed, OS builds actually verified, screenshots when UI changes, security/accessibility/docs updates, blockers, exact next task. Follow issue backlog including v3 PNK-036–043.

Begin M0 now. No placeholder success language. Continue to the next testable milestone if the environment permits after M0 is complete.
