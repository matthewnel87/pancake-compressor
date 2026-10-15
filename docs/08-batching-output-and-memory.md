# 08 — File safety, scheduler, memory and storage

## Security before processing

- Enumeration: traverse only explicitly selected directories; do **not** follow symlinks/reparse points/junctions; ignore output directories and OS hidden metadata where appropriate; detect duplicate canonical paths.
- Path validation: normalise/canonicalise roots, validate relative path components, disallow `..`, device paths and escapes. Validate input and destination are distinct in Rust; guard destination containment, including changes after selection.
- Path checks must be robust against time-of-check/time-of-use changes: use safe handle/path APIs where available, avoid following links at output creation, and never assume string `starts_with()` is a containment proof. On Windows handle case-insensitive filesystems and reserved names.
- Input check: trust file signature and actual decoder support rather than extension alone. Validate upper bounds for dimensions, total pixels, decoded bytes, frame count and decompression ratio before allocating.
- Never invoke shell with file names; paths can contain Unicode, emoji, spaces, quotes or characters meaningful to shell.

## Output policy

1. Default creates `_pancake` near selected root/folder without modifying original; custom output directory available.
2. Preserve nested paths (`img/header/photo.jpg`→`_pancake/img/header/photo.webp`); repeated base names are safe because relative paths are retained.
3. Reject output inside input tree if it would be rescanned, or explicitly exclude `_pancake` subtree consistently.
4. Collision default is numbered rename (`photo-pancake-2.webp`), alternatively skip. Never overwrite existing destination by default. Resolve collisions safely under concurrency.
5. Write to unique temporary file **in target directory** with exclusive-create semantics, flush where appropriate, validate result, then finalize without inadvertently replacing any existing file. Rename semantics differ by OS; use platform-safe no-clobber strategy or generate exclusive destination names.
6. Partial output cleanup on error/cancel/crash recovery; do not delete unrelated files. Completed exports survive cancelled batches.
7. If `avoidLarger` for keep-format is on, skip generating a *larger replacement* (or copy original only with a clearly named policy); per-file state indicates 'not smaller', not successful savings.
8. Report directory permission errors, full disk, read-only inputs, bad names and path length limits without retry loops.

## Memory / CPU model

Memory mapping an input compressed file does **not** prevent large pixel buffers being allocated during decode/encode. E.g. 16,000 × 16,000 × 4 bytes ≈ 1.024 GB for a single RGBA buffer, before intermediates or codec scratch space.

- Probe width/height, channels/bit depth and codec overhead **before** decoding; reject impossible/malicious dimensions and check integer-overflow-safe multiplications.
- Define configurable user memory limit with safe conservative default; respect available system RAM but do not trust a single free-RAM snapshot as a guarantee.
- Use weighted semaphore for predicted live-memory consumption + separate CPU concurrency cap. Do not create unbounded threads from `rayon` tasks that each spawn many codec threads.
- Use one large-job slot by default for high-memory images; small photos may process concurrently. Bound previews to small dimensions and memory.
- Graceful OOM-prevention error when estimated memory unavailable; actual allocations can still fail, so handle errors where supported and avoid panicking.
- For cancellation, use an atomic flag checked at each stage and codec-specific hooks if available; never promise interruption within milliseconds where encoder is opaque.

## Priority test cases

- 1,000 tiny inputs; 100 mid-sized images; one intentionally huge-dimension PNG header; memory budget below decode requirement; symlink cycle; symlink pointing outside selected root; output folder nested under input; `../../escape`; weird Unicode filenames; same-name collisions; mid-write cancellation; disk-full simulation; Windows case collision (`A.JPG`, `a.jpg`); terminal count consistency.

## Performance measurement

Collect elapsed duration, peak resident RAM, output size and throughput on named hardware, not fabricated figures. Benchmark with warm/cache notes, image class, preset, encoder threads and source licences. No 'faster than TinyPNG' claim without reproducible comparisons.
