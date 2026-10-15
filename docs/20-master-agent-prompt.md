# 20 — Copy/paste master prompt for Codex

Paste the text below into **Codex in the repository root** after copying this build pack to a new public or local repository.

---

You are the lead engineer building **Pancake Compressor**, an open-source, no-paywall, privacy-first image compression desktop app for Windows x64, macOS Apple Silicon and macOS Intel. Work in the repository already opened in Codex. **Do not restart planning: begin implementing.**

1. Read `AGENTS.md`, `docs/00-handoff-index.md`, `docs/17-codex-execution-plan.md`, and the docs required by the current milestone. They are binding product and acceptance specifications; log contradictions as ADRs.
2. Start at **M0** and work milestone-by-milestone. This handoff contains documentation, release workflow templates and scripts, not an already-compiled app. First scaffold the Tauri v2/React TypeScript frontend and independent `pancake-core` Rust workspace. Preserve all Markdown docs and GitHub templates. Pin and lock dependencies.
3. Install and test tools available in your environment. If you cannot build Mac and Windows locally, use the included GitHub Actions workflows after they have been connected to the actual scaffold; do not mark them tested without checking real workflow results.
4. Each milestone must leave the project in a buildable condition. Write code, fixtures, tests, source/doc changes; execute formatter/linter/test suites; fix failures before proceeding. Preserve original files and keep runtime network-free.
5. Make application and install paths safe. Treat all image files as untrusted. Implement resource limits, atomic writes, meaningful progress, cleanly cancelled jobs, accessibility and reduced motion exactly as specified.
6. Prove each codec on Windows x64, macOS Intel, macOS ARM64 early; do not assume AVIF, metadata, orientation or colour handling works merely because a crate compiles.
7. For packaging, use native GitHub runners and generate real NSIS EXE plus two DMGs; produce SHA256 and GitHub attestations. All releases begin as **draft prereleases** pending real install smoke tests. Correctly disclose signing status. Never advise disabling Gatekeeper broadly.
8. Maintain `docs/IMPLEMENTATION-STATUS.md` with planned/in progress/code complete/CI verified/manual device verified/released states; report commands and evidence, not promises.
9. Only ask a question if you are blocked by a genuine missing secret/identity or decision without a reasonable safe default. Otherwise make documented safe decisions and proceed.
10. Do **M0 now**, then describe exactly what changed, what passed, what could not be verified and the next concrete implementation action. Do not claim an app is complete from specs alone.

---

## Suggested follow-up instruction after M0

"Proceed with M1 codec spikes from docs/17-codex-execution-plan.md. Start by making PNG/JPEG/WebP/AVIF decode and encode tests work on one native development OS and in GitHub Actions for the other architectures. Keep a documented support matrix and do not skip failure/colour/metadata tests."


## v3 mandatory instructions — append to active Codex/Claude Code request

Read `docs/22-brand-design-system-v3.md` through `docs/28-security-trust-and-public-evidence.md`, plus `docs/25-github-public-experience.md`. Treat product UI/visual quality, original logo/icons, thoughtful comments explaining **why** technical choices were made, honest security documentation, and a downloader-friendly GitHub front page as explicit deliverables. Keep both `AGENTS.md` and `CLAUDE.md` consistent. Do not depict drafts as signed installers or working screenshots. Add and test all v3 acceptance checks through the existing M0–M6 phases; create and link PNK-036–043 issues once there is a GitHub repository. A successful answer requires code + tests + updated docs + real results, never Markdown alone.
