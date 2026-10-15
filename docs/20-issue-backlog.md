# 20 — GitHub-ready engineering issue backlog

Each issue is independently reviewable with acceptance tests; label tasks by milestone and attach an actual PR/CI URL upon execution.

| ID | Milestone | Title | Dependencies | Completion test |
|---|---|---|---|---|
| PNK-001 | M0 | Scaffold Tauri v2 React TS app | — | `npm run tauri dev` opens on macOS/Windows |
| PNK-002 | M0 | Create Cargo workspace and pancake-core | 001 | Cargo workspace fmt/test/clippy pass |
| PNK-003 | M0 | Pin deps / lockfiles / licence audit | 002 | npm/cargo lockfiles and licence decision logged |
| PNK-004 | M0 | Least-privilege Tauri settings and CSP | 001 | capabilities reviewed, no remote content |
| PNK-005 | M0 | CI for TS, Rust, security audits | 001–004 | PR checks actually pass |
| PNK-006 | M0 | Package bare app across Win/Mac | 005 | Win EXE + Mac ARM/Intel DMGs generated |
| PNK-007 | M1 | Create synthetic fixture corpus | 002 | provenance manifest and tests |
| PNK-008 | M1 | Signature/dimension/animation scanner | 007 | rejects malformed/huge/animated safely |
| PNK-009 | M1 | Lossless and lossy PNG codec paths | 007–008 | golden pixel/visual tests |
| PNK-010 | M1 | JPEG codec, EXIF rotation, matte | 007–008 | orientation/alpha conversion tests |
| PNK-011 | M1 | WebP still encode/decode | 007–008 | lossy/lossless and alpha tests |
| PNK-012 | M1 | AVIF still decode/encode cross-platform | 007–008 | target OS build+golden tests |
| PNK-013 | M1 | Metadata/ICC capability matrix + policy | 009–012 | fixture evidence, explicit warnings |
| PNK-014 | M1 | Smart Auto candidates and presets | 009–013 | deterministic per-fixture choice |
| PNK-015 | M2 | User-selected input/output registration | 008 | IPC cannot access arbitrary outside paths |
| PNK-016 | M2 | Output path security / no-clobber | 015 | symlink/traversal/collision tests |
| PNK-017 | M2 | Bounded memory scheduler | 009–014 | pixel bomb/100+ batch tests |
| PNK-018 | M2 | Cancel/cleanup and crash-safe exports | 016–017 | no original files changed |
| PNK-019 | M2 | Event model, snapshots, monotonic IDs | 017–018 | out-of-order/lost-event tests |
| PNK-020 | M3 | File/folder import + drop UX | 015,019 | keyboard and drag paths |
| PNK-021 | M3 | Presets/settings + output controls | 014,020 | correct Rust commands and validation |
| PNK-022 | M3 | Result queue, savings calculations | 019–021 | error/skip/success counts reconcile |
| PNK-023 | M3 | Animated pancake + real progress | 019–022 | reduced-motion and aria progress tests |
| PNK-024 | M3 | Scaled before/after previews | 009–014,022 | no large IPC buffers, correct colour |
| PNK-025 | M3 | Accessible keyboard and screen-reader audit | 020–024 | keyboard-only + assistive QA |
| PNK-026 | M4 | Security code review and fuzz corpus | 008–018 | tests run, vulnerabilities triaged |
| PNK-027 | M4 | Offline/no-network runtime verification | 020–025 | tested on Win/Mac with network blocked |
| PNK-028 | M4 | Reproducible benchmark report | 009–018 | fixtures, hardware, numbers published |
| PNK-029 | M5 | CI native release build and artifact names | 006,025–028 | exact 3 assets and hashes |
| PNK-030 | M5 | Attestation integrity verification | 029 | `gh attestation verify` on downloads |
| PNK-031 | M5 | OS install/uninstall smoke matrix | 029 | clean OS tests Win + two Mac variants |
| PNK-032 | M5 | Public technical preview and documentation | 030–031 | complete release notes and status |
| PNK-033 | M6 | Windows OSS code signing eligibility | 032 | accepted SignPath or alternative decision |
| PNK-034 | M6 | macOS Developer ID/notarisation | 032 | signed and notarised artifacts verified |
| PNK-035 | M6 | Stable release security and support review | 033–034 | stable gate evidence complete |

## Issue template

```md
### Goal
Why this matters to user and what changes.

### Dependencies
Referenced PNK issues and current milestone.

### Implementation scope
Exact interfaces/features/files expected.

### Acceptance criteria
- [ ] Required happy path
- [ ] Failure/cancellation or security path
- [ ] Automated test(s) added
- [ ] OS platform coverage noted
- [ ] User-facing documentation updated

### Proof
PR URL, CI run URL, test commands, manual device/OS, screenshots (synthetic content), known limitations.
```

Task management: M0 (001–006) before codec M1; do not mark a GitHub issue complete merely because CodeX wrote an initial code sketch.

## v3 additional public/brand tasks (not yet implemented)

| ID | Milestone | Title | Dependencies | Completion test |
|---|---|---|---|---|
| PNK-036 | M0→M5 | Public README, support, FAQ and OS install docs | 001, 029 | verified links and real screenshots only after builds |
| PNK-037 | M5 | GitHub Discussions and feedback forms | 032 | manual settings verified; feedback and private security routes separate |
| PNK-038 | M1→M4 | Rustdoc, Tauri contract and architecture tour | 002, 009–019 | reviewer can trace real image and export flow from code/comments |
| PNK-039 | M3 | Explore 3 original logos, deliver selected SVG + icons | 020 | licence provenance, 16px light/dark and native bundles tested |
| PNK-040 | M3 | Visual system, light/dark/reduced-motion | 020–025 | keyboard, 200% scaling, contrast and screenshots |
| PNK-041 | M3→M4 | Startup and 1,000-image responsiveness | 017–025 | reproducible measurement and hardware details |
| PNK-042 | M4→M6 | Public security evidence and release trust | 026–035 | hashes, attestation, signing truth and smoke-test records |
| PNK-043 | M5 | End-user GitHub download/support usability test | 029–032 | 3–5 people identify right download & report feedback |
