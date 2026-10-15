# Claude Code handoff — Pancake Compressor

Read and obey [`AGENTS.md`](AGENTS.md) as the **single source of truth** for coding-agent policies. Do not duplicate or contradict it here. Next read `docs/00-handoff-index.md`, `docs/26-agent-collaboration-and-review.md`, and the current milestone in `docs/17-codex-execution-plan.md`.

## Working agreement
- The canonical implementer is Codex unless the maintainer explicitly assigns Claude Code a scoped task or independent review. Do not make concurrent changes to the same branch/working tree.
- Prefer a feature branch or separate worktree and an ordinary PR with commands actually executed, screenshots (when UI changes) and tests.
- Never claim unexecuted CI, unsigned installers, threat-model controls, branding assets or release trust badges are verified.
- Make code legible for people: document non-obvious logic, image-format trade-offs, byte/pixel budgets, failure cases, security invariants and Rust ↔ Tauri ↔ React interfaces. See `docs/24-code-commenting-and-maintainer-guide.md`.
- A design idea or illustrative placeholder is not a shipped app icon. Before public release, deliver the source SVG plus reproducible Windows/macOS icons per `docs/27-logo-and-brand-asset-production.md`.
- Prioritise responsiveness, accessibility, safety and clarity over decorative animations.
