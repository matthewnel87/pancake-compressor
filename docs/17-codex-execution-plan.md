# 17 — Codex execution plan: milestone-by-milestone

This is the **master implementation checklist**. Codex is expected to implement, compile, test, document, and commit one milestone at a time. All unchecked items are *not implemented*. Start with `AGENTS.md`, the product spec and `00-handoff-index.md`.

## Deliverables and ordering

| Stage | Target deliverable | Completion gate |
|---|---|---|
| M0 | Tauri + React + Rust core scaffold, compiles with lockfiles, PR CI wired | Windows and macOS smoke builds green in real CI |
| M1 | Real decoding/encoding of PNG/JPEG/WebP/AVIF | Matrix tests and licence checks green on 3 architectures |
| M2 | Safe batch processor, presets, folder layout, cancellation | Deterministic tests and corrupt-file/resource tests |
| M3 | Accessible friendly pancake UI, preview, progress, export | User-journey tests and actual offline workflow |
| M4 | Security, fuzzing, privacy, robustness and benchmarks | Threat controls demonstrated; security report written |
| M5 | Installers, verified release assets, real platform tests | Draft preview release and release evidence |
| M6 | Optional stable release | Signing/notarisation, public support documentation and final gates |

### M0 — Bootstrap and cross-platform smoke test

**Read:** docs 02–05, 13, 16, 19.

