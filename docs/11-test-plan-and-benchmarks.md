# 11 — Verification, quality gates and benchmarking

## Test pyramid

**Unit (Rust core):** signature detection, dimension/pixel cap, quality preset mapping, file extension/path normalisation, collision naming, output byte accounting, scheduler memory permits, timestamp/event order, cancelled job state.

**Integration (Rust core):** codec encode/decode round-trips by matrix; mock compressed outputs; bad/malformed/truncated file handling; temporary file clean-up; alpha/ICC/EXIF policies; ensure no silent animation frame loss.

**Frontend (Vitest/React Testing Library):** format/preset controls, drag target, queue, progress ARIA semantics, empty/errors, cancellation, render >1,000 rows using virtualization, mock delayed event/out-of-order event recovery, reduced-motion.

**Platform E2E:** offline install → launch → import → compress → output folder → cancel → uninstall on Win11 x64, macOS ARM64 13+ and macOS Intel 13+ (test exact real versions and note hardware/VM). GitHub runners alone building DMG/EXE are insufficient evidence of installation success.

## Fixture matrix

| Fixture | Why it matters | Test |
|---|---|---|
| JPEG photograph 2K/4K | lossy encoders, orientation | dimensions, appearance, bytes |
| Transparent logo PNG | alpha fringing, palette quantisation | transparency and edges |
| Gradient UI PNG | banding | compare high/low preset |
| Pixel art icon PNG | sharp edges | lossless and palette mode |
| EXIF orientation JPEG | double rotation | visually correct output |
| Embedded ICC | colour correctness | profile preserve/warning |
| Still WebP with alpha | decoder and converter | alpha + valid output |
| Still AVIF, 8-bit | native decoding | Mac Intel/ARM + Windows |
| Animated WebP/AVIF | safety | reject or explicitly handle, never lose frames silently |
| Corrupt/truncated samples | parser resilience | safe per-file failure |
| Huge width/height header | decompression bomb | reject pre-decode |
| Unicode/emoji/space pathname | platform FS correctness | expected export paths |
| Output collision/symlink/path escape | data integrity | rejection/rename |
| 100/1000-file collection | queue stability | bounded memory and responsiveness |

Fixtures must be synthetically generated or under documented permissive licence. Avoid shipping private test images. Start with small samples and explicitly add high-memory fixtures on appropriate runners, not standard 7 GB macOS ARM runner by default.

## Gate table

| Gate | PR | Release candidate | Stable release |
|---|---|---|---|
| TypeScript typecheck, unit tests, lint, build | Required | Required | Required |
| `cargo fmt`, clippy, unit/integration | Required | Required | Required |
| Rust/JS vulnerability and licence audits | Required (warnings investigated) | Required | Required |
| macOS ARM + Intel / Win x64 package build | Recommended PR or nightly | Required | Required |
| Offline functionality smoke on 3 OS/arch | — | Required | Required |
| Installer/uninstaller smoke | — | Required | Required |
| Corrupt-file/path security scenarios | Required for changed paths | Required | Required |
| Accessible keyboard + reduced-motion audit | UI changes | Required | Required |
| Hash and build attestation per binary | — | Required | Required |
| Sign/notarise if status says so | — | Required if claimed | Required if claimed |

## Benchmark methodology

Pin machine/OS, toolchain, hardware CPU/RAM, image test set and codec preset. Record original/output bytes, median duration over 3–5 runs, max RSS, compression ratio, CPU peak and visual quality metrics/qualitative review. Compare only same input and visual target against TinyPNG, Caesium, ImageOptim or Squoosh. Publish a CSV and exact commands in releases, do not claim 'fastest' or 'tinyPNG quality' without data.

## Developer QA checklist

- Check crashes and OS dialog permissions; verify output opens in common browsers/image viewers.
- Disable internet (and observe outgoing connections) while processing; check WebView resources are bundled locally.
- Confirm `0 files` never displays `NaN%`; cancellations can never double-count.
- Check disk full mid-batch; sleep/wake; multiple rapid clicks; drag nested folders; safe-window close while jobs run.
- When tests weren't run, record that explicitly; no invented green checks.
