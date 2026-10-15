# How Pancake Compressor works — a developer tour

> The diagram below is the **target** architecture. As of M0 only the shell exists; see "Where the code lives today" for what is real.

## Where the code lives today (M0)

| Path | What it does |
|---|---|
| `crates/pancake-core/src/lib.rs` | Engine crate root: version, privacy constant with a compile-time check, crate-wide lints (no `unsafe`, no `unwrap`/`panic!`). |
| `crates/pancake-core/src/limits.rs` | Overflow-safe memory estimates and the pre-decode pixel caps (`ImageLimits`, ADR-024). |
| `crates/pancake-core/src/format.rs` | Header-only identification: magic bytes, declared dimensions, alpha hint, animation refusal (ADR-025) and `preflight`, which every file must pass before decoding. Tests and synthetic fixture builders are in `format/tests.rs`. |
| `src-tauri/src/lib.rs` | Builds the Tauri app and registers the only callable commands. |
| `src-tauri/src/commands.rs` | `get_app_info`, the single M0 IPC command (read-only, no file access). |
| `src-tauri/capabilities/main.json` | Built-in Tauri APIs the window may use (deliberately minimal). |
| `src-tauri/tauri.conf.json` | Window, Content Security Policy and installer settings. |
| `src/lib/ipc.ts` | Typed TypeScript mirror of the Rust commands. |
| `src/App.tsx` | Launch screen. |
| `src/styles/tokens.css` | Colour and spacing tokens, with the measured contrast record. |

No decoding, encoding, file selection or batch code exists yet. Header inspection is the first M1 step; codecs follow.

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
