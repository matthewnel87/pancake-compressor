# 09 — Pancake Compressor: brand, progress UI and micro-interactions

## Product identity

- **Product name:** Pancake Compressor.
- **Subtitle:** Unlimited image compression and conversion.
- **Optional marketing line:** Squash file sizes, not image quality. This is positioning, not a guarantee of visually lossless output.
- **Tone:** Helpful, calm, lighthearted, never misleading or excessively cutesy.
- **Core product promise:** Free, open source, completely local, no account, no subscription, no artificial usage quota.
- **Brand idea:** A simplified stack of pancakes subtly flattens during active compression and grows a checkmark at completion. Keep simple enough to read at 24 px; it must not make the workflow slower.
- **Naming:** Use display name `Pancake Compressor` and proposed repo slug `pancake-compressor` (subject to name/trademark/repository checks before release). Do not use TinyPNG branding or mascots.

## Design principle: Users must always know what's happening

1. Immediately acknowledge an import: `12 images added` and show queued rows.
2. When running, persist a visible status region, including filename, stage, number completed, and cancellation action.
3. Keep a full-width, **determinate batch** progress track whose denominator is the total of accepted jobs. Caption: `Compressing image 4 of 12` (or `4 of 12 finished` when using completed count).
4. For individual jobs, show state: `Queued`, `Reading`, `Decoding`, `Compressing`, `Saving`, `Done`, `No improvement`, `Skipped`, `Failed`, `Cancelling`, `Cancelled`. Show indeterminate spinner during the codec stage unless the actual codec reports valid proportional progress.
5. Users can see a finished image's input bytes, output bytes and savings. Do not invent ratios while processing.
6. End with a genuinely informative summary (`10 optimised · 2 skipped · 32 MB saved`), and accessible links to output directory and errors.

## Sample desktop copy (illustrative, not actual processing)

```text
[Pancake icon] Pancake Compressor                 [Settings]

             Drop images or folders
          PNG • JPEG • WebP • AVIF

Format [Keep original ▾]   Mode [Smart Auto ▾]

         ◉ Compressing your images…
             4 of 12 files finished
         [████████░░░░░░░░░░░░] 33%

  hero.png           ✓ Done           4.2 MB → 980 KB  −77%
  illustration.webp  ✓ Done           2.1 MB → 700 KB  −67%
  product.jpg        ◌ Compressing    Working…
  logo.png           · Queued

                  [Cancel batch]
```

**Rule:** Batch percentage in this illustration is `floor(100 × completed/total)`; output savings are sample values only. The currently encoding file has an indeterminate icon, *not a fabricated progress percentage*. The displayed `4 of 12 finished` must reconcile with actual completed/skipped/failed terminal state in implementation.

## Motion and feedback

- Brand animation: pancake layers compress gently and release while work is active; subtle 1–1.5s loops, ideally CSS transforms (GPU-friendly) with no layout shift.
- The animation is decoration, **not the measurement of actual progress**.
- After each completed image, an unobtrusive check indicator transitions in; errors show a distinct alert icon.
- Completion state: pancake stack settles, a checkmark appears, and final savings tally is shown.
- For `prefers-reduced-motion: reduce`, disable repetitive and decorative animation; keep static icon + text and real progress updates.
- Do not animate the overall progress bar beyond its actual computed completion ratio. Never freeze the interface while Rust works.
- No confetti, sound or exaggerated celebration by default.

## Technical event model

Tauri commands: `scan_inputs`, `start_batch`, `cancel_batch`, `get_batch_snapshot`. Rust owns authoritative state and counters. UI listens to events, identified by batch ID and monotonic sequence number. Conceptual event shape:

```ts
type FileStage = 'queued'|'reading'|'decoding'|'encoding'|'writing'|'done'|'no_improvement'|'skipped'|'failed'|'cancelling'|'cancelled';
type BatchProgress = {
  batchId: string;
  sequence: number;
  total: number;
  completed: number; // count of terminal jobs; UI can label these as "processed" / "finished"
  active: Array<{ fileId: string; displayName: string; stage: FileStage }>;
  bytesSaved: number; // only finalised jobs
};
```

- Compute percent: `Math.floor(100 * completed / total)` for total > 0; a byte-weighted estimator may be added later, but must not masquerade as precise progress.
- Terminal statuses are `done`, `no_improvement`, `skipped`, `failed`, `cancelled`. Prefer `completed` = terminal jobs, not only successful jobs. When describing counts, distinguish `optimised`, `unchanged`, `skipped`, `failed`.
- Events should include a monotonic sequence to prevent reordering problems and a snapshot RPC to recover after missed events or window reloads.
- Batch ends with a final authoritative snapshot, verified output and any errors. No invented duration or ETA; only show ETA if measured reliably.
- Bounded Rust background jobs and throttled UI events (~5–10 per second, plus guaranteed terminal transitions); failures in one job must not abort the entire queue.

## UX states / acceptance checklist

- [ ] Empty state introduces supported formats and local privacy.
- [ ] Import immediately shows correct number of queued, supported images.
- [ ] Compression shows prominent batch progress and current file stages without blocking the UI.
- [ ] Batch progress continues correctly when 2+ images encode in parallel.
- [ ] No fake per-file percentages, byte-savings claims or ETA.
- [ ] Cancel requests stop unscheduled work, finish/cancel active tasks safely, delete temp files and report each file's outcome.
- [ ] Output status confirms when file is safely written and validated, not just encoded in RAM.
- [ ] `prefers-reduced-motion`, `aria-live=polite` for state change and properly labelled progressbar supported.
- [ ] Progress updates are not announced on every rapid tick; live region announcements are throttled/coalesced.
- [ ] Tab order, keyboard-only actions, contrast and icon+text statuses are tested.
- [ ] Finished screen displays original bytes, final bytes and real savings, plus `Open output folder` action.

## Priority

MVP / M2: real batch progress, per-file stage, visible working state, reduced-motion-safe pancake animation, final result summary. Later enhancement: reliable within-file progress if supported by a codec, history and resumable batches if truly beneficial.
