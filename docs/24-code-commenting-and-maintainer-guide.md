# 24 — Maintainable, explained source code (mandatory)

## Goal
A curious GitHub visitor can understand how Pancake Compressor works without asking its original author. The code is educational **and** production-safe, but not drowned in commentary. Comment the **why, constraints, data flow and surprising decisions**, rather than narrating each straightforward expression.

## Rust standards
- Each core module (`formats`, `decode`, `encode`, `batch`, `paths`, `security`, `io`, `metrics`) starts with a short `//!` description: responsibility, what it does NOT own, and key safety invariants.
- Public types and functions get `///` docs, meaningful parameter definitions, return/error policy and examples when useful. Prefer doctests that compile.
- Record limits and arithmetic units explicitly: input bytes, decoded pixels, allocation estimate, available memory, cancellation boundaries. Include justification and links to benchmarks/design docs where relevant.
- Security-sensitive code must say **why** a path, symlink, temp write or codec result is rejected. Comment every `unsafe` block with a `// SAFETY:` precondition and the test validating it; prefer safe alternatives.
- When rejecting format metadata, comment the user-visible consequence (transparency, ICC/EXIF, animation) and refer to `docs/07-image-format-contract.md`.
- Avoid panic/unwrap on untrusted input; use context-aware error types and propagate stable user-facing outcomes.

## React/TypeScript standards
- Each substantial feature component starts with short purpose comments only when the contract is non-obvious.
- Document Rust/Tauri IPC type contracts including event ordering and snapshot reconciliation; generated types should be preferred to hand-maintained duplicates.
- Explain why event handling is throttled, why files are virtualised, and why the UI cannot claim per-file % during codec calls that expose no progress.
- Use semantic names and pure functions. Don't leave tutorial prose or dead comments in production JSX.

## Example of good comments (illustrative only)

```rust
/// Estimates working memory before decoding a potentially hostile image.
///
/// Dimensions are untrusted. Use checked arithmetic to prevent a wrapped
/// allocation estimate from bypassing the per-job memory budget.
fn estimated_rgba_bytes(width: u32, height: u32) -> Option<u64> {
    u64::from(width).checked_mul(u64::from(height))?.checked_mul(4)
}
```

```ts
// Rust emits file-completion events only after output is validated and renamed.
// Use batchId + sequence to ignore late messages from a cancelled prior batch.
if (event.batchId !== activeBatchId || event.sequence <= lastSequence) return;
```

## Documentation mapping
| Source responsibility | Public doc to update on meaningful changes |
|---|---|
| formats and codec policies | `docs/07-image-format-contract.md`, `docs/ARCHITECTURE-FOR-CONTRIBUTORS.md` |
| batch cancellation, temp files, output | `docs/08-batching-output-and-memory.md`, `docs/SECURITY-OVERVIEW.md` |
| UI flows, progress | `docs/23-ui-ux-flows-performance-v3.md`, end-user FAQ |
| dependencies/licensing | `docs/04-stack-dependencies-and-licences.md`, notices and SBOM |
| build and signing | platform install guides + `docs/14-signing-and-provenance.md` |

## PR review checklist
- Is the public behaviour or threat boundary explained in comments/docs?
- Could another developer understand errors and file states from the tests?
- Did we update docs, examples, diagrams and screenshots when UX or APIs changed?
- Are comments correct and maintained? Stale commentary is worse than no commentary.
- Do tests and CI verify implementation? Markdown alone never establishes a feature works.
