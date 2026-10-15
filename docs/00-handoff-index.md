# 00 — Codex handoff map and status

**Status at delivery:** documentation, release workflow templates and licence file created and packaged. **No functional app, test evidence, installed binaries, GitHub repository or code signing have been produced by this handoff.** Treat all future requirements as not implemented until a commit and passing test prove otherwise.

## Authoritative files

| Document | Scope | Used in |
|---|---|---|
| `01-product-and-scope.md` | Vision, personas, MVP/out-of-scope, feature acceptance | All |
| `02-decisions-and-open-questions.md` | Baseline stack and architecture decisions | M0–M5 |
| `03-architecture-and-ipc.md` | Modules, job state machine, IPC and progress | M0–M4 |
| `04-stack-dependencies-and-licences.md` | Dependencies, codec spikes, licensing | M0–M2 |
| `05-repository-structure.md` | Expected repository layout and scaffolding | M0 |
| `06-user-stories-and-acceptance.md` | Behavioural contract | M1–M4 |
| `07-image-format-contract.md` | Codec semantics and conversion matrix | M1–M3 |
| `08-batching-output-and-memory.md` | Queue, safety, output constraints | M2–M3 |
| `09-ux-progress-and-brand.md` | Brand, UX copy, progress states and animations | M3 |
| `10-threat-model.md` | Threats, mitigations and security proof | M0–M5 |
| `11-test-plan-and-benchmarks.md` | Cross-format test fixtures, CI and evidence | M1–M5 |
| `12-accessibility-and-design-system.md` | Layout, keyboard and motion contract | M3 |
| `13-build-install-release.md` | Windows/macOS packaging and install smoke tests | M0/M5 |
| `14-signing-and-provenance.md` | Code signing, notarisation, checksum/attestation | M5 |
| `15-release-documentation.md` | End-user installation, troubleshooting and security page | M5 |
| `16-github-governance-and-automation.md` | Public repo, CI, dependency automation and community | M0/M5 |
| `17-codex-execution-plan.md` | Ordered deliverables and explicit acceptance gates | All |
| `18-costs-risks-and-roadmap.md` | Free vs optional costs, risk register and v1.1 work | All |
| `19-research-and-technical-sources.md` | URLs and exact verification targets | All |
| `20-master-agent-prompt.md` | Paste-ready Codex instruction | M0 onward |
| `21-release-qa-checklist.md` | Human release verification form | M5/M6 |
| `IMPLEMENTATION-STATUS.md` | Actual implementation evidence; nothing pre-checked | All |
| `20-issue-backlog.md` | 35 tracked engineering tasks with dependencies | All |
| `21-public-trust-center.md` | Public-facing security and privacy evidence | M5 |

## Independent verification

During execution, maintain `docs/IMPLEMENTATION-STATUS.md` with each feature's state: planned / code complete / tested macOS / tested Windows / released. Always link CI logs, manual test device + OS versions, and release hashes when available. This avoids shipping specs dressed up as proof.

## Definition of first public preview

A real installer for Win x64 and two real Mac DMGs, launched on target hardware/VMs, able to optimise mixed still images offline without corrupting originals; honest unsigned/signed labels; downloadable as clearly **preview** release. A stable `v1.0.0` requires the full gates in docs 11, 13 and 14.

## Additional public/open-source and brand requirements (v3)

- `22-brand-design-system-v3.md` — tactile/digi-cute restrained visual identity, tokens, light/dark, quality gates.
- `23-ui-ux-flows-performance-v3.md` — one-window workflow, progress rules, proposed responsiveness budgets and acceptance tests.
- `24-code-commenting-and-maintainer-guide.md` — meaningful comments and architectural explanations.
- `25-github-public-experience.md` — download-first GitHub README, Discussions and reviews, issue categories.
- `26-agent-collaboration-and-review.md` — Codex primary, Claude Code optional reviewer, single canonical plan.
- `27-logo-and-brand-asset-production.md` — logo concepts, original SVG and native icon deliverables.
- `28-security-trust-and-public-evidence.md` — transparent trust signals tied to actual release test evidence.
- `ARCHITECTURE-FOR-CONTRIBUTORS.md`, `SECURITY-OVERVIEW.md`, `INSTALL-WINDOWS.md`, `INSTALL-MACOS.md`, `FAQ.md`, `REVIEW-AND-FEEDBACK.md` — public-facing material.
- Root `CLAUDE.md`, `SUPPORT.md`, `CODE_OF_CONDUCT.md`, `ACCESSIBILITY.md` and extra GitHub issue forms.

These documents elaborate existing security/UX specs. Where differences appear, prefer stronger verified safety and log a dated ADR; never delete existing requirements silently.
