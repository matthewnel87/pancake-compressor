# 04 — Stack, codec candidates and dependency licence gates

## Free technical stack

| Component | Candidate | Role / gate |
|---|---|---|
| Tauri | v2 | Desktop, IPC, OS packaging; disable unused capabilities |
| UI | React/TypeScript + Vite + Tailwind | Accessible interface, local bundled resources |
| Images | Rust `image` crate | Still-image decoding with explicit features; verify AVIF decoder |
| PNG lossless | `oxipng` | Lossless optimiser; validate preservation rules |
| PNG lossy | `imagequant` + `lodepng` | Indexed palette with dithering; GPL policy |
| JPEG | `mozjpeg` Rust binding | Quality-focused recompression; validate builds + failure handling |
| WebP | Rust `webp` / libwebp | Lossy/lossless; verify alpha and ICC support |
| AVIF encode | `ravif` (rav1e) | Native AVIF still-image output |
| AVIF decode | `image` `avif-native` / dav1d (candidate) | Native dependency; cross-platform spike |
| Parallel work | `rayon` + custom resource scheduler | Not just unrestricted `par_iter()` |
| File enumeration | `walkdir` | No symlink following; safety policies |
| CI | GitHub Actions | Standard hosted runners public repository |
| Audits | `cargo-audit`, `cargo-deny`, `npm audit` | Security + licences; generate SBOM later |

## Required proof before adopting a codec

For each codec and each of Windows x64/macOS Intel/macOS ARM:

1. Compile from a fresh checkout using recorded Rust/node/OS tools.
2. Round-trip fixtures (photo, flat logo, transparent PNG, gradients, large dimensions, EXIF rotate, embedded ICC, corrupt/truncated input).
3. Test **actual** codec encoding/decoding support, not just extension handling.
4. Record licence, transitive dependencies, native library linking, binary size impact, memory peak and average throughput.
5. Ensure outputs render in Chrome, Safari and Firefox where the format should be supported.
6. Fail closed if metadata/ICC semantics cannot be supported as specified; expose limitations rather than making fidelity claims.

## Licence plan

- Initial project target: **GPL-3.0-or-later**, reflecting anticipated `imagequant` GPL compatibility. Included `LICENSE` is GPLv3 text; `-or-later` permission is specified in project notices/SPDX and must be confirmed for all contributions.
- Before first public release run licence checks for **every** Rust and npm direct/transitive dependency; review both dynamic and static linking and third-party notice obligations.
- Beware confusing library source licence with a wrapper's licence or optional/commercial licencing routes.
- CI dependency update bots must create reviewable PRs; do not auto-merge major upgrades or vulnerable codec replacements without tests.

## No bogus performance promises

Installer size, startup time, RAM, compression ratio, AVIF speed and memory usage are **not known** until measured on both OS families. Do not reuse the older brainstorm's claimed 10–15 MB installer, sub-20 MB RAM or 1 GB file guarantees.

## M0 licence audit — 2026-10-15

Measured with `cargo deny list -l crate` (cargo-deny 0.20.2) on the M0 lockfile: 381 crates in the Rust tree (Tauri 2.12.4 shell, no codecs yet).

| Licence expression | Crates |
|---|---|
| MIT and/or Apache-2.0 (incl. dual and triple licences) | 341 |
| Unicode-3.0 | 18 (ICU data crates) |
| MPL-2.0 | 5 (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`) |
| Unlicense/MIT, BSD-3-Clause, Zlib, CC0-1.0/MIT-0, 0BSD | 15 |
| GPL-3.0-or-later | 2 (this project) |

All are compatible with distributing the app as GPL-3.0-or-later. MPL-2.0 is file-level copyleft: those crates' own files stay under MPL-2.0, and MPL section 3.3 permits combining them into a GPL work. Two crates offer LGPL-2.1-or-later only as one alternative alongside MIT/Apache-2.0; we use them under MIT/Apache-2.0.

The allow-list is enforced by `deny.toml` (`cargo deny check`) in the CI `audit` job. **Not yet done:** npm licence tree for the bundled front end, third-party notices file in the installers, and re-auditing when codec crates arrive in M1 (`imagequant` is GPL-3.0-or-later, which is why ADR-005 chose GPL).

### M1 decoding dependencies — 2026-10-15

`image` 0.25.10 with `default-features = false` and only `png`, `jpeg`, `webp` enabled (MIT OR Apache-2.0). This pulls in the pure-Rust `png`, `zune-jpeg` and `image-webp` decoders and adds 22 crates to the engine, with no C code or build tools. `cargo deny check` passes. The `image` crate's `avif` (ravif) and `avif-native` (dav1d, a C library) features are deliberately **off**; AVIF is handled in its own step.

### M1 AVIF decoding dependencies — 2026-10-15

`avif-decode` 3.0.0 (BSD-3-Clause, default features off), which uses `avif-parse` 2.1.0 (MPL-2.0), `rav1d` 1.1.0 (BSD-2-Clause, Rust port of dav1d) and `yuv` 0.8 for colour conversion; plus `imgref` and `bytemuck` for buffer conversion. `ravif` 0.13.0 (BSD-3-Clause, default features off) is a **dev-dependency only**, used to generate AVIF test fixtures.

- **BSD-2-Clause** added to `deny.toml` (permissive, GPL-compatible).
- **RUSTSEC-2024-0436** (`paste` unmaintained) is ignored with a reason: `paste` is a compile-time proc-macro used by `rav1d` and `rav1e`, so none of its code runs in the app. Remove when those crates move to a maintained fork.
- **Build tool:** on x86 targets `avif-decode` enables `rav1d`'s assembly, which needs **NASM ≥ 2.14** at build time. Apple Silicon builds use pure Rust and need nothing extra.
