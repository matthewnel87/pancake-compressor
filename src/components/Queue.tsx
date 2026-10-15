/**
 * The file list, windowed so a 1,000-file batch stays responsive: only rows
 * near the visible area are rendered (docs/23 performance budget). Rows have
 * a fixed height; ARIA table roles with `aria-rowindex` keep the full list
 * understandable to screen readers even though most rows are not in the DOM.
 *
 * State is shown as text (and size results), never by colour alone.
 */
import { useState } from "react";
import type { FileRow } from "../lib/batchProgress";
import { formatBytes, messageFor, noticeText, stateLabel } from "../lib/text";

const ROW_HEIGHT = 52;
const VIEW_HEIGHT = 360;
const OVERSCAN = 6;

interface QueueProps {
  total: number;
  rows: Record<number, FileRow>;
  /** Called with a file's index when its Compare button is pressed. */
  onCompare?: (fileIndex: number) => void;
}

function resultText(row: FileRow | undefined): string {
  if (!row) return "—";
  if (row.state === "completed" && row.originalBytes !== null && row.outputBytes !== null) {
    return `${formatBytes(row.originalBytes)} → ${formatBytes(row.outputBytes)}`;
  }
  if (row.state === "decoding" || row.state === "encoding" || row.state === "writing") return "Working…";
  return messageFor(row.code) || "—";
}

export function Queue({ total, rows, onCompare }: QueueProps) {
  const [scrollTop, setScrollTop] = useState(0);
  if (total === 0) return null;

  const first = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const last = Math.min(total, Math.ceil((scrollTop + VIEW_HEIGHT) / ROW_HEIGHT) + OVERSCAN);
  const visible = Array.from({ length: last - first }, (_, offset) => first + offset).map((index) => {
    const row = rows[index];
    return (
      <div
        key={index}
        role="row"
        aria-rowindex={index + 2}
        className="absolute left-0 right-0 grid grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,2fr)] items-center gap-3 border-b border-line px-3 text-sm"
        style={{ top: index * ROW_HEIGHT, height: ROW_HEIGHT }}
      >
        <span role="cell" className="truncate" title={row?.file}>
          {row?.file ?? `File ${index + 1}`}
        </span>
        <span role="cell" className="flex items-center gap-2">
          {stateLabel(row?.state ?? "queued")}
          {row?.state === "completed" && onCompare && (
            <button
              type="button"
              className="rounded-control border border-line px-2 py-1 text-xs"
              aria-label={`Compare before and after for ${row.file}`}
              data-compare-index={index}
              onClick={() => onCompare(index)}
            >
              Compare
            </button>
          )}
        </span>
        <span role="cell" className="tabular-nums text-ink-muted">
          <span className="block truncate">{resultText(row)}</span>
          {row && row.notices.length > 0 && (
            <span className="block truncate text-xs">{row.notices.map(noticeText).join(" · ")}</span>
          )}
        </span>
      </div>
    );
  });

  return (
    <div role="table" aria-label="Images in this batch" aria-rowcount={total + 1} className="rounded-card border border-line">
      <div role="rowgroup">
        <div
          role="row"
          aria-rowindex={1}
          className="grid grid-cols-[minmax(0,2fr)_minmax(0,1fr)_minmax(0,2fr)] gap-3 border-b border-line px-3 py-2 text-sm font-semibold"
        >
          <span role="columnheader">Image</span>
          <span role="columnheader">Status</span>
          <span role="columnheader">Result</span>
        </div>
      </div>
      <div
        role="rowgroup"
        tabIndex={0}
        aria-label="Image list, scrollable"
        className="relative overflow-y-auto"
        style={{ height: Math.min(VIEW_HEIGHT, total * ROW_HEIGHT) }}
        onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
      >
        <div style={{ height: total * ROW_HEIGHT }}>{visible}</div>
      </div>
    </div>
  );
}
