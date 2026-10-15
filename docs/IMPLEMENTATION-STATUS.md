# Implementation status — evidence ledger

Updated: 2026-10-13. **This handoff is specifications/workflow templates only. No application code, installers, compiled artefacts, dependency audits or manual installation tests have been run.**

| Area | Status | Evidence / blocker |
|---|---|---|
| Product requirements and coding instructions | Prepared | `docs/00`–`docs/20` |
| Privacy/security/governance document templates | Prepared | `SECURITY.md`, `PRIVACY.md`, `CONTRIBUTING.md` |
| GitHub Actions workflow templates | Prepared, unrun | `.github/workflows/` — requires real app scaffold |
| Rust codec project and image support | Not implemented | Codex M0 and M1 |
| React/Tauri UI and animation | Not implemented | Codex M0/M3 |
| Windows x64 NSIS build | Not built or tested | Codex M5 |
| Mac Apple Silicon DMG | Not built or tested | Codex M5 |
| Mac Intel DMG | Not built or tested | Codex M5 |
| GitHub attestations/checksums | Not generated | Only source CI workflow/scripts prepared |
| Windows signing | Not provisioned | Optional SignPath Foundation application/other |
| Apple notarisation | Not provisioned | Paid Developer ID programme and CI secrets required |
| Manual clean OS installation | Not tested | Needs devices/VMs |

Add timestamped dated rows with PR/commit and CI run links, reproducible command output, OS version, and reviewer for each feature. Don't overwrite historic negative results.

## v3 preparation update — 2026-10-15

Documentation templates for downloader-first GitHub, dual coding agents, logo/design system, maintainable comments, public trust and feedback were added. **No GitHub settings, application source implementation, live logo design approval, release binaries, runtime security checks or installer smoke tests were performed by this documentation update.**

## M0 local evidence — 2026-10-15 (Claude Code, branch `m0-bootstrap`, macOS 26.1 arm64)

Toolchain recorded: Node 24.15.0, npm 11.12.1, Rust 1.99.1 (rustup, pinned in `rust-toolchain.toml`), Xcode at `/Applications/Xcode.app`, Tauri 2.12.4, create-tauri-app 4.8.0.

