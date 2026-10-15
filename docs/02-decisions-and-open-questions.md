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
| ADR-016 | Bundle identifier `io.github.matthewnel87.pancakecompressor` | `io.github.<maintainer>` placeholder; a custom domain | Uses the maintainer's GitHub namespace (`matthewnel87`), which the maintainer controls. Changing it after release breaks macOS preferences and Windows upgrade paths, so confirm before the first public preview. | Provisional, maintainer to confirm |
| ADR-017 | Pin Node 24 LTS in `.nvmrc` and Rust 1.99.1 in `rust-toolchain.toml`; CI reads Node from `.nvmrc` and installs Rust 1.99.1 explicitly | Keep CI on `node-version: '22'` and `rust-toolchain@stable` | Node 24 is the active LTS and is required by the current Vite 8 / ESLint 10 toolchain range; local development already ran on 24.15.0. Floating `stable` would make CI differ from local builds. Bump both deliberately in their own PRs. | Final for M0 |
| ADR-018 | Light-mode `--accent` changed from `#AD6025` to `#A65B22` | Keep the proposal; use accent only for large text | `#AD6025` measured 4.48:1 on `--surface` (fails 4.5:1 for normal text). `#A65B22` measures 4.84:1, and white text on it 5.07:1. Visually near-identical. | Final |
| ADR-019 | Stage installers in `release-artifacts/`, not `dist/` | Change Vite `outDir` instead | Vite builds the web front end into `dist/`, so CI would have uploaded web assets alongside the installer and `verify_staged.py` would always fail. Tauri's `frontendDist` keeps its default. | Final |
| ADR-020 | Remove the scaffold's `opener` plugin; window capability limited to `core:app:default`, `core:event:default`, `core:window:default` | Keep `core:default` plus `opener:default` | The opener plugin can launch URLs and files, which M0 does not need (AGENTS.md rule 5). Narrow capability set reduces what a compromised webview could do. | Final for M0; revisit when the dialog plugin is added in M2 |
| ADR-021 | Windows NSIS `installMode: currentUser`; macOS `minimumSystemVersion: 13.0` | Per-machine install (needs admin) | Per-user install avoids an admin prompt and matches docs/13. macOS 13 matches ADR-008. Both still need real install tests. | Provisional pending install tests |
| ADR-022 | `pancake-core` forbids `unsafe` and denies `unwrap`/`expect`/`panic!` via crate lints; privacy flag enforced by a compile-time assertion | Rely on review only | Turns AGENTS.md rules 2 and 4 into build failures instead of review comments. | Final |
| ADR-023 | Do not set `bundle.licenseFile` | Show the GPL as an installer/DMG licence page | With `licenseFile`, the DMG refused to open until the user clicked "Agree", and NSIS adds a licence page. The GPL does not require acceptance to run the program, so a click-through misrepresents it and adds friction. The licence ships in the source and stays declared via `bundle.license`. Third-party notices bundle is still required before release (docs/04). | Final |
| ADR-024 | Pre-decode caps: 65,535 px per side and 2^28 pixels (about 268 MP, 1 GiB as RGBA8) via `ImageLimits::DEFAULT`; checked by `format::preflight` from header dimensions only | No cap (rely on decoder errors); 100 MP cap; per-machine cap only | Decoders allocate from declared dimensions, so a ~60-byte file can demand tens of GB. A fixed cap stops that before decoding. The value is far above camera photos but within one decode on an 8 GB machine. The per-batch memory budget is separate (M2). Revisit with M2 benchmarks. | Provisional (2026-10-15) |
| ADR-025 | Animated inputs refused at header level: APNG (`acTL`), WebP with VP8X animation flag, AVIF with `avis` brand (even if a still cover image exists). GIF, HEIC/HEIF, TIFF, BMP, JPEG XL and lossless/hierarchical/arithmetic JPEG reported as unsupported | Decode first frame with a warning | docs/07 forbids silent frame loss. Refusing early is predictable and needs no decoder. Re-evaluate GIF/HEIF support after v0.1. | Final for v0.1 (2026-10-15) |
| ADR-026 | For AVIF, `inspect` reports the **largest** `ispe` size in the file rather than resolving the primary item | Parse `pitm` + `ipma` to find the primary item's size | Largest size is the safe bound for memory caps and needs far less parsing of untrusted structures. The decoder reports the true primary size later. | Final for M1 (2026-10-15) |
| ADR-027 | AVIF decoding with `avif-decode` 3.0 (`avif-parse` + `rav1d`, a Rust port of dav1d). Before decoding, the AV1 sequence header's max frame size (colour and alpha) must fit inside the preflight-capped container size. AVIFs with `irot`/`imir` are refused. x86 builds need NASM ≥ 2.14 (CI installs it with `ilammy/setup-nasm`) | `image` crate `avif-native` (C dav1d via pkg-config, hard to build on Windows); write an AV1 decoder binding ourselves | Pure Rust apart from optional assembly, so it builds on all three targets without system libraries. `avif-decode` sets no frame-size limit, so our pre-check closes that gap. It does not apply rotation/mirroring, so refusing is safer than producing a sideways copy. NASM is a build-time tool only. It uses one thread per CPU, which the M2 scheduler must account for | Provisional until CI proves Windows and Intel builds (2026-10-15) |
| ADR-028 | Disable debug assertions for `avif-parse` in dev/test builds (`[profile.dev.package.avif-parse]`) | Leave as is; patch the crate | `avif-parse` 2.1.0 `debug_assert!`s a condition hostile files can trigger, then returns an error in release. With assertions on, a damaged AVIF crashed debug builds and tests. Matching release behaviour keeps tests honest. Report upstream | Final (2026-10-15) |
| ADR-029 | PNG output: lossless via `oxipng` 10.2 (default features off; `optimize_alpha` and `scale_16` off; all chunks kept); lossy via `imagequant` 4.4 → indexed PNG (`png` crate) → `oxipng`. Every output is decoded again and checked (identical pixels for lossless; same size and kept transparency for lossy) before it is returned | oxipng defaults incl. zopfli and rayon; `optimize_alpha` on for extra savings | Pixel-exact by construction and verified. No hidden thread pool (the M2 scheduler controls parallelism). Rewriting the colour of fully transparent pixels is a real (if invisible) pixel change, so it is off. Lossy quality floor returns `quality-not-reached` instead of banded output. oxipng refuses to modify C2PA-signed PNGs; surfaced as `protected-content` | Final for M1 (2026-10-15) |
| ADR-030 | Release profile `panic = "unwind"` (was `abort`) | Keep abort; isolate MozJPEG in a subprocess | MozJPEG reports libjpeg errors by unwinding a panic and its docs require `catch_unwind` around every call, which cannot work with abort. Unwinding also lets the batch engine (M2) turn a panic in any codec into one failed file instead of a crashed app (AGENTS.md rule 3). Our own code still may not panic on input (crate lints). Cost: slightly larger binary | Final (2026-10-15) |
| ADR-031 | JPEG output via MozJPEG (`mozjpeg` 0.10.13 / `mozjpeg-sys` 2.2.3): progressive, optimised Huffman, quality 1-100, 4:2:0 or 4:4:4. EXIF orientation is applied to pixels and no orientation tag is written; transparency is composited onto a matte (white default) and reported; ICC profile written as APP2 segments; a profile too large for JPEG is refused, never dropped | Keep EXIF orientation tag and leave pixels; libjpeg-turbo via `image` crate encoder | Applying orientation and writing no tag cannot double-rotate in any viewer. MozJPEG gives markedly smaller files than the `image` crate encoder. `IJG` licence allowed (GPL-compatible) with its documentation attribution obligation | Final for M1 (2026-10-15) |