- [ ] Confirm working directory and existing user work; do not overwrite existing source. Save ADRs if deviations needed.
- [ ] Install Rust toolchain, Node LTS, native build prerequisites; record versions. Scaffold **Tauri v2 + React/TS/Vite** using official supported generator. Choose `npm` package manager, check in `package-lock.json`.
- [ ] Create `crates/pancake-core` pure Rust crate in root Cargo workspace; add `src-tauri` workspace member. Check in `Cargo.lock` and `rust-toolchain.toml` after validation.
- [ ] Configure app id, icons, CSP, bounded commands, minimal capabilities, brand and dev scripts; no runtime network or updater. Specifically set Windows WebView2 `offlineInstaller` and document the size tradeoff; provide Mac ad-hoc identity for preview builds.
- [ ] Add one Tauri command `get_app_info` that returns version + runtime privacy indication; no file permissions needed for it.
- [ ] Add a minimal accessible launch screen with functional native window. Use Tauri-generated starter icons until proper icons are created.
- [ ] Wire `npm run build`, `npm run test`, `npm run lint`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`.
- [ ] Make `.github/workflows/ci.yml` pass on Windows x64 + macOS Intel + ARM; fix any toolchain/linker breakage before M1. Build one preliminary NSIS and both DMGs in CI as evidence of packaging toolchain.
- [ ] Record screenshots, command outputs and SHA in `docs/IMPLEMENTATION-STATUS.md`. No 'working compressor' claim yet.

**Gate M0:** fresh checkout installs, builds the frontend and launches branded blank shell on Windows/Mac. GitHub build matrix produces all target bundles (not yet a functional image compressor).

### M1 — Codec feasibility first (four formats)

**Read:** docs 04, 07, 11, 19.

- [ ] Build `ImageInput`/`ImageOutput` contracts, magic-byte validation, extension detection, still-image vs animated detector.
- [ ] Implement still PNG/JPEG/WebP/AVIF decoding; spikes on every OS/architecture; make AVIF decoder dependency/linking explicit.
- [ ] Implement original-format lossless PNG via oxipng and lossy indexed PNG via imagequant/lodepng; verify alpha and palette correctness.
- [ ] Implement JPEG MozJPEG and lossless/lossy WebP with dimensions and transparency policies.
- [ ] Implement AVIF still output using ravif or alternative proven build; measure throughput and visual quality.
- [ ] Decode then re-open every written output; verify proper dimensions, format bytes, colour/orientation/transparency behaviour. Animated sources fail clearly; never flatten silently.
- [ ] Create synthetic + licensed redistributable fixtures covering orientation, ICC, alpha and malformed data.
- [ ] Record exact dependencies, SPDX licences, native libs, size/time/memory results for three builds.

**Gate M1:** each supported input/output mapping has a passing or documented explicitly-unsupported test across platforms. No silent flattening/metadata loss.

### M2 — Batch engine, resources, output safety, presets

**Read:** docs 03, 06–08, 10.

- [ ] Implement native file/folder picker and user-granted input registration. Rust validates every path; don't trust JS-supplied arbitrary paths.
- [ ] Traversal preserves nested structure and refuses symlinks, path escapes, file duplicates, special device files, destination-as-input recursion.
- [ ] Task scheduler bounded by both CPU parallelism and memory budget, with reliable cancellation tokens. Prove no 'spawn per file' unbounded fan-out.
- [ ] Atomic temp file in output directory, validate output and only then rename into final destination. Keep originals by default; handle output conflicts.
- [ ] Implement `Smart Auto`, quality presets and no-bigger default based on actual codec outcomes; set transparent clear winner/fallback rules.
- [ ] Rust emits versioned structured events with job id, file id, phase, sequence number, count and saving bytes (no fake per-file %).
- [ ] Build task tests for 500 mixed small files, nested folders, cancellation, corrupt inputs, zero-byte, already-optimised and huge-dimension inputs.

**Gate M2:** no data loss and no unbounded crashes in acceptance corpus; results reconcile exactly with queued count.

### M3 — Accessible GUI and pancake UX

**Read:** docs 09, 12, 06.

- [ ] Build main screen and native drag-drop + Browse Files + Browse Folder control (keyboard alternative).
- [ ] Controls: Smart Auto/Keep Format/WebP/AVIF/JPEG/PNG, preset, output folder, optional resize, lossless options where available.
- [ ] File queue and informative processing states, real batch progress, honest size results, skip/fail counts, cancellation.
- [ ] Add pancake stack squashing **only while jobs active**; disable with `prefers-reduced-motion`.
- [ ] Side-by-side before/after image preview at matched zoom and correct dimensions; robust fallback for unpreviewable outputs.
- [ ] End-of-batch summary and Open Output Folder; clear/try-again; errors with accessible plain-language suggestions.
- [ ] Verify keyboard/screen reader flows, 200% scaling, colour contrast and stale async event handling.

**Gate M3:** non-technical person can process a mixed batch from a fresh install with no guidance, offline.

### M4 — Security/privacy hardening

**Read:** SECURITY.md, PRIVACY.md, docs 10–11, 14.

- [ ] Restrictive CSP and Tauri capabilities; no shell/http updater/CDN/remote image fonts; inspect dependencies for unexpected networking.
- [ ] Hostile-image fuzz harness/corpus for decoder adapters (libFuzzer or equivalent Rust tooling). Resource caps on decompression bombs; return errors rather than process termination.
- [ ] Test destination traversal, symlink swap/TOCTOU, path separators/case collisions, temp cleanup and cancellation race conditions.
- [ ] Test offline using disconnected VM + platform network monitoring; record results, don't claim mathematically impossible proof of zero network behaviour.
- [ ] Run cargo audit/deny and npm audit; resolve critical/high findings or block release with explicit justification.
- [ ] Dependency SBOM and THIRD_PARTY_NOTICES; verify GPL obligations and source-to-binary correspondence.
- [ ] Assess native decoder isolation if any risk cannot be controlled in-process.

**Gate M4:** defensible security documentation and evidence; all blocking risk items resolved.

### M5 — Actual installers and preview release

**Read:** docs 13–16 and docs/evidence template.

- [ ] Run tag-triggered `.github/workflows/release.yml`, inspect EXE and both DMGs plus public artifact attestations.
- [ ] Verify release script fails on any missing bundle; confirm SHA256SUMS matches downloadable bytes.
- [ ] Perform **separate real clean-install tests** on Windows x64, Mac ARM64, Mac Intel (not merely GitHub build completion). Verify operation entirely offline, UI, output, uninstall, no file deletion.
- [ ] Document clear unsigned macOS/Windows warning status; Apple signing is not free for consumer distribution; never tell people to blanket-disable OS security.
- [ ] Complete release notes and source/installer link documentation. Publish **preview** only after manual gate; otherwise keep draft.

**Gate M5:** installable *preview* artifacts actually tested and verifiable, with transparent unsigned warnings.

### M6 — Stable v1.0 consumer distribution (not automatic)

- [ ] Native signed/notarised Mac binaries, confirmed on fresh macOS; budget Apple programme membership or justified alternative delivery restriction.
- [ ] Windows signing (SignPath Foundation if approved or separate signing solution), verify real signature status; SmartScreen reputation may take time.
- [ ] Complete public audit/evidence, privacy, install FAQ, accessibility checks and security report.
- [ ] Update changelog and version tag; freeze and manually publish only after every release gate.

## Suggested ordered GitHub issue backlog

| ID | Title | Stage |
|---|---|---|
| PNK-001 | Bootstrap Tauri v2 desktop, brand, version command | M0 |
| PNK-002 | Rust workspace and core contract | M0 |
| PNK-003 | Protected CI and cross-platform blank installer builds | M0 |
| PNK-004 | Format sniffing and animated detection | M1 |
| PNK-005 | PNG lossy+lossless encoders with fixtures | M1 |
| PNK-006 | JPEG mozjpeg with orientation policy | M1 |
| PNK-007 | WebP lossless/lossy codec | M1 |
| PNK-008 | AVIF decoder/encoder packaging spike | M1 |
| PNK-009 | Golden conversion matrix tests | M1 |
| PNK-010 | Native dialogs and trusted path registration | M2 |
| PNK-011 | Nested folder enumeration without traversal | M2 |
| PNK-012 | Bounded job queue + cancellation + IPC events | M2 |
| PNK-013 | Atomic output + conflict + larger-file policy | M2 |
| PNK-014 | Smart Auto and website presets | M2 |
| PNK-015 | Responsive drop zone and accessible controls | M3 |
| PNK-016 | Real progress list and batch summaries | M3 |
| PNK-017 | Pancake loading animation / reduced motion | M3 |
| PNK-018 | Preview comparison and error UX | M3 |
| PNK-019 | Threat model tests, fuzzing, memory limits | M4 |
| PNK-020 | Privacy offline proof and audit reports | M4 |
| PNK-021 | Licence compliance, SBOM, notices | M4 |
| PNK-022 | Cross-platform release matrix + checksums | M5 |
| PNK-023 | GitHub attestations + security documentation | M5 |
| PNK-024 | Manual installer QA Windows x64 | M5 |
| PNK-025 | Manual installer QA Mac ARM and Intel | M5 |
| PNK-026 | Mac notarisation / Windows signing acceptance | M6 |
| PNK-027 | Public release docs and accessibility sign-off | M6 |

## Codex reporting contract

After each stage, produce exactly: changed files; commands and real results; OS/arch actually exercised; missing evidence; security/quality impact; unresolved blockers; next scoped stage. **Never mark a requirement done merely because it is documented.**

## v3 cross-cutting work: required in the original milestone order

These are **additional acceptance criteria**, not a request to skip codec or security milestones.

- **M0 (public experience baseline):** make README downloader-first and honest about no binaries; adopt both `AGENTS.md` and `CLAUDE.md`, source comment standards, support/accessibility/governance docs, and GitHub issue forms. Create a reproducible asset export task; do not make final logo without design sign-off.
- **M1–M2 (explain the engine):** document each module, codec conversion and failure path in Rustdoc; update `ARCHITECTURE-FOR-CONTRIBUTORS.md` to reference actual source files; explain memory/queue/file-safety invariants with accurate comments and tests.
- **M3 (brand/UX):** explore 3 original logo concepts, select/test 16px mark, produce local SVG/native icon source, implement a sleek single-window interface, light/dark, reduced motion, responsive progress and 1,000-row stress test. Record screenshots and keyboard/screen-reader results.
- **M4 (security publication):** publish measured privacy/security checks in plain language with dates and OS/build scope; disclose vulnerabilities through private reports, not public issues.
- **M5–M6 (download + community):** populate verified actual release links/manifest in downloader-first README; test install guides on each supported architecture; GitHub Discussions + categories and vulnerability-reporting configuration (manual repository settings); ensure screenshot/review UX, no unverifiable trust claims.

### New PNK issues to create
PNK-036 downloader-first public README/install/FAQ; PNK-037 discussion and feedback channel; PNK-038 source comments and architecture tour; PNK-039 brand explorations/SVG native icons; PNK-040 light/dark/reduced motion quality; PNK-041 large batch responsiveness tests; PNK-042 public trust and signing evidence; PNK-043 documentation correctness and onboarding usability walkthrough.
