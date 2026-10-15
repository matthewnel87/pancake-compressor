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
