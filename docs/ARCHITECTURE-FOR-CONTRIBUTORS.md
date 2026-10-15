# How Pancake Compressor works — a developer tour

> The diagram below is the **target** architecture. As of M0 only the shell exists; see "Where the code lives today" for what is real.

## Where the code lives today (M0)

| Path | What it does |
|---|---|
| `crates/pancake-core/src/lib.rs` | Engine crate root: version, privacy constant with a compile-time check, crate-wide lints (no `unsafe`, no `unwrap`/`panic!`). |
| `crates/pancake-core/src/limits.rs` | Overflow-safe memory estimates and the pre-decode pixel caps (`ImageLimits`, ADR-024). |
| `crates/pancake-core/src/format.rs` | Header-only identification: magic bytes, declared dimensions, alpha hint, animation refusal (ADR-025) and `preflight`, which every file must pass before decoding. Tests and synthetic fixture builders are in `format/tests.rs`. |
| `crates/pancake-core/src/decode.rs` | Decodes PNG, JPEG and still WebP (pure Rust, `image` crate) after preflight, with decoder limits taken from the header; reports EXIF orientation and ICC profile without applying them. |
| `crates/pancake-core/src/decode/avif.rs` | AVIF decoding via `avif-decode`, with a pre-check that the AV1 stream cannot declare a bigger frame than the container (ADR-027). |
| `crates/pancake-core/src/encode.rs` | Shared encoder types (`Encoded`, `Adjustments`, errors) and helpers for orientation and metadata carriage. Every encoder re-decodes and checks its own output before returning it. |
| `crates/pancake-core/src/auto.rs` | Presets, keep-format and rule-based Smart Auto: builds a bounded candidate list, keeps the smallest valid output, never returns a file larger than the original (ADR-015, ADR-035). |
| `crates/pancake-core/src/scan.rs` | Lists files under selected folders without following links, skipping output folders, duplicates and OS clutter (ADR-037). |
| `crates/pancake-core/src/output.rs` | Writes results with an atomic no-overwrite rename from a flushed temporary file; numbered or skip on conflicts (ADR-036). |
| `crates/pancake-core/src/batch.rs` | Runs a batch with fixed workers and a memory budget, cancellation, per-file panic containment and ordered progress events (ADR-038). |
| `crates/pancake-core/src/metadata.rs` | Bounds-checked EXIF reader that resets orientation to upright (ADR-034). |
| `crates/pancake-core/src/encode/png.rs` | Lossless PNG (`oxipng`, pixel-identical) and lossy palette PNG (`imagequant`) (ADR-029). |
| `crates/pancake-core/src/encode/jpeg.rs` | JPEG via MozJPEG: applies EXIF orientation, flattens transparency onto a matte and reports both in `Adjustments`; carries ICC (ADR-031). |
| `crates/pancake-core/src/encode/webp.rs` | Still WebP via libwebp, lossy or lossless (exact), keeps alpha; embeds ICC with a small RIFF edit (ADR-032). |
| `crates/pancake-core/src/encode/avif.rs` | Still AVIF via ravif, lossy only, keeps alpha; refuses to drop an ICC profile unless explicitly allowed (ADR-033). |
| `src-tauri/src/lib.rs` | Builds the Tauri app and registers the only callable commands. |
| `src-tauri/src/commands.rs` | `get_app_info`, the single M0 IPC command (read-only, no file access). |
| `src-tauri/capabilities/main.json` | Built-in Tauri APIs the window may use (deliberately minimal). |
| `src-tauri/tauri.conf.json` | Window, Content Security Policy and installer settings. |
| `src/lib/ipc.ts` | Typed TypeScript mirror of the Rust commands. |
| `src/App.tsx` | Launch screen. |
| `src/styles/tokens.css` | Colour and spacing tokens, with the measured contrast record. |

No file selection or batch code exists yet. Header inspection, decoding for all four formats and Encoding for all four formats, scanning, safe output and the batch engine are done. Wiring them to the desktop app (file pickers, events) and the UI follow.

```text
React / TypeScript interface
        │  validated Tauri IPC commands and versioned events
        ▼
Tauri Rust desktop boundary: user-authorised paths and UI events
        │
        ▼
`pancake-core` Rust crate (platform-independent)
  input sniff → decode → policy and memory budget → encode candidate(s)
      → compare actual outputs → validate → atomic safe export
        │
        ▼
Local `_pancake` folder + final output metadata
```

## Read this before contributing
1. `docs/01-product-and-scope.md` — user-facing promise and MVP limits.
2. `docs/07-image-format-contract.md` — supported still-image format semantics, metadata and transparency policy.
3. `docs/08-batching-output-and-memory.md` — scheduler, cancellation, memory and filesystem guarantees.
4. `docs/10-threat-model.md` — attack paths and mitigations.
5. `docs/24-code-commenting-and-maintainer-guide.md` — idioms and comment standards.
6. `docs/17-codex-execution-plan.md` — ordered work and acceptance tests.

## Key architectural boundaries
- React renders and gathers intent; it does **not** decode original image buffers or choose arbitrary filesystem paths.
- Rust Tauri commands authorise and validate each operation; `pancake-core` works on trusted APIs but untrusted image content.
- Codec wrappers are isolated behind a documented contract so they can be audited and replaced without rewriting the UI.
- Jobs are bounded by estimated decoded bytes and hardware limits. 'Unlimited' means no software batch quotas.
- Progress counts **real finished/failed/cancelled** jobs. Per-file coding stage is indeterminate without genuine codec progress.

## Contributor workflow
Fork/branch → small change → format/test/security checks → PR with explanation and docs → maintainer review → CI → merge. Do not submit private photographs or private filesystem paths as fixtures.

## Maintain this guide
Once the implementation exists, replace conceptual names with actual module paths, diagrams generated from source contracts when practical, and runnable setup commands. Never leave a misleading architecture diagram.
