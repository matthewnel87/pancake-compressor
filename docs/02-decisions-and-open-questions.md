# 02 — Architecture decisions / ADR ledger

Decisions marked **provisional** require M0/M1 evidence; do not treat guesses as established requirements.

| ID | Decision | Reason | Status |
|---|---|---|---|
| ADR-001 | Product name Pancake Compressor | User-selected brand | Final |
| ADR-002 | Tauri v2 + React/TypeScript/Vite/Tailwind | Thin native desktop, rich UX, Rust IPC | Final |
| ADR-003 | Rust `pancake-core` crate separate from desktop | Test codecs without GUI; future CLI possible | Final |
| ADR-004 | No runtime networking or updater in MVP | Privacy/security/reproducibility | Final |
| ADR-005 | GPL-3.0-or-later anticipated | `imagequant` GPL licensing compatibility; audit before publish | Provisional pending audit |
| ADR-006 | Output `_pancake/`; never overwrite originals by default | Data loss prevention | Final |
| ADR-007 | Windows x64 NSIS + macOS ARM64/Intel DMG | Distributable desktop apps | Final |
| ADR-008 | OS versions: Windows 11 x64, macOS 13+ | Testable support window | Provisional validation |
| ADR-009 | Smart Auto rule-based; no cloud model | Predictable, zero cost, local | Final |
| ADR-010 | Still images only in v0.1 | Prevent animation loss | Final |
| ADR-011 | No implicit metadata discard; clear policy in format spec | Privacy and visual fidelity | Final |
| ADR-012 | Public-repo GitHub Actions standard runners | Free build and test automation | Final |
| ADR-013 | Unsigned preview is clearly marked; public stable Mac release ideally signed/notarised | Respect OS security | Final policy, funding unresolved |
| ADR-014 | Use native OS dialogs and Rust file operations; avoid broad FS plugin | Reduce attack surface | Final |
| ADR-015 | Default 'avoid bigger outputs' policy | Avoid regression but report skipped files | Final |

## Decisions left for codec spike (do not guess)

1. **Exact encoder binding and versions** across Win x64, macOS x64/ARM64; native library linking or vendoring, patch licensing.
2. **AVIF input decoder**: `image` crate with `avif-native` feature + dav1d or alternate audited decoder; packaging has to pass on all platforms.
3. **Metadata**: which encoders can preserve ICC/EXIF/XMP natively? Where unsupported, clearly warn or block sensitive modes. Establish truthful default.
4. **Color correctness**: profile conversion/embedding decisions; perform golden-image comparisons (not merely raw RGB delta).
5. **Smart Auto**: compressed candidates and scoring. Start with bounded candidates; benchmark speed/time/memory.
6. **Mac app signing**: Apple Developer Program funding or preview-only distribution with user-facing explanation. No hidden bypasses.
7. **Windows code signing**: evaluate SignPath Foundation after public release or paid certificate; no approval assumed.

## Change control

For every new decision: create `ADR-NNN` entry with date, options considered, security/licensing consequence, tests, and reason. Do not silently change output semantics, permissions or supported OSes.

## ADRs added during M0 (2026-10-15, Claude Code, branch `m0-bootstrap`)

| ID | Decision | Options considered | Consequence / reason | Status |
|---|---|---|---|---|
| ADR-016 | Bundle identifier `io.github.matthewnel87.pancakecompressor` | `io.github.<maintainer>` placeholder; a custom domain | Uses the maintainer's GitHub Pages namespace, which the maintainer controls. Changing it after release breaks macOS preferences and Windows upgrade paths, so confirm before the first public preview. | Provisional, maintainer to confirm |
| ADR-017 | Pin Node 24 LTS in `.nvmrc` and Rust 1.99.1 in `rust-toolchain.toml`; CI reads Node from `.nvmrc` and installs Rust 1.99.1 explicitly | Keep CI on `node-version: '22'` and `rust-toolchain@stable` | Node 24 is the active LTS and is required by the current Vite 8 / ESLint 10 toolchain range; local development already ran on 24.15.0. Floating `stable` would make CI differ from local builds. Bump both deliberately in their own PRs. | Final for M0 |
| ADR-018 | Light-mode `--accent` changed from `#AD6025` to `#A65B22` | Keep the proposal; use accent only for large text | `#AD6025` measured 4.48:1 on `--surface` (fails 4.5:1 for normal text). `#A65B22` measures 4.84:1, and white text on it 5.07:1. Visually near-identical. | Final |
| ADR-019 | Stage installers in `release-artifacts/`, not `dist/` | Change Vite `outDir` instead | Vite builds the web front end into `dist/`, so CI would have uploaded web assets alongside the installer and `verify_staged.py` would always fail. Tauri's `frontendDist` keeps its default. | Final |
| ADR-020 | Remove the scaffold's `opener` plugin; window capability limited to `core:app:default`, `core:event:default`, `core:window:default` | Keep `core:default` plus `opener:default` | The opener plugin can launch URLs and files, which M0 does not need (AGENTS.md rule 5). Narrow capability set reduces what a compromised webview could do. | Final for M0; revisit when the dialog plugin is added in M2 |
| ADR-021 | Windows NSIS `installMode: currentUser`; macOS `minimumSystemVersion: 13.0` | Per-machine install (needs admin) | Per-user install avoids an admin prompt and matches docs/13. macOS 13 matches ADR-008. Both still need real install tests. | Provisional pending install tests |
| ADR-022 | `pancake-core` forbids `unsafe` and denies `unwrap`/`expect`/`panic!` via crate lints; privacy flag enforced by a compile-time assertion | Rely on review only | Turns AGENTS.md rules 2 and 4 into build failures instead of review comments. | Final |
| ADR-023 | Do not set `bundle.licenseFile` | Show the GPL as an installer/DMG licence page | With `licenseFile`, the DMG refused to open until the user clicked "Agree", and NSIS adds a licence page. The GPL does not require acceptance to run the program, so a click-through misrepresents it and adds friction. The licence ships in the source and stays declared via `bundle.license`. Third-party notices bundle is still required before release (docs/04). | Final |
