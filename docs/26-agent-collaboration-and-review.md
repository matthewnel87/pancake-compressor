# 26 — Codex primary, Claude Code optional: collaboration contract

## Choice
Use **Codex as primary implementer** because the project needs coordinated Rust/React edits, tests, native build workflows and verifiable GitHub pull requests. **Claude Code** is an equally valid alternative implementer and an optional independent reviewer for UI quality or security/architecture consistency. This is a workflow preference, not a benchmark-proven universal superiority claim.

## One canonical source of truth
- `AGENTS.md` applies to both agents; `CLAUDE.md` points to it rather than forking requirements.
- `docs/17-codex-execution-plan.md` determines order; `docs/00-handoff-index.md` maps docs; `docs/IMPLEMENTATION-STATUS.md` records real evidence.
- Do not launch two write-enabled agents against the same worktree/branch; use separate branches or worktrees and a maintainer-reviewed merge.

## Operating sequence
1. Codex takes one numbered PNK issue, reads relevant security/product/docs, names precise acceptance tests.
2. Codex implements smallest testable increment. It runs commands locally and logs evidence, or marks unrun checks.
3. When UI changed: provide real screenshots on Win/Mac in light, dark, reduced-motion states and at 200% scaling.
4. When image code changed: include hostile-input tests, save-original invariants, codec fixture results and peak memory observations where applicable.
5. An independent review (Claude Code or Codex review with a separate context) checks correctness, threat model, UX clarity, tests, docs and code comments; do not automatically merge generated findings.
6. Maintainer approves and merges; tag/release only after separate release checks.

## Code review prompts
**Security:** "Review this diff for malicious image-file parsing, destination/path traversal, unsafe memory, Tauri IPC permissions, private log leaks, error handling and supply-chain changes. Give file/line references, severity and reproducible fixes. Do not change code in review mode."

**UX:** "Review actual screenshots and frontend source against docs/22, docs/23, docs/12. Identify friction in a first-time 10-image batch, 200% zoom, dark mode, keyboard-only flow and reduced motion. Prioritise issues. Do not replace the design system without approval."

**Documentation:** "Compare README, installation, privacy and release notes with what the code and CI actually do. Remove unverified claims. Ensure downloadable releases have accurate platform and signing instructions."

## Deliverable after every PR
Changed paths; what and why; how code works; commands genuinely run; what failed/not run; screenshots or CI links where available; risk; documentation links; next issue. No success theatre.
