# START HERE — Codex or Claude Code

This ZIP is a **GitHub-ready build specification and automation template**, not a compiled desktop app. It preserves the original v2 engine/security/release docs and adds v3 public GitHub onboarding, documentation standards, design and brand instructions.

## To build
1. Create or open your own repo `pancake-compressor` and put these files in its root.
2. In **Codex** (recommended primary), open repository root. Read `AGENTS.md`, `docs/00-handoff-index.md`, `docs/17-codex-execution-plan.md`, `docs/22`–`docs/28`.
3. In **Claude Code** (optional second implementer/reviewer), root `CLAUDE.md` points to the same `AGENTS.md` and plan. Do not let two agents edit the same working branch concurrently.
4. Paste `docs/20-master-agent-prompt.md` or the separate v3 master prompt in the ZIP. Execute M0, then M1–M6 with actual tests.
5. Enable GitHub Discussions and private vulnerability reporting in your repository settings; these are not enabled just by adding Markdown files.
6. Do not publish downloads until the correct native installers exist, are tested, and their signing status is clearly disclosed.

## First real outcomes
Scaffold Tauri v2 + React/TS + Rust core, pin toolchains, compile blank shell on Windows and Mac CI. Record evidence in `docs/IMPLEMENTATION-STATUS.md`. Then prove supported codec conversion and security behaviours before UI polish.

## Priority for public users
**Download → Know it works → Trust → Give feedback → Read/build source**. Keep front page straightforward. Read `docs/25-github-public-experience.md`.
