/**
 * End-of-batch summary: honest totals that always add up (docs/23 state 4).
 * "Kept original" files are counted separately from savings; nothing is
 * reported as saved unless a smaller file was actually written.
 */
import type { BatchSnapshot } from "../lib/ipc";
import { savingText } from "../lib/text";

interface SummaryProps {
  snapshot: BatchSnapshot;
  onOpenFolder: () => void;
  onStartAgain: () => void;
}

export function Summary({ snapshot, onOpenFolder, onStartAgain }: SummaryProps) {
  const s = snapshot;
  const headline =
    s.cancelled > 0
      ? "Stopped. Completed images are saved; unfinished images were not exported."
      : s.completed > 0
        ? `Nicely done. You saved ${savingText(s.originalBytes, s.outputBytes)}.`
        : "Finished. No smaller versions were needed.";
  const rows: [string, number][] = [
    ["Compressed", s.completed],
    ["Already compact (original kept)", s.notSmaller],
    ["Skipped", s.skipped],
    ["Failed", s.failed],
    ["Not started (cancelled)", s.cancelled],
  ];
  return (
    <section aria-labelledby="summary-title" className="rounded-card border border-line bg-surface-raised p-5">
      <h2 id="summary-title" className="text-lg font-semibold" tabIndex={-1}>
        {headline}
      </h2>
      <dl className="mt-3 grid grid-cols-[1fr_auto] gap-x-6 gap-y-1 text-sm tabular-nums">
        {rows
          .filter(([, count]) => count > 0)
          .map(([label, count]) => (
            <div key={label} className="contents">
              <dt>{label}</dt>
              <dd className="text-right">{count}</dd>
            </div>
          ))}
      </dl>
      <div className="mt-5 flex flex-wrap gap-3">
        {s.completed > 0 && (
          <button type="button" className="btn-primary" onClick={onOpenFolder}>
            Open output folder
          </button>
        )}
        <button type="button" className="btn-secondary" onClick={onStartAgain}>
          Compress more images
        </button>
      </div>
    </section>
  );
}