| Area | Status | Evidence |
|---|---|---|
| Tauri v2 + React/TS/Vite scaffold, Rust workspace, `pancake-core` crate | Implemented | Official `create-tauri-app` react-ts template, merged and reduced (ADR-020) |
| `get_app_info` IPC command | Implemented, unit-tested | `src-tauri/src/commands.rs`; 2 tests incl. camelCase contract |
| Front-end checks | Passed locally | `npm run lint` (0 warnings), `npm run test` (3/3), `npm run build` (JS 222 kB / 69.7 kB gzip, CSS 11.8 kB) |
| Rust checks | Passed locally | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked` (8 unit/integration + 1 doctest) |
| Contrast of colour tokens | Measured | All text pairs ≥ 4.5:1 after ADR-018; values recorded in `src/styles/tokens.css` |
| macOS arm64 DMG | Built locally, **not install-tested on a clean machine** | `npm run tauri build -- --target aarch64-apple-darwin --bundles dmg` with `APPLE_SIGNING_IDENTITY=-`. DMG 1,902,351 bytes, SHA256 `b55ce1c54c22dc1f80b14f51916d01e2ce8f6574fe1c6ddfed376dceabc6f292` (local build, not a release). App 3.8 MB, arm64 only, `LSMinimumSystemVersion` 13.0 |
| DMG behaviour | Checked | Mounts without a licence prompt (after ADR-023), contains app + Applications link |
| Bundle identifier | Changed after this build | The DMG above was built before the bundle identifier was finalised (ADR-016), so CI builds supersede it |
| Signature | Ad-hoc only | `codesign --verify --deep --strict` passes; `Signature=adhoc`, hardened runtime flag set. `spctl --assess` **rejects** it: Gatekeeper will block downloaded copies. Not notarised |
| Launch smoke test (build machine, not clean OS) | Passed | Process started, main window 960×640 on screen within ~0.75 s of `open`, still running after 6 s at ~100 MB RSS. `lsof -i` showed 0 sockets for the app process. WebKit helper processes were **not** inspected, so this is not yet a full no-network proof |
| Screenshot | Not captured | Terminal lacks macOS Screen Recording permission |
| Windows x64 NSIS, macOS Intel DMG | **Not built** | Needs GitHub Actions run (blocked on push permission for workflow files) |
| GitHub CI on 3 runners | **Not run** | Moving to `matthewnel87/pancake-compressor`; push pending GitHub CLI login for that account |
| Dependency audits (`npm audit`, `cargo audit`), licence check | `npm install` reported 0 vulnerabilities; `cargo audit` and licence tree **not run** | CI `audit` job, and cargo-deny still to be added |

**Gate M0 is not yet met**: it requires green Windows and macOS (Intel + ARM) CI bundles.

## M1 step 1: format sniffing and preflight — 2026-10-15 (branch `m1-format-sniffing`, stacked on `m0-bootstrap`)

| Area | Status | Evidence |
|---|---|---|
| Magic-byte detection (PNG, JPEG, WebP, AVIF) and named unsupported formats (GIF, BMP, TIFF, HEIF, JPEG XL, uncommon JPEG processes) | Implemented, unit-tested | `crates/pancake-core/src/format.rs`; `cargo test --workspace --locked` on macOS arm64: 22 engine tests + 3 doctests pass |
| Header dimensions and alpha hint | Implemented, unit-tested | Synthetic header fixtures per format (`fixtures/README.md`) |
| Animated input refusal (APNG, animated WebP, AVIF sequence) | Implemented, unit-tested | `animated_inputs_are_refused_not_flattened` |
| Decompression-bomb refusal before decode | Implemented, unit-tested | `preflight_refuses_decompression_bombs_before_decoding` (ADR-024) |
| Robustness against hostile headers | Tested, **not fuzzed** | Every truncation of every fixture and 20,000 seeded random inputs return without panic. Real fuzzing (cargo-fuzz) is M4 work |
| Checked against real files from other encoders | **Not done** | Fixtures are hand-built headers; real encoder output is checked once codecs land |
| Windows / macOS Intel runs of these tests | **Not run** | Will run in CI when this branch has a PR |

## M1 step 2: PNG, JPEG and WebP decoding — 2026-10-15 (branch `m1-decode`, stacked on `m1-format-sniffing`)

| Area | Status | Evidence |
|---|---|---|
| PNG decode (8/16-bit, alpha) | Implemented, tested | Pixel-exact round trip incl. transparent and half-transparent pixels; 16-bit depth kept |
| WebP decode (lossless, alpha) | Implemented, tested | Pixel-exact round trip. **Lossy WebP decode not yet tested** (needs a lossy encoder, which arrives with the libwebp step) |
| JPEG decode (baseline) | Implemented, tested | Mean absolute error < 4 on a smooth gradient. Progressive and CMYK JPEG, and EXIF orientation from real camera files, **not yet tested** |
| Decoder memory limits from header | Implemented, tested | `caller_limits_are_enforced_before_decoding`; decoder `max_alloc` = pixels x 8 bytes + 64 MiB |
| Damaged data | Tested | Truncations every 7 bytes and half-zeroed files for all three formats return errors, never panic; edited IHDR is refused |
| AVIF decode | **Not implemented** | Returns `format-not-yet-supported` |
| Results on Windows / macOS Intel | **Not run yet** | macOS arm64 only: `cargo test --workspace --locked` 32 engine tests + 3 doctests pass; clippy and `cargo deny check` clean |

## M1 step 3: AVIF decoding — 2026-10-15 (branch `m1-avif-decode`, stacked on `m1-decode`)

| Area | Status | Evidence |
|---|---|---|
| AVIF decode (8-bit with alpha, 10-bit) | Implemented, tested on macOS arm64 | Real files from `ravif`: 8-bit RGBA mean error < 6 and transparent area stays transparent; 10-bit mid-grey decodes as full-range 16-bit (~32,896), not dark |
| AV1 frame larger than container | Refused before AV1 decoding, tested | `avif_frame_larger_than_container_is_refused` (ADR-027) |
| Rotation/mirroring (`irot`/`imir`) | Refused with a clear message, tested | `rotated_or_mirrored_avif_is_refused_until_supported` |
| Damaged AVIF | Tested in debug **and release** builds | Truncations every 5 bytes and half-zeroed file return errors. Found and fixed: `avif-parse` panicked in debug builds on a truncated file (ADR-028) |
| Header inspector vs real encoder output | Checked for AVIF | `format::inspect` reads the correct 48×32 size from a `ravif` file |
| Embedded ICC profile in AVIF | **Not returned** (always `None`) | Wide-gamut AVIFs could shift colour when converted; to be handled in PNK-013 metadata work |
| AVIF grid (tiled) images, 12-bit, monochrome | **Not tested** | Need fixtures from another encoder (e.g. libavif) |
| Windows x64 / macOS Intel build (needs NASM) | **Not yet proven** | This PR's CI is the first run with NASM installed |
| Checks on macOS arm64 | Passed | `cargo test` (36 engine tests + 3 doctests) in debug and `--release`; clippy clean; `cargo deny check` clean |

## M1 step 4: PNG and JPEG encoding — 2026-10-15 (branch `m1-png-encode`, stacked on `m1-avif-decode`)

| Area | Status | Evidence |
|---|---|---|
| Lossless PNG (oxipng) | Implemented, tested | Pixel-identical incl. hidden RGB of fully transparent pixels; 16-bit precision kept; `tEXt` metadata kept; output verified by re-decoding |
| Lossy PNG (imagequant) | Implemented, tested | Flat-colour image becomes indexed PNG with exact colours and alpha; 16-bit source reduced to 8-bit palette; ICC profile carried |
| Quality floor | Tested | Noise at min quality 95 returns `quality-not-reached` instead of a banded image |
| EXIF/XMP in lossy PNG output | **Not carried yet** | PNK-013 metadata matrix |
| C2PA-protected PNG | Mapped to `protected-content`, **not tested** with a real signed file |
| Size/time benchmarks | **Not measured** | M1 exit criterion; needs representative fixtures |
| Checks on macOS arm64 | Passed | 44 engine tests + 3 doctests; clippy clean; `cargo deny check` clean |
| JPEG (MozJPEG) | Implemented, tested | Valid progressive JPEG; quality 40 smaller than 95; out-of-range quality rejected |
| JPEG transparency | Tested | Transparent area becomes white (default) or black (custom matte); `transparency_flattened` reported |
| JPEG orientation | Tested | Rotate90 source becomes a 20×40 upright output with no EXIF block; `orientation_applied` reported. **Not yet tested with real camera files** |
| JPEG ICC profile | Tested | 140 kB profile split across APP2 segments and read back identical; oversize profile refused, not dropped |
| JPEG greyscale / 16-bit | Tested | Grey stays 1-channel; 16-bit reports `precision_reduced` |
| Release panic strategy | Changed to unwind (ADR-030) | Required for MozJPEG error handling |
| Checks on macOS arm64 | Passed | 54 engine tests + 3 doctests; clippy clean; `cargo deny check` clean (IJG allowed) |
| WebP lossless (libwebp) | Implemented, tested | Pixel-identical incl. hidden RGB under transparent pixels |
| WebP lossy | Implemented, tested | Decodes (now also covers **lossy WebP decoding**); smaller than lossless on a textured image; transparency kept |
| WebP ICC | Tested | Survives simple lossy (VP8), simple lossless (VP8L) and extended (VP8X+ALPH) layouts, odd-length profile, RIFF size consistent |
| Observation | Recorded | On a perfectly smooth synthetic gradient, lossless WebP was smaller than lossy. The M2 "avoid larger output" comparison must compare real outputs, not assume |
| Checks on macOS arm64 | Passed | 61 engine tests + 3 doctests; clippy clean; `cargo deny check` clean |
| AVIF encode (ravif) | Implemented, tested | Decodes back at correct size; quality 30 smaller than 95; alpha kept; orientation applied |
| AVIF and colour profiles | Tested | Source with ICC refused by default (`colour-profile-not-supported`); allowed loss reported as `colour_profile_dropped` |
| Checks on macOS arm64 | Passed | 66 engine tests + 3 doctests; clippy clean; `cargo deny check` clean |

## M1 format matrix and first benchmark — 2026-10-15

| Area | Status | Evidence |
|---|---|---|
| Every input format → every output mode | Tested on macOS arm64 | `tests/format_matrix.rs`: 5 inputs × 6 output modes; table in docs/07 "M1 implemented matrix" |
| Benchmark (opt-in) | Measured once, macOS arm64 only | `cargo test -p pancake-core --release --test benchmark -- --ignored --nocapture` on Apple M1 Pro, 16 GB, macOS 26.1 |

Synthetic 4000×3000 input (gradients + grain + hard edges), PNG source 19,871,605 bytes. Times include re-decoding for verification. **Synthetic content: not a prediction of savings on real photos.**

| Encoder (defaults) | Output bytes | % of PNG | Time |
|---|---|---|---|
| PNG lossless, effort 2 | 9,319,068 | 46.9% | 8.72 s |
| PNG palette | 2,861,210 | 14.4% | 8.62 s |
| JPEG q80 4:2:0 | 377,833 | 1.9% | 0.56 s |
| WebP lossy q80 | 344,380 | 1.7% | 1.00 s |
| WebP lossless | 7,865,000 | 39.6% | 7.86 s |
| AVIF q70 speed 6, 1 thread | 51,645 | 0.3% | 6.98 s |

Peak memory for the whole run: **1.01 GB** (`/usr/bin/time -l`). For M2 this means roughly 1 GB per concurrent 12 MP job until the extra pixel copies (orientation clone, RGBA conversions, verification decode) are reduced. **Windows and Intel numbers not measured yet.**

## M1 step 5: metadata (PNK-013) — 2026-10-15 (branch `m1-metadata`, stacked on `m1-encoders`)

| Area | Status | Evidence |
|---|---|---|
| EXIF/XMP read on decode (PNG, JPEG, WebP) | Implemented | `DecodedImage.exif` / `.xmp` |
| EXIF orientation reset | Implemented, tested | Both byte orders; every truncation; non-standard layouts refused (`metadata.rs` tests + doctest) |
| Carriage per format | Tested | `tests/metadata_round_trip.rs`: JPEG, WebP lossy/lossless, PNG palette keep EXIF+XMP; AVIF keeps EXIF, reports XMP dropped; all outputs upright, re-read EXIF says upright |
| Unreadable EXIF | Tested | Not copied, reported as dropped |
| **Bug fixed** | PNG palette ignored EXIF orientation | Sideways camera images would have stayed sideways with orientation lost. Now rotated and reported like the other encoders |
| Not done | | EXIF/XMP in AVIF **inputs** not read; PNG metadata after IDAT not read; extended XMP for JPEG; IPTC not carried; privacy-first strip option (M2/M3) |
| Checks on macOS arm64 | Passed | 70 unit + matrix + 3 metadata + 2 API tests + 4 doctests; clippy clean; `cargo deny check` clean |

## M1 step 6: presets and Smart Auto (PNK-014) — 2026-10-15 (same branch)

| Area | Status | Evidence |
|---|---|---|
| Presets (Balanced, Smaller, Best quality, Lossless) | Implemented, tested | `auto::tests` |
| Smart Auto | Implemented, tested | Photo → WebP/JPEG, deterministic across runs; transparent graphic → PNG/WebP with alpha kept, never JPEG |
| Never larger (ADR-015) | Tested | Quality-20 JPEG re-encoded at Best quality keeps the original (`original-already-smaller`) |
| Lossless preset | Tested | PNG photo → pixel-identical; JPEG keep → original kept (`no-suitable-mode`) |
| AVIF consent | Tested | Profile loss needs consent; AVIF in Auto only when enabled |
| Preset numbers | **Provisional** | Not tuned on real photographs yet |
| Checks on macOS arm64 | Passed | 78 unit + matrix + 3 metadata + 2 API tests + 4 doctests; clippy clean |

## M2 engine core — 2026-10-15 (branch `m2-batch-engine`, stacked on `m1-metadata-auto`; local)

| Area | Status | Evidence |
|---|---|---|
| Safe output writing | Implemented, tested | Never overwrites (numbered or skip); 50 threads writing one name → 50 distinct complete files, no temporaries left; unsafe relative paths and device names refused; write failure leaves nothing |
| Scanner | Implemented, tested (symlink tests Unix-only) | Nested relative paths; output and `_pancake` folders skipped; overlapping/repeated roots listed once; OS clutter ignored; depth limit; symlinks to outside folders, loops and file links not followed; link roots refused; FIFOs skipped. **Windows junction test not written** (needs elevated rights to create) |
| Batch engine | Implemented, tested | Mixed batch (2 images, text, GIF, damaged PNG, APNG) → exact codes and counts; 500 nested files reconcile with mirrored output paths; originals byte-identical; existing outputs kept; cancel before start → 0 files written; cancel mid-batch → finished outputs complete, no temporaries, counts reconcile; memory-budget skip before decoding; peak concurrency ≤ worker count; unusable output folder fails every file cleanly |
| Not done in M2 yet | | Tauri commands, native file/folder pickers, event bridge to the UI, batch snapshot for reconnect, disk-full simulation, Windows case-collision test, huge-dimension PNG test in the batch corpus |
| Checks on macOS arm64 | Passed | 105 unit + matrix + 3 metadata + 2 API tests + 4 doctests; clippy clean; `cargo deny check` clean |
