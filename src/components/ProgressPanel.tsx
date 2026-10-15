/**
 * Batch progress while compressing.
 *
 * The bar is data-driven: finished files over all files (AGENTS.md rule 6).
 * The pancake squish is decoration only and stops when the batch ends; with
 * `prefers-reduced-motion` it does not move at all (docs/23).
 *
 * The screen-reader status is a separate polite live region that changes at
 * meaningful steps (every 10% and at the end), not on every event, so
 * assistive technology is not flooded (docs/12).
 */
import { PancakeMark } from "./PancakeMark";

interface ProgressPanelProps {
  processed: number;
  total: number;
  /** Text for the polite live region; updated by the parent at milestones. */
  announcement: string;
  cancelling: boolean;
  onCancel: () => void;
}

export function ProgressPanel({ processed, total, announcement, cancelling, onCancel }: ProgressPanelProps) {
  const percent = total === 0 ? 0 : Math.floor((processed / total) * 100);
  return (
    <section aria-labelledby="progress-title" className="rounded-card border border-line bg-surface-raised p-5">
      <div className="flex items-center gap-4">
        <PancakeMark className="pancake-squash h-12 w-12 shrink-0" />
        <div className="flex-1">
          <h2 id="progress-title" className="text-lg font-semibold">
            {cancelling ? "Stopping after the current images…" : "Squashing your images…"}
          </h2>
          <p className="tabular-nums text-ink-muted">
            {processed} of {total} processed
          </p>
        </div>
        <button type="button" className="btn-secondary" onClick={onCancel} disabled={cancelling}>
          Cancel
        </button>
      </div>
      <div
        role="progressbar"
        aria-label="Images processed"
        aria-valuemin={0}
        aria-valuemax={total}
        aria-valuenow={processed}
        aria-valuetext={`${processed} of ${total} images processed`}
        className="mt-4 h-3 overflow-hidden rounded-full bg-line"
      >
        <div className="h-full rounded-full bg-accent transition-[width] duration-200" style={{ width: `${percent}%` }} />
      </div>
      <p className="sr-only" role="status" aria-live="polite">
        {announcement}
      </p>
    </section>
  );
}
