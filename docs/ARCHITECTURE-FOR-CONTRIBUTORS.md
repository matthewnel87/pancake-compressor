# How Pancake Compressor works — a developer tour

> Architecture target; this handoff does not yet contain the implementation. Update the diagram and links to real source when Codex scaffolds code.

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
